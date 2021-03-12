use crate::prelude::*;

use std::collections::HashMap;
use slab::Slab;

use config::device::Device as DeviceConfig;
use backend::PhysDevice;
use device::Device;
use plug::PlugId;

pub struct Devices {
    slots:   HashMap<String, Slab<usize>>,
    devices: Slab<(Device, Vec<PlugId>)>,
}

impl Devices {

    pub fn new(config: &config::Config) -> Self {
        let devices = Slab::with_capacity(64);
        let mut slots = HashMap::new();
        for config_device in &config.devices {
            slots.insert(config_device.name.clone(), Slab::with_capacity(64));
        }

        Self { devices, slots }
    }

    pub fn find_from_plugargs(&self, plug: &protocol::PlugArgs) -> Result<Vec<&Device>, String> {
        let mut devices = Vec::new();
        for (i, device_name) in plug.devices.iter().enumerate() {
            let device = self.devices.iter().find(|(_, (d, _))| &d.name == device_name);
            if let Some((_, device)) = device {
                devices.push(&device.0);
            }
            else {
                return Err(format!("Device ({}, {}) not found", i, device_name));
            }
        }

        Ok(devices)
    }

    pub fn plugs(&self, deviceid: DeviceId) -> &Vec<PlugId> {
        &self.devices[deviceid.0].1
    }

    pub fn register(&mut self, config: &DeviceConfig, device: PhysDevice) -> DeviceId {
        let entry = self.devices.vacant_entry();
        let slot = self.slots.get_mut(&config.name).unwrap().vacant_entry();
        let deviceid = DeviceId(entry.key());
        let device = Device::new(deviceid, config, slot.key(), device);
        info!("[Device {:?}] Registered", device.name);
        slot.insert(entry.key());
        entry.insert((device, Vec::with_capacity(4)));

        deviceid
    }

    pub fn register_plug(&mut self, deviceid: DeviceId, plugid: PlugId) {
        self.devices[deviceid.0].1.push(plugid);
    }

    pub fn unregister_plug(&mut self, deviceid: DeviceId, plugid: PlugId) {
        let plugs = &mut self.devices[deviceid.0].1;
        plugs.retain(|i| *i != plugid);
    }

    pub fn unregister(&mut self, deviceid: DeviceId) -> Result<(Device, Vec<PlugId>), String> {
        let device = self.devices.remove(deviceid.0);
        let slots = self.slots.get_mut(&device.0.config.name).unwrap();
        slots.remove(device.0.slot_idx);

        info!("[Device {:?}] Unregistered", device.0.name);
        Ok(device)
    }
}

impl std::ops::Index<DeviceId> for Devices {
    type Output = Device;
    fn index(&self, id: DeviceId) -> &Device {
        &self.devices[id.0].0
    }
}

impl std::ops::IndexMut<DeviceId> for Devices {
    fn index_mut(&mut self, id: DeviceId) -> &mut Device {
        &mut self.devices[id.0].0
    }
}
