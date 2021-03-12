use crate::prelude::*;
use std::collections::HashSet;

use multitap_core::{
    config::{
        Config,
        adapter::Adapter as AdapterConfig,
    },
    input::{KeyId, KeyEvent, KeyState},
};
use crate::device::Device;

pub struct AutoPlug {
    per_adapters: Vec<PerAdapter>,
}

struct PerAdapter {
    adapter:  AdapterConfig,
    triggers: Vec<Trigger>,
}

struct Trigger {
    device_name: String,
    key_id:      KeyId,
    candidates:  HashSet<DeviceId>,
}

impl AutoPlug {
    pub fn new(config: &Config) -> Self {
        let mut per_adapters = Vec::new();

        for adapter in &config.adapters {
            if let Some(per_adapter) = PerAdapter::new(config, adapter) {
                per_adapters.push(per_adapter);
            }
        }

        Self { per_adapters }
    }

    pub fn on_input_key_change(&mut self, device: &Device, event: &KeyEvent) -> Option<(AdapterConfig, Vec<DeviceId>)> {
        for per_adapter in &mut self.per_adapters {
            if let Some(autoplug) = per_adapter.on_input_key_change(device, event) {
                return Some(autoplug);
            }
        }

        return None;
    }
}

impl PerAdapter {
    fn new(config: &Config, adapter: &AdapterConfig) -> Option<Self> {
        let adapter = adapter.clone();
        let mut triggers = Vec::new();
        for device in &adapter.inputs {
            for (device_name, key_name) in &adapter.autoplug {
                if &device.name == device_name {
                    if let Some(device_config) = config.find_device_by_name(&device.r#match) {
                        let device_name = device_name.clone();
                        let key_id = device_config.keys[key_name];
                        let candidates = HashSet::new();
                        triggers.push(Trigger { device_name, key_id, candidates });
                    }
                    else {
                        println!("[Autoplug({})] Unrecognized key: {}#{}", adapter.name, device_name, key_name);
                        return None;
                    }
                }
            }
        }

        if triggers.len() > 0 {
            Some(Self { adapter, triggers })
        }
        else {
            None
        }
    }

    fn on_input_key_change(&mut self, device: &Device, event: &KeyEvent) -> Option<(AdapterConfig, Vec<DeviceId>)> {
        let mut indexes = vec![];
        let mut all = true;
        for trigger in &mut self.triggers {
            if trigger.device_name == device.config.name && trigger.key_id == event.id {
                use KeyState::*;
                match event.state {
                    Released => { trigger.candidates.remove(&device.id); },
                    Pressed => { trigger.candidates.insert(device.id); },
                    AutoRepeat => { trigger.candidates.insert(device.id); },
                }
            }

            if trigger.candidates.len() == 1 {
                let candidate = trigger.candidates.iter().next().unwrap();
                if !indexes.contains(candidate) {
                    indexes.push(*candidate);
                }
            }
            else {
                all = false;
            }
        }

        if all {
            for trigger in &mut self.triggers {
                trigger.candidates.clear();
            }
            Some((self.adapter.clone(), indexes))
        }
        else {
            None
        }
    }
}
