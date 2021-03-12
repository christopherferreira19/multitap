use evdev_rs as evdev;
use evdev::{TimeVal, InputEvent, enums::{EventType, EventCode}};
use multitap_core::input;

pub trait AsEventCode {
    fn as_event_code(&self) -> EventCode;
}

pub trait AsEvdevEvent {
    fn as_evdev_event(&self) -> InputEvent;
}

fn timeval(timestamp: input::Timestamp) -> TimeVal {
    TimeVal { tv_sec: timestamp.seconds as i64, tv_usec: timestamp.microseconds as i64 }
}

impl AsEventCode for input::SyncKind {
    fn as_event_code(&self) -> EventCode {
        let event_type = EventType::EV_SYN as u32;
        let event_code = self.0 as u32;
        EventCode::EV_UNK { event_type, event_code }
    }
}

impl AsEvdevEvent for input::SyncEvent {
    fn as_evdev_event(&self) -> InputEvent {
        let time = timeval(self.timestamp);
        let event_code = self.kind.as_event_code();
        let value = 0;
        InputEvent { time, event_code, value }
    }
}

impl AsEventCode for input::KeyId {
    fn as_event_code(&self) -> EventCode {
        let event_type = EventType::EV_KEY as u32;
        let event_code = self.0 as u32;
        EventCode::EV_UNK { event_type, event_code }
    }
}

impl AsEvdevEvent for input::KeyEvent {
    fn as_evdev_event(&self) -> InputEvent {
        let time = timeval(self.timestamp);
        let event_code = self.id.as_event_code();
        let value = self.state as _;
        InputEvent { time, event_code, value }
    }
}

impl AsEventCode for input::AxisId {
    fn as_event_code(&self) -> EventCode {
        let event_type = EventType::EV_ABS as u32;
        let event_code = self.0 as u32;
        EventCode::EV_UNK { event_type, event_code }
    }
}

impl AsEvdevEvent for input::AxisEvent {
    fn as_evdev_event(&self) -> InputEvent {
        let time = timeval(self.timestamp);
        let event_code = self.id.as_event_code();
        let value = self.state.0 as _;
        InputEvent { time, event_code, value }
    }
}

impl AsEventCode for input::MotionId {
    fn as_event_code(&self) -> EventCode {
        let event_type = EventType::EV_REL as u32;
        let event_code = self.0 as u32;
        EventCode::EV_UNK { event_type, event_code }
    }
}

impl AsEvdevEvent for input::MotionEvent {
    fn as_evdev_event(&self) -> InputEvent {
        let time = timeval(self.timestamp);
        let event_code = self.id.as_event_code();
        let value = self.state.0 as _;
        InputEvent { time, event_code, value }
    }
}

impl AsEvdevEvent for input::InputEvent {
    fn as_evdev_event(&self) -> InputEvent {
        use input::InputEvent::*;
        match self {
            Sync(event) => event.as_evdev_event(),
            Key(event) => event.as_evdev_event(),
            Axis(event) => event.as_evdev_event(),
            Motion(event) => event.as_evdev_event(),
        }
    }
}
