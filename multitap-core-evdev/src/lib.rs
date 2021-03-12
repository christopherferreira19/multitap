pub mod ids;

use nix::libc::{c_uint, c_char};
#[allow(non_camel_case_types)]
pub type __u16 = nix::libc::c_ushort;
pub use evdev_sys::{timeval, input_event, input_absinfo};