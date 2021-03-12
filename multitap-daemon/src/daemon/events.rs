use crate::prelude::*;
use daemon::Daemon;

impl eventloop::EventHandler for Daemon {

    type Platform = backend::Platform;

    fn on_command<CF: ClientWriteFactory<backend::Platform>>(&mut self, client: CF, command: protocol::Command) -> protocol::Result {
        use protocol::Command::*;
        match command {
            Reload => panic!("Unexpected Reload command !"),
            Quit => panic!("Unexpected Quit command !"),
            Ping => Ok(protocol::Response::Ack),
            Plug(plug) => self.plug(plug),
            Reset => self.reset(),
            Monitor => self.register_monitor(client),
        }
    }

    fn on_device_add(&mut self, device: backend::PhysDevice) -> Option<DeviceId> {
        let spec = device.spec();
        let index = {
            let config = match self.config.find_device_by_spec(&spec) {
                Some(v) => v,
                None => {
                    warn!("[Device] No config found for {:?}", spec);
                    return None
                }
            };

            self.devices.register(config, device)
        };

        Some(index)
    }

    fn on_device_remove(&mut self, deviceid: DeviceId) {
        let (device, plugids) = match self.devices.unregister(deviceid) {
            Ok(device) => device,
            Err(err) => {
                error!("[Daemon] {}", err);
                return;
            }
        };
        for plugid in plugids {
            let plug = self.plugs.unregister(plugid);
            self.devices.unregister_plug(plug.deviceid, plugid);
        }

        info!("[Device {:?}] Unregistered", device.name);
    }

    fn on_input_sync(&mut self, deviceid: DeviceId, event: input::SyncEvent) {
        let plugids = self.devices.plugs(deviceid);
        for &plugid in plugids {
            let plug = &mut self.plugs[plugid];
            let portids = plug.sync_ports(event.clone());
            for &portid in portids {
                let event = &event.clone().into();
                self.ports[portid].emit_event(event).unwrap();
            }
        }
    }

    fn on_input_key_change(&mut self, deviceid: DeviceId, event: input::KeyEvent) {
        let plugids = self.devices.plugs(deviceid);
        if plugids.is_empty() {
            let autoplug = {
                let device = &self.devices[deviceid];
                self.autoplug.on_input_key_change(device, &event)
            };
            if let Some((adapter, indexes)) = autoplug {
                for (_, port) in &self.ports {
                    // TODO: Hardcoded port
                    if port.name == "Gamepad:1" {
                        let port_names: Vec<String> = vec![port.name.clone()];
                        let ports = vec![port];
                        for &deviceid in &indexes {
                            match self.plugs.register(&adapter, &self.devices[deviceid], &ports) {
                                Ok(plug) => {
                                    self.devices.register_plug(deviceid, plug.id);
                                    let device = &self.devices[deviceid];
                                    let event = protocol::Event::Plugged(protocol::PluggedEvent {
                                        device: device.name.clone(),
                                        ports: port_names.clone(),
                                        adapter: plug.name.clone(),
                                    });
                                    for client in &mut self.monitors { 
                                        client.write(&event).unwrap();
                                    }
                
                                    info!("[Device {:?}] Plugged into {:?} using {:?}",
                                        device.name,
                                        port_names,
                                        adapter.name);
                                },
                                Err(err) => {
                                    let device = &self.devices[deviceid];
                                    warn!("[Device {:?}] Plugging into {:?} using {:?} error {:?}",
                                        device.name,
                                        port_names,
                                        adapter.name,
                                        err);
                                }
                            }
                        }
                    }
                }
            }

            let device = &self.devices[deviceid];
            trace!("[Device {}] Unplugged, ignoring {:?}", device.name, event.short_display());
        }
        else {
            for &plugid in plugids {
                let plug = &mut self.plugs[plugid];
                let outputs = plug.translate_key(&event).unwrap();
                if outputs.is_empty() {
                    trace!("[Adapter {:?}] Ignoring {:?}", plug.name, event.short_display());
                }
                else {
                    for (output, portid) in outputs {
                        trace!("[Adapter {:?}] {:?} => {:?}", plug.name, event.short_display(), output.short_display());
                        self.ports[portid].emit_event(&output).unwrap();
                    }
                }
            }
        }
    }

    fn on_input_axis_change(&mut self, deviceid: DeviceId, event: input::AxisEvent) {
        let plugids = self.devices.plugs(deviceid);
        if plugids.is_empty() {
        }
        else {
            for &plugid in plugids {
                let plug = &mut self.plugs[plugid];
                let outputs = plug.translate_axis(&event).unwrap();
                for (output, portid) in outputs {
                    self.ports[portid].emit_event(&output).unwrap();
                }
            }
        }
    }

    fn on_input_motion(&mut self, deviceid: DeviceId, event: input::MotionEvent) {
        let plugids = self.devices.plugs(deviceid);
        if plugids.is_empty() {
            let device = &self.devices[deviceid];
            trace!("[Device {}] Unplugged, ignoring {:?}", device.name, event.short_display());
        }
        else {
            for &plugid in plugids {
                let plug = &mut self.plugs[plugid];
                let outputs = plug.translate_motion(&event).unwrap();
                if outputs.is_empty() {
                    trace!("[Adapter {:?}] Ignoring {:?}", plug.name, event.short_display());
                }
                else {
                    for (output, portid) in outputs {
                        trace!("[Adapter {:?}] {:?} => {:?}", plug.name, event.short_display(), output.short_display());
                        self.ports[portid].emit_event(&output).unwrap();
                    }
                }
            }
        }
    }
}
