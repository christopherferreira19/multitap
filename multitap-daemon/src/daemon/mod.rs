mod devices;
mod ports;
mod plugs;
mod events;

use crate::prelude::*;

pub struct Daemon {
    config:       config::Config,
    devices:      devices::Devices,
    ports:        ports::Ports,
    plugs:        plugs::Plugs,
    autoplug:     autoplug::AutoPlug,
    monitors:     Vec<backend::ClientWrite>,
}

impl Daemon {

	pub fn new(config: config::Config) -> Self {
        let devices = devices::Devices::new(&config);
        let ports = ports::Ports::new(&config);
        let plugs = plugs::Plugs::new();//HashMap::new();
        let autoplug = autoplug::AutoPlug::new(&config);
        let monitors = vec![];

        Self {
            config, 
            ports, devices,
            plugs, autoplug, monitors,
        }
    }

    fn register_monitor<CF: ClientWriteFactory<backend::Platform>>(&mut self, client: CF) -> protocol::Result {
        self.monitors.push(client.create());

        let mut ports = vec![];
        for (_, port) in &self.ports {
            if port.config.name == "Gamepad" {
                ports.push(port.name.clone());
            }
        }

        Ok(protocol::Response::MonitorInit(protocol::MonitorInit { ports }))
    }

    fn plug(&mut self, plug: protocol::PlugArgs) -> protocol::Result {
        let (deviceids, ports, adapter) = {
            let devices = self.devices.find_from_plugargs(&plug)?;
            let ports = self.ports.find_from_plugargs(&plug)?;
    
            let ports_kind: Vec<String> = ports.iter().map(|p| p.config.name.clone()).collect();
            let adapter_name = plug.adapter.or_else(|| {
                let adapter_names: Vec<Option<String>> = devices.iter().map(|d| d.config.find_adapter(&ports_kind)).collect();
                adapter_names.iter().fold(adapter_names[0].clone(), |l, r| if l == *r { l } else { None })
            });
    
            let adapter_name = match adapter_name {
                Some(adapter_name) => adapter_name,
                None => return Err(format!("No default adapter found for {:?}", ports_kind)),
            };
    
            let adapter = match self.config.find_adapter_by_name(&adapter_name) {
                Some(adapter) => adapter,
                None => return Err(format!("Adapter {} not found", adapter_name)),
            };

            let deviceids: Vec<DeviceId> = devices.iter().map(|d| d.id).collect();
            (deviceids, ports, adapter)
        };

        for deviceid in deviceids {
            let device_name = &self.devices[deviceid].name.clone();
            let port_names: Vec<String> = ports.iter().map(|p| p.name.clone()).collect();

            let result = self.plugs.register(adapter, &self.devices[deviceid], &ports);
            match result {
                Ok(plug) => {
                    self.devices.register_plug(deviceid, plug.id);
                    for client in &mut self.monitors {
                        let plug = protocol::Event::Plugged(protocol::PluggedEvent {
                            device: device_name.clone(),
                            ports: port_names.clone(),
                            adapter: plug.name.clone(),
                        });
                        client.write(&plug).unwrap();
                    }

                    info!("[Device {:?}] Plugged into {:?} using {:?}",
                        device_name,
                        port_names,
                        adapter.name);
                },
                Err(err) => {
                    warn!("[Device {:?}] Plugging into {:?} using {:?} error {:?}",
                        device_name,
                        port_names,
                        adapter.name,
                        err);
                }
            }
        }

        Ok(protocol::Response::Ack)
    }

    fn reset(&mut self) -> protocol::Result {
        warn!("Reset !");
        Ok(protocol::Response::Ack)
    }
}
