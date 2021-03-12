use multitap_core::input::*;
use multitap_core::config::adapter::Mapper;
use crate::device::Device;
use crate::port::Port;
use crate::config::port::Axis;
use crate::config::adapter::Adapter;

#[derive(Debug)]
pub enum Error {
    InvalidTransformCreateCalled,
    UnsupportedAutoRepeatForReverseKey(KeyId),
    Find(String),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub trait Transform<E> {
    fn apply(&mut self, event: &E) -> Result<Vec<InputEvent>>;
}

pub struct TransformCreateContext<'a> {
    pub device: &'a Device,
    pub port: &'a Port,
    pub adapter: &'a Adapter,
}

impl<'a> TransformCreateContext<'a> {
    fn find_input_key(&self, name: &str) -> Result<KeyId> {
        let res = self.device.config.keys.get(name);
        match res {
            Some(val) => Ok(*val),
            None => Err(Error::Find(format!("[Plug {:?}] Unable to find device key/button {:?}", self.adapter.name, name))),
        }
    }
    fn find_output_key(&self, name: &str) -> Result<KeyId> {
        let res = self.port.config.keys.get(name);
        match res {
            Some(val) => Ok(*val),
            None => Err(Error::Find(format!("[Plug {:?}] Unable to find port key/button {:?}", self.adapter.name, name))),
        }
    }
    fn find_input_axis(&self, name: &str) -> Result<AxisId> {
        let res = self.device.config.axes.get(name);
        match res {
            Some(val) => Ok(*val),
            None => Err(Error::Find(format!("[Plug {:?}] Unable to find device axis {:?}", self.adapter.name, name))),
        }
    }
    fn find_input_axisinfo(&self, id: AxisId) -> Result<AxisInfo> {
        let res = self.device.axis_info(id);
        match res {
            Some(val) => Ok(val),
            None => Err(Error::Find(format!("[Plug {:?}] Unable to query device axis info {:?}", self.adapter.name, id))),
        }
    }
    fn find_output_axis(&self, name: &str) -> Result<Axis> {
        let res = self.port.config.axes.get(name);
        match res {
            Some(val) => Ok(*val),
            None => Err(Error::Find(format!("[Plug {:?}] Unable to find port axis {:?}", self.adapter.name, name))),
        }
    }
    fn find_input_motion(&self, name: &str) -> Result<MotionId> {
        let res = self.device.config.motions.get(name);
        match res {
            Some(val) => Ok(*val),
            None => Err(Error::Find(format!("[Plug {:?}] Unable to find device motion {:?}", self.adapter.name, name))),
        }
    }
    fn find_output_motion(&self, name: &str) -> Result<MotionId> {
        let res = self.port.config.motions.get(name);
        match res {
            Some(val) => Ok(*val),
            None => Err(Error::Find(format!("[Plug {:?}] Unable to find port motion {:?}", self.adapter.name, name))),
        }
    }
}

fn single<E: Into<InputEvent>>(event: E) -> Result<Vec<InputEvent>> {
    Ok(vec![event.into()])
}

pub mod key;
pub mod axis;
pub mod motion;
pub use key::KeyTransform;
pub use axis::AxisTransform;
pub use motion::MotionTransform;
