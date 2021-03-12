use multitap_daemon_core::eventloop::prelude::*;

use multitap_daemon_core as generic;
use multitap_daemon_signals as signals;
use multitap_daemon_uds as uds;
use multitap_daemon_udev as udev;
use multitap_daemon_evdev as evdev;

pub use evdev::PhysDevice;
pub use evdev::VirtDevice;
pub use uds::ClientWrite;

#[derive(Debug)]
pub struct Platform;
impl generic::Platform for Platform {
    type Source = EventSource;
    type Error = Error;
    type PhysDevice = PhysDevice;
    type ClientWrite = uds::ClientWrite;

    fn initialize<H: EventHandler<Platform = Self>>(sources: &mut sources::EventSources<Self>, handler: &mut H) -> Result<(), Error> {
        let signals = EventSource::Signals(signals::Signals::new()?);
        let commands = EventSource::CommandsListener(uds::CommandsListener::new()?);
        sources.register(signals, handler)?;
        sources.register(commands, handler)?;

        let mut device_monitor = udev::DeviceMonitor::new()?;
        let registrables = device_monitor.enumerate(handler)?;
        let device_monitor = EventSource::DeviceMonitor(device_monitor);
        sources.register(device_monitor, handler)?;

        for registrable in registrables {
            sources.register(registrable, handler)?;
        }

        Ok(())
    }
}

pub enum EventSource {
    Signals(signals::Signals),
    CommandsListener(uds::CommandsListener),
    CommandsClient(uds::CommandsClient),
    DeviceMonitor(udev::DeviceMonitor<Platform>),
    Device(evdev::eventloop::DeviceSource),
}

impl From<signals::Signals> for EventSource {
    fn from(source: signals::Signals) -> Self { EventSource::Signals(source) }
}
impl From<uds::CommandsListener> for EventSource {
    fn from(source: uds::CommandsListener) -> Self { EventSource::CommandsListener(source) }
}
impl From<uds::CommandsClient> for EventSource {
    fn from(source: uds::CommandsClient) -> Self { EventSource::CommandsClient(source) }
}
impl From<udev::DeviceMonitor<Platform>> for EventSource {
    fn from(source: udev::DeviceMonitor<Platform>) -> Self { EventSource::DeviceMonitor(source) }
}
impl From<evdev::eventloop::DeviceSource> for EventSource {
    fn from(source: evdev::eventloop::DeviceSource) -> Self { EventSource::Device(source) }
}

impl generic::eventloop::EventSource<Platform> for EventSource {

    type Error = Error;

    fn register<H: EventHandler<Platform = Platform>>(&mut self, registry: &Registry, token: Token, handler: &mut H) -> Result<(), Error> {
        use EventSource::*;
        match self {
            Signals(ref mut s)          => s.register(registry, token, handler)?,
            CommandsListener(ref mut s) => s.register(registry, token, handler)?,
            CommandsClient(ref mut s)   => s.register(registry, token, handler)?,
            DeviceMonitor(ref mut s)    => s.register(registry, token, handler)?,
            Device(ref mut s)           => s.register(registry, token, handler)?,
        }
        Ok(())
    }

    fn deregister<H: EventHandler<Platform = Platform>>(&mut self, registry: &Registry, handler: &mut H) -> Result<(), Error> {
        use EventSource::*;
        match self {
            Signals(ref mut s)          => s.deregister(registry, handler)?,
            CommandsListener(ref mut s) => s.deregister(registry, handler)?,
            CommandsClient(ref mut s)   => s.deregister(registry, handler)?,
            DeviceMonitor(ref mut s)    => s.deregister(registry, handler)?,
            Device(ref mut s)           => s.deregister(registry, handler)?,
        }
        Ok(())
    }

    fn read_event<H: EventHandler<Platform = Platform>>(&mut self, handler: &mut H) -> Result<EventProcessing<Platform>, Error> {
        use EventSource::*;
        let res = match self {
            Signals(ref mut s)          => { let res = s.read_event(handler); res? },
            CommandsListener(ref mut s) => { let res = s.read_event(handler); res? },
            CommandsClient(ref mut s)   => { let res = s.read_event(handler); res? },
            DeviceMonitor(ref mut s)    => { let res = s.read_event(handler); res? },
            Device(ref mut s)           => { let res = s.read_event(handler); res? },
        };
        Ok(res)
    }
}

#[derive(Debug)]
pub enum Error {
    Sources(sources::Error),
    Signals(signals::Error),
    Commands(uds::Error),
    DeviceMonitor(udev::Error),
    Device(evdev::eventloop::Error),
}

impl From<Error> for generic::eventloop::Error<Platform> {
    fn from(err: Error) -> Self { generic::eventloop::Error::Platform(err) }
}
impl From<sources::Error> for Error {
    fn from(err: sources::Error) -> Self { Error::Sources(err) }
}
impl From<signals::Error> for Error {
    fn from(err: signals::Error) -> Self { Error::Signals(err) }
}
impl From<uds::Error> for Error {
    fn from(err: uds::Error) -> Self { Error::Commands(err) }
}
impl From<udev::Error> for Error {
    fn from(err: udev::Error) -> Self { Error::DeviceMonitor(err) }
}
impl From<evdev::eventloop::Error> for Error {
    fn from(err: evdev::eventloop::Error) -> Self { Error::Device(err) }
}
