pub use multitap_core::{
    input::{AxisId, AxisInfo},
    config::device::Device as Config,
};
use multitap_daemon_core::{DeviceId, PhysDevice};
use crate::{backend, plug};

pub struct Device {
    pub id:       DeviceId,
    pub slot_idx: usize,
    pub name:     String,
    pub fullname: String,
    pub config:   Config,
        backend:  backend::PhysDevice,
    pub plug:     Option<plug::Plug>,
}

impl Device {

    pub fn new(
        id:       DeviceId,
        config:   &Config,
        slot_idx: usize,
        backend:  backend::PhysDevice,
    ) -> Self {
        let config = config.clone();
        let name = format!("{}:{}", config.name, slot_idx + 1);
        let fullname = format!("{} {}", config.fullname, slot_idx + 1);
        let plug = None;
        Self {
            id, slot_idx,
            config, name, fullname,
            backend, plug
        }
    }

    pub fn axis_info(&self, axis: AxisId) -> Option<AxisInfo> {
        self.backend.axis_info(axis)
    }
}
