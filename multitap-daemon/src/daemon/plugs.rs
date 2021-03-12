use crate::prelude::*;

use slab::Slab;

use plug::Plug;

pub struct Plugs {
    plugs: slab::Slab<Plug>,
}

impl Plugs {
    pub fn new() -> Self {
        let plugs = Slab::with_capacity(64);
        Plugs { plugs }
    }

    pub fn register(&mut self, adapter: &config::adapter::Adapter, device: &device::Device, ports: &[&port::Port]) -> Result<&Plug, plug::Error> {
        let entry = self.plugs.vacant_entry();
        let plugid = PlugId(entry.key());
        let plug = Plug::new(plugid, adapter, device, ports)?;
        entry.insert(plug);
        Ok(&self[plugid])
    }

    pub fn unregister(&mut self, plugid: PlugId) -> Plug {
        self.plugs.remove(plugid.0)
    }
}

impl std::ops::Index<PlugId> for Plugs {
    type Output = Plug;
    fn index(&self, id: PlugId) -> &Plug {
        &self.plugs[id.0]
    }
}

impl std::ops::IndexMut<PlugId> for Plugs {
    fn index_mut(&mut self, id: PlugId) -> &mut Plug {
        &mut self.plugs[id.0]
    }
}
