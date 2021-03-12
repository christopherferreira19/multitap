pub use multitap_core_evdev::ids::{InputId, SyncKind, KeyId, AxisId, MotionId};

pub trait ShortDisplay {
    fn short_display(&self) -> String;
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Timestamp {
    pub seconds: usize,
    pub microseconds: usize,
}

#[derive(Clone, Debug)]
pub enum InputEvent {
    Sync(SyncEvent),
    Key(KeyEvent),
    Axis(AxisEvent),
    Motion(MotionEvent),
}

impl ShortDisplay for InputEvent {
    fn short_display(&self) -> String {
        use InputEvent::*;
        match self {
            Sync(e) => e.short_display(),
            Key(e) => e.short_display(),
            Axis(e) => e.short_display(),
            Motion(e) => e.short_display(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SyncEvent {
    pub timestamp: Timestamp,
    pub kind: SyncKind
}

impl From<SyncEvent> for InputEvent {
    fn from(event: SyncEvent) -> Self { InputEvent::Sync(event) }
}

impl ShortDisplay for SyncEvent {
    fn short_display(&self) -> String { format!("Syn({})", self.kind) }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyState {
    Released = 0,
    Pressed = 1,
    AutoRepeat = 2,
}

#[derive(Clone, Debug)]
pub struct KeyEvent {
    pub timestamp: Timestamp,
    pub id: KeyId,
    pub state: KeyState,
}

impl From<KeyEvent> for InputEvent {
    fn from(event: KeyEvent) -> Self { InputEvent::Key(event) }
}

impl ShortDisplay for KeyEvent {
    fn short_display(&self) -> String { format!("Key({}) = {:?}", self.id, self.state) }
}

#[derive(Copy, Clone, Debug)]
pub struct AxisState(pub isize);

pub struct AxisInfo {
    pub value: i32,
    pub min:   i32,
    pub max:   i32,
    pub flat:  i32,
}

#[derive(Clone, Debug)]
pub struct AxisEvent {
    pub timestamp: Timestamp,
    pub id: AxisId,
    pub state: AxisState,
}

impl From<AxisEvent> for InputEvent {
    fn from(event: AxisEvent) -> Self { InputEvent::Axis(event) }
}

impl ShortDisplay for AxisEvent {
    fn short_display(&self) -> String { format!("Axis({}) = {}", self.id, self.state.0) }
}

#[derive(Copy, Clone, Debug)]
pub struct MotionState(pub isize);

#[derive(Clone, Debug)]
pub struct MotionEvent {
    pub timestamp: Timestamp,
    pub id: MotionId,
    pub state: MotionState,
}

impl From<MotionEvent> for InputEvent {
    fn from(event: MotionEvent) -> Self { InputEvent::Motion(event) }
}

impl ShortDisplay for MotionEvent {
    fn short_display(&self) -> String { format!("Motion({}) = {}", self.id, self.state.0) }
}
