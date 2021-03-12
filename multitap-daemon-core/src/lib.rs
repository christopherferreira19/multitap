use std::fmt::Debug;

use multitap_core::{config, input, protocol};

pub mod eventloop;
use eventloop::{EventHandler, EventSource, sources::EventSources};

pub trait Platform: Sized + Debug {
    type Error: Debug;
    type Source: EventSource<Self, Error = Self::Error>;
    type PhysDevice: PhysDevice;
    type ClientWrite: ClientWrite;

    fn initialize<H: EventHandler<Platform = Self>>(sources: &mut EventSources<Self>, handler: &mut H) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeviceId(pub usize);

pub trait PhysDevice: Sized {
    type Error;

    fn spec(&self) -> config::device::DeviceSpec;

    fn axis_info(&self, axis: input::AxisId) -> Option<input::AxisInfo>;
}

pub trait VirtDevice: Sized {
    type Error;

    fn create(name: &str, config: &config::port::Port) -> Result<Self, Self::Error>;

    fn emit_event(&self, event: &input::InputEvent) -> Result<(), Self::Error>;
}

pub trait ClientWrite {
    type Error;
    fn write(&mut self, response: &protocol::Event) -> Result<(), Self::Error>;
}

pub trait ClientWriteFactory<P: Platform> {
    fn create(&self) -> P::ClientWrite;
}
