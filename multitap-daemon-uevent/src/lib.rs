use multitap_daemon_core::eventloop::prelude::*;

use std::marker::PhantomData;
use std::time::Duration;
use std::path::{Path, PathBuf};
use std::os::unix::io::{RawFd, AsRawFd};

use nix::sys::time::TimeSpec;
use nix::sys::timerfd::{self, TimerFd};
use nix::sys::socket;
use mio::unix::SourceFd;

use multitap_daemon_evdev as evdev;

mod error;
pub use error::{Error, Result};

pub struct DeviceMonitor<P: Platform> {
    socket:   RawFd,
    _phantom: PhantomData<P>,
}

pub struct DeviceRegistration<P: Platform> {
    timerfd: Option<TimerFd>,
    syspath: PathBuf,
    devnode: PathBuf,
    _phantom: PhantomData<P>,
}

impl<P: Platform> DeviceMonitor<P> {

    pub fn new() -> Result<Self> {
        let pid = 0;
        let groups = 1; // Only consider message from root group
        let addr = socket::NetlinkAddr::new(pid, groups);
        let addr = socket::SockAddr::Netlink(addr);
        let socket = socket::socket(
            socket::AddressFamily::Netlink,
            socket::SockType::Datagram,
            socket::SockFlag::SOCK_NONBLOCK,
            socket::SockProtocol::NetlinkKObjectUEvent,
        )?;
        socket::bind(socket, &addr)?;

        let _phantom = PhantomData;
        Ok(Self { socket, _phantom })
    }
}

impl<P: Platform> DeviceMonitor<P>
where
    P::Source: From<evdev::eventloop::DeviceSource>,
    P::PhysDevice: From<(String, evdev::evdev::Device)>,
{
    pub fn enumerate<H: EventHandler<Platform = P>>(&mut self, handler: &mut H) -> Result<Vec<<<H as EventHandler>::Platform as Platform>::Source>> {
        use std::io::BufRead;

        let file = std::fs::File::open("/proc/bus/input/devices")?;
        let reader = std::io::BufReader::new(file);
        let mut syspath = None;
        let mut devnode = None;
        let mut devices = vec![];
        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                syspath.take();
                devnode.take();
            }
            else if line.starts_with("S: ") {
                let idx = line.find('=').unwrap();
                syspath = Some(Path::new("/sys").join(&line[idx+2..]));
                
            }
            else if line.starts_with("H: ") {
                for word in line.split(&[' ', '='][..]) {
                    if word.contains("event") {
                        devnode = Some(Path::new("/dev/input").join(word));
                    }
                }
            }
            else if line.starts_with("B: KEY=") {
                let syspath = syspath.take().unwrap();
                let devnode = devnode.take().unwrap();
                if let Some(device) = utils::device_check_add(handler, &syspath, &devnode)? {
                    devices.push(device);
                }

            }
        }
        
        Ok(devices)
    }
}

impl<P: Platform> EventSource<P> for DeviceMonitor<P>
where
    P::Source: From<DeviceRegistration<P>>,
    P::PhysDevice: From<(String, evdev::evdev::Device)>,
{
    type Error = Error;

    fn register<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, token: Token, _: &mut H) -> Result<()> {
        let fd = self.socket.as_raw_fd();
        SourceFd(&fd).register(registry, token, Interest::READABLE)?;
        Ok(())
    }

    fn deregister<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, _: &mut H) -> Result<()> {
        let fd = self.socket.as_raw_fd();
        SourceFd(&fd).deregister(registry)?;
        Ok(())
    }

    fn read_event<H: EventHandler<Platform = P>>(&mut self, _: &mut H) -> Result<EventProcessing<P>> {
        let mut buf = [0u8; 8192];
        let flags = socket::MsgFlags::empty();

        let size = match socket::recv(self.socket, &mut buf, flags) {
            Ok(0) => return Ok(EventProcessing::Done),
            Ok(size) => size,
            Err(nix::Error::Sys(nix::errno::EWOULDBLOCK)) => return Ok(EventProcessing::Done),
            Err(err) => return Err(Error::from(err)),
        };

        let msg = unsafe { std::str::from_utf8_unchecked(&buf[0..size]) };
        let mut action = None;
        let mut devpath = None;
        let mut devnode = None;
        for line in msg.split_terminator('\0') {
            if let Some(idx) = line.find('=') {
                let key = &line[..idx];
                let value = &line[idx+1..];
                match key {
                    "ACTION"  => { action = Some(value); },
                    "DEVPATH" => { devpath = Some(Path::new("/sys").join(&value[1..])); },
                    "DEVNAME" => { devnode = Some(Path::new("/dev").join(value)); },
                    _ => (),
                }
            }
        }

        match (action, devnode) {
            (Some("add"), Some(devnode)) if devnode.to_str().unwrap().starts_with("/dev/input/event") => {
                let syspath = devpath.unwrap().parent().unwrap().to_owned();
                trace!("# Detected event input {} {}", syspath.display(), devnode.display());
                let device_registration = DeviceRegistration::new(syspath, devnode)?;
                device_registration.schedule(Duration::from_millis(200))?;
                let device_registration = P::Source::from(device_registration);
                Ok(EventProcessing::Register(device_registration))
            },
            _ => {
                Ok(EventProcessing::Continue)
            },
        }
    }
}

