pub use multitap_core::config::port::Port as Config;
use multitap_core::input::InputEvent;
use multitap_daemon_core::VirtDevice;
use crate::backend;

type Error = <backend::VirtDevice as VirtDevice>::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PortId(pub usize);

pub struct Port {
    pub id:       PortId,
    pub name:     String,
    pub fullname: String,
    pub config:   Config,
        backend:  backend::VirtDevice,
}

impl Port {
    pub fn new(id: PortId, config: &Config, slot: &str) -> Result<Port, Error> {
        let config = config.clone();
        let (name, fullname) = match slot {
            "unique" => (config.name.to_string(), config.fullname.to_string()),
            slot => (format!("{}:{}", config.name, slot), format!("{} {}", config.fullname, slot))
        };
        let backend = backend::VirtDevice::create(&name, &config)?;
        Ok(Port { id, name, fullname, config, backend })
    }

    pub fn emit_event(&self, event: &InputEvent) -> Result<(), Error> {
        self.backend.emit_event(&event)?;
        Ok(())
    }
}
