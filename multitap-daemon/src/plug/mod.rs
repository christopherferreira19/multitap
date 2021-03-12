mod table;
mod transform;

use crate::prelude::*;
use config::adapter::{Adapter, Mapper};
use transform::{Transform, KeyTransform, AxisTransform, MotionTransform};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlugId(pub usize);

pub struct Plug {
    pub id:        PlugId,
    pub name:      String,
    pub deviceid:  DeviceId,
    pub portids:   Vec<PortId>,

    keys:     table::Key<Vec<(KeyTransform, PortId)>>,
    axes:     table::Axis<Vec<(AxisTransform, PortId)>>,
    motions:  table::Motion<Vec<(MotionTransform, PortId)>>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Unknown,
    Find(String),
    Transform(transform::Error),
}
type Result<T, E = Error> = std::result::Result<T, E>;

impl From<transform::Error> for Error {
    fn from(error: transform::Error) -> Self {
        Error::Transform(error)
    }
}

macro_rules! try_find {
    ($find:expr, $adapter:expr, $name:expr, $value:expr) => {
        match $find {
            Some(val) => val,
            None => return Err(Error::Find(format!(
                "[Plug {:?}] Unable to find {} {:?}", $adapter.name, $name, $value
            ))),
        }
    };
}

impl Plug {

    pub fn new(
        id:     PlugId,
        config: &Adapter,
        device: &device::Device,
        ports:  &[&port::Port],
    ) -> std::result::Result<Plug, Error> {
        let name = config.name.clone();

        let deviceid = device.id;
        let portids = ports.iter().map(|p| p.id).collect();
        let mut keys = table::Key::default();
        let mut axes = table::Axis::default();
        let mut motions = table::Motion::default();

        for port in ports {
            Plug::fill(config, device, port, &mut keys, &mut axes, &mut motions)?;
        }

        Ok(Plug { id, name, deviceid, portids, keys, axes, motions })
    }

    fn fill(
        adapter: &Adapter,
        device:  &device::Device,
        port:    &port::Port,
        keys:    &mut table::Key<Vec<(KeyTransform, PortId)>>,
        axes:    &mut table::Axis<Vec<(AxisTransform, PortId)>>,
        motions: &mut table::Motion<Vec<(MotionTransform, PortId)>>,
    ) -> std::result::Result<(), Error> {
        let input = try_find!(
            adapter.inputs.iter().find(|i| i.r#match == device.config.name),
            adapter, "input", device.config.name
        );
        let output = try_find!(
            adapter.outputs.iter().find(|i| i.r#match == port.config.name),
            adapter, "output", port.config.name
        );

        let couple = (input.name.clone(), output.name.clone());
        let map = try_find!(adapter.map.get(&couple), adapter, "map", couple);

        let context = transform::TransformCreateContext { device, port, adapter };
        for mapper in map {
            match mapper {
                Mapper::Key(..) |
                Mapper::KeyReversed(..) |
                Mapper::KeyToAxis(..) => {
                    let (input, transform) = KeyTransform::create(&context, mapper)?;
                    keys[input].push((transform, port.id));
                },
                Mapper::Axis(..) |
                Mapper::AxisReversed(..) |
                Mapper::AxisToKeys(..) |
                Mapper::HatToKeys(..) => {
                    let (input, transform) = AxisTransform::create(&context, mapper)?;
                    axes[input].push((transform, port.id));
                }
                Mapper::Motion(..) => {
                    let (input, transform) = MotionTransform::create(&context, mapper)?;
                    motions[input].push((transform, port.id));
                },
            }
        }

        Ok(())
    }

    pub fn sync_ports(&self, _: input::SyncEvent) -> &[PortId] {
        &self.portids
    }

    pub fn translate_key(&mut self, event: &input::KeyEvent) -> Result<Vec<(input::InputEvent, PortId)>> {
        let mut vec = Vec::with_capacity(self.keys[event.id].len());
        for (transform, id) in &mut self.keys[event.id] {
            let id = *id;
            for event in transform.apply(event)? {
                vec.push((event, id))
            }
        }
        Ok(vec)
    }

    pub fn translate_axis(&mut self, event: &input::AxisEvent) -> Result<Vec<(input::InputEvent, PortId)>> {
        let mut vec = Vec::with_capacity(self.axes[event.id].len());
        for (transform, id) in &mut self.axes[event.id] {
            let id = *id;
            for event in transform.apply(event)? {
                vec.push((event, id))
            }
        }
        Ok(vec)
    }

    pub fn translate_motion(&mut self, event: &input::MotionEvent) -> Result<Vec<(input::InputEvent, PortId)>> {
        let mut vec = Vec::with_capacity(self.motions[event.id].len());
        for (transform, id) in &mut self.motions[event.id] {
            let id = *id;
            for event in transform.apply(event)? {
                vec.push((event, id))
            }
        }
        Ok(vec)
    }
}
