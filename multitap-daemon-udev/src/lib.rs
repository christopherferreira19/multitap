use multitap_daemon_core::eventloop::prelude::*;

use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;

use mio::unix::SourceFd;

use multitap_daemon_evdev as evdev;

mod error;
pub use error::{Error, Result};

pub struct DeviceMonitor<P: Platform> {
    socket: udev::MonitorSocket,
    _phantom: PhantomData<P>,
}

impl<P: Platform> EventSource<P> for DeviceMonitor<P>
where
    P::Source: From<evdev::eventloop::DeviceSource>,
    P::PhysDevice: From<(String, evdev::evdev::Device)>,
{

    type Error = Error;

    fn register<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, token: Token, _: &mut H) -> Result<()> {
        SourceFd(&self.socket.as_raw_fd()).register(registry, token, Interest::READABLE)?;
        Ok(())
    }

    fn deregister<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, _: &mut H) -> Result<()> {
        SourceFd(&self.socket.as_raw_fd()).deregister(registry)?;
        Ok(())
    }

    fn read_event<H: EventHandler<Platform = P>>(&mut self, handler: &mut H) -> Result<EventProcessing<P>> {
        match self.socket.next() {
            Some(event) => self.device_event(handler, &event),
            None => Ok(EventProcessing::Done),
        }
    }
}

impl<P: Platform> DeviceMonitor<P> {

    pub fn new() -> Result<Self> {
        let mut builder = udev::MonitorBuilder::new()?;
        builder = builder.match_subsystem("input")?;
        let socket = builder.listen()?;
        let _phantom = PhantomData;
        Ok(Self { socket, _phantom })
    }
}

impl<P> DeviceMonitor<P>
where
    P: Platform,
    P::Source: From<evdev::eventloop::DeviceSource>,
    P::PhysDevice: From<(String, evdev::evdev::Device)>,
{
    pub fn enumerate<H: EventHandler<Platform = P>>(&mut self, handler: &mut H) -> Result<Vec<P::Source>> {
        let mut registrables = vec![];
        let mut enumerator = udev::Enumerator::new()?;
        enumerator.match_subsystem("input")?;
        let devices = enumerator.scan_devices()?;
        for device in devices {
            if let Some(registrable) = self.device_check_add(handler, &device)? {
                registrables.push(registrable);
            }
        }

        Ok(registrables)
    }

    fn device_event<H: EventHandler<Platform = P>>(&mut self, handler: &mut H, event: &udev::Event) -> Result<EventProcessing<P>> {
        if event.event_type() != udev::EventType::Add {
            Ok(EventProcessing::Continue)
        }
        else if let Some(source) = self.device_check_add(handler, &event.device())? {
            Ok(EventProcessing::Register(source))
        }
        else {
            Ok(EventProcessing::Continue)
        }
    }

    fn device_check_add<H: EventHandler<Platform = P>>(&self, handler: &mut H, udevice: &udev::Device) -> Result<Option<P::Source>> {
        let sysname = udevice.sysname().to_os_string().into_string()?;
        if !sysname.starts_with("event") {
            return Ok(None);
        }

        let devnode = match udevice.devnode() {
            Some(path) => path,
            None => return Err(Error::UdevNoDevNode),
        };

        let driver = match utils::driver(&udevice)? {
            Some(driver) => driver,
            None => {
                trace!("Ignoring device {}: unable to locate driver", devnode.display());
                return Ok(None)
            },
        };

        let res = evdev::phys::create(devnode, driver, handler)?;
        Ok(res)
    }
}

mod utils {
    use super::*;

    pub fn driver(udevice: &udev::Device) -> Result<Option<String>> {
        let syspath = udevice.syspath();
        for syscomponent in syspath {
            let syscomponent = syscomponent.to_os_string().into_string()?;
            if syscomponent.contains("virtual") {
                return Ok(Some("virtual".to_string()));
            }
        }

        let parent = match udevice.parent() {
            Some(p) => p,
            None    => return Ok(None),
        };
        let pparent = match parent.parent() {
            Some(p) => p,
            None    => return Ok(None),
        };
        let driver  = match pparent.driver() {
            Some(d) => d.to_os_string().into_string(),
            None    => return Ok(None),
        }?;      

        Ok(Some(driver))
    }
}
