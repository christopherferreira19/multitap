use crate::prelude::*;

use slab::Slab;

use port::Port;

pub struct Ports {
    ports: Slab<Port>,
}

impl Ports {
    pub fn new(config: &config::Config) -> Self {
        let mut ports = Slab::with_capacity(64);
        for config_port in &config.ports {
            for slot in &config_port.slots {
                let entry = ports.vacant_entry();
                let id = PortId(entry.key());
                let port = Port::new(id, config_port, slot).unwrap();
                info!("[Port \"{}\"] Created", port.name);
                entry.insert(port);
            }
        }

        Ports { ports }
    }

    pub fn find_from_plugargs(&self, plug: &protocol::PlugArgs) -> Result<Vec<&Port>, String> {
        let mut ports = Vec::new();
        for (i, port_name) in plug.ports.iter().enumerate() {
            let port = self.ports.iter().find(|(_, p)| &p.name == port_name);
            if let Some((_, port)) = port {
                ports.push(port);
            }
            else {
                return Err(format!("Port ({}, {}) not found", i, port_name));
            }
        }

        Ok(ports)
    }

    pub fn iter(&self) -> Iter {
        Iter(self.ports.iter())
    }
}

impl std::ops::Index<PortId> for Ports {
    type Output = Port;
    fn index(&self, id: PortId) -> &Port {
        &self.ports[id.0]
    }
}

impl std::ops::IndexMut<PortId> for Ports {
    fn index_mut(&mut self, id: PortId) -> &mut Port {
        &mut self.ports[id.0]
    }
}

pub struct Iter<'a>(slab::Iter<'a, Port>);

impl<'a> Iterator for Iter<'a> {
    type Item = (usize, &'a Port);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for &'a Ports {
    type Item = (usize, &'a Port);
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}