impl<P: Platform> DeviceRegistration<P> {
    fn new(syspath: PathBuf, devnode: PathBuf) -> Result<Self, Error> {
        let timerfd = TimerFd::new(timerfd::ClockId::CLOCK_MONOTONIC, timerfd::TimerFlags::TFD_NONBLOCK)?;
        let timerfd = Some(timerfd);
        let _phantom = PhantomData;
        Ok(Self { timerfd, syspath, devnode, _phantom })
    }

    fn schedule(&self, duration: Duration) -> Result<(), Error> {
        let expiration = TimeSpec::from(duration);
        let expiration = timerfd::Expiration::OneShot(expiration);
        let flags = timerfd::TimerSetTimeFlags::empty();
        self.timerfd.as_ref().unwrap().set(expiration, flags)?;
        Ok(())
    }
}

impl<P: Platform> EventSource<P> for DeviceRegistration<P>
where
    P::Source: From<evdev::eventloop::DeviceSource>,
    P::PhysDevice: From<(String, evdev::evdev::Device)>,
{
    type Error = Error;

    fn register<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, token: Token, _: &mut H) -> Result<()> {
        let fd = self.timerfd.as_ref().unwrap().as_raw_fd();
        SourceFd(&fd).register(registry, token, Interest::READABLE)?;
        Ok(())
    }

    fn deregister<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, _: &mut H) -> Result<()> {
        let fd = self.timerfd.as_ref().unwrap().as_raw_fd();
        SourceFd(&fd).deregister(registry)?;
        Ok(())
    }

    fn read_event<H: EventHandler<Platform = P>>(&mut self, handler: &mut H) -> Result<EventProcessing<P>> {
        if self.timerfd.is_none() {
            return Ok(EventProcessing::Done);
        }

        let mut buf = [0u8; 8];
        nix::unistd::read(self.timerfd.as_ref().unwrap().as_raw_fd(), &mut buf)?;
        {
            // Drop timerfd (& implicitily close it)
            std::mem::take(&mut self.timerfd);
        }

        match utils::device_check_add(handler, &self.syspath, &self.devnode)? {
            Some(source) => Ok(EventProcessing::Register(source)),
            None => Ok(EventProcessing::Continue),
        }
    }
}

mod utils {
    use super::*;

    pub(in super) fn device_check_add<P, H>(handler: &mut H, syspath: &Path, devnode: &Path) -> Result<Option<P::Source>>
    where
        P: Platform,
        P::Source: From<evdev::eventloop::DeviceSource>,
        P::PhysDevice: From<(String, evdev::evdev::Device)>,
        H: EventHandler<Platform = P>,
    {
        let driver = syspath.join("device/driver");
        if !driver.exists() {
            return Ok(None);
        }
        let driver = nix::fcntl::readlink(&driver)?;
        let driver = Path::new(&driver);
        let driver = driver.file_name().unwrap().to_str().unwrap();
        let driver = driver.to_string();
        
        let res = evdev::phys::create(devnode, driver, handler)?;
        Ok(res)
    }
}
