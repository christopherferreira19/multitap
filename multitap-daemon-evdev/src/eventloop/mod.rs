use multitap_daemon_core::eventloop::prelude::*;
use std::os::unix::io::AsRawFd;

use mio::unix::SourceFd;
use multitap_core_evdev::__u16;
use super::evdev;

use multitap_core::input::{self, InputId};
mod error;
pub use error::{Error, Result};

pub struct DeviceSource {
    pub device: evdev::Device,
    pub id:     DeviceId,
}

fn is_done(err: &std::io::Error) -> bool {
    use nix::libc::{EAGAIN, ENODEV};
    match err.raw_os_error() {
        Some(errno) => errno == EAGAIN || errno == ENODEV,
        None => false,
    }
}

impl<P: Platform> EventSource<P> for DeviceSource {

    type Error = Error;

    fn register<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, token: Token, _: &mut H) -> Result<()> {
        let fd = self.device.file().as_raw_fd();
        SourceFd(&fd).register(registry, token, Interest::READABLE)?;
        Ok(())
    }

    fn deregister<H: EventHandler<Platform = P>>(&mut self, registry: &Registry, handler: &mut H) -> Result<()> {
        let fd = self.device.file().as_raw_fd();
        SourceFd(&fd).deregister(registry)?;
        handler.on_device_remove(self.id);
        Ok(())
    }

    fn read_event<H: EventHandler<Platform = P>>(&mut self, handler: &mut H) -> Result<EventProcessing<<H as EventHandler>::Platform>> {
        match self.device.next_event(evdev::ReadFlag::NORMAL) {
            Ok((_, event)) => { 
                dispatch::dispatch(handler, self.id, event)?;
                Ok(EventProcessing::Continue)
            },
            Err(err) if is_done(&err) => Ok(EventProcessing::Done),
            Err(err) => Err(err.into()),
        }
    }
}

mod dispatch {

    use super::*;

    type Result<T, E = Error> = std::result::Result<T, E>;

    const EV_SYN_CODE: u32 = evdev::enums::EventType::EV_SYN as _;
    const EV_KEY_CODE: u32 = evdev::enums::EventType::EV_KEY as _;
    const EV_ABS_CODE: u32 = evdev::enums::EventType::EV_ABS as _;
    const EV_REL_CODE: u32 = evdev::enums::EventType::EV_REL as _;

    pub fn dispatch<H>(handler: &mut H, deviceid: DeviceId, event: evdev::InputEvent) -> Result<()>
    where
        H: EventHandler,
    {
        use evdev::enums::EventCode::*;
        match event.event_code {
            EV_SYN(syn) => sync_event(handler, deviceid, &event, syn as u32)?,
            EV_KEY(key) => key_event(handler, deviceid, &event, key as u32)?,
            EV_ABS(abs) => axis_event(handler, deviceid, &event, abs as u32)?,
            EV_REL(rel) => motion_event(handler, deviceid, &event, rel as u32)?,
            EV_UNK { event_type, event_code } => match event_type {
                EV_SYN_CODE => sync_event(handler, deviceid, &event, event_code)?,
                EV_KEY_CODE => key_event(handler, deviceid, &event, event_code)?,
                EV_ABS_CODE => axis_event(handler, deviceid, &event, event_code)?,
                EV_REL_CODE => motion_event(handler, deviceid, &event, event_code)?,
                _ => (),
            },
            _ => (),
        }

        Ok(())
    }

    pub fn timestamp(event: &evdev::InputEvent) -> input::Timestamp {
        let seconds = event.time.tv_sec as _;
        let microseconds = event.time.tv_usec as _;
        input::Timestamp { seconds, microseconds }
    }

    pub fn sync_event<H>(handler: &mut H, deviceid: DeviceId, event: &evdev::InputEvent, code: u32) -> Result<()>
    where
        H: EventHandler,
    {
        let timestamp = timestamp(event);
        let kind = input::SyncKind::from_raw(code as __u16);
        let event = input::SyncEvent { timestamp, kind };
        handler.on_input_sync(deviceid, event);
        Ok(())
    }

    pub fn key_event<H>(handler: &mut H, deviceid: DeviceId, event: &evdev::InputEvent, code: u32) -> Result<()>
    where
        H: EventHandler,
    {
        let timestamp = timestamp(event);
        let id = input::KeyId::from_raw(code as __u16);
        let state = match event.value {
            0 => input::KeyState::Released,
            1 => input::KeyState::Pressed,
            2 => input::KeyState::AutoRepeat,
            state => return Err(Error::UnsupportedKeyState(state)),
        };
        let event = input::KeyEvent { timestamp, id, state };
        handler.on_input_key_change(deviceid, event);
        Ok(())
    }

    pub fn axis_event<H>(handler: &mut H, deviceid: DeviceId, event: &evdev::InputEvent, code: u32) -> Result<()>
    where
        H: EventHandler,
    {
        let timestamp = timestamp(event);
        let id = input::AxisId::from_raw(code as __u16);
        let state = input::AxisState(event.value as isize);
        let event = input::AxisEvent { timestamp, id, state };
        handler.on_input_axis_change(deviceid, event);
        Ok(())
    }

    pub fn motion_event<H>(handler: &mut H, deviceid: DeviceId, event: &evdev::InputEvent, code: u32) -> Result<()>
    where
        H: EventHandler,
    {
        let timestamp = timestamp(event);
        let id = input::MotionId::from_raw(code as __u16);
        let state = input::MotionState(event.value as isize);
        let event = input::MotionEvent { timestamp, id, state };
        handler.on_input_motion(deviceid, event);
        Ok(())
    }
}

