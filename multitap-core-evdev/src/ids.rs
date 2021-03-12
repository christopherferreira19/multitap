use std::fmt;
use std::convert::TryFrom;
use serde::de::{self, Visitor, Deserializer};
use serde_derive::Serialize;
use super::*;

include!("ids-gen.rs");

pub trait InputId: Sized + Copy {
    const TYPE_CODE: c_uint;
    const TYPE_NAME: &'static str;
    fn from_raw(raw: __u16) -> Self;
    fn as_raw(&self) -> __u16;
    fn from_name<S: AsRef<str>>(name: S) -> Option<Self> {
        let cname = std::ffi::CString::new(name.as_ref()).unwrap();
        let cname = cname.as_ptr() as *const c_char;
        let value = unsafe {
            evdev_sys::libevdev_event_code_from_name(Self::TYPE_CODE, cname)
        };
        if value == -1 {
            None
        }
        else {
            Some(Self::from_raw(value as __u16))
        }
    }
    fn name(&self) -> Option<&'static str> {
        // Get the name as a 'const char*' borrowed from libevdev global names table
        let name = unsafe { evdev_sys::libevdev_event_code_get_name(Self::TYPE_CODE, self.as_raw() as c_uint) };
        if name == std::ptr::null() { return None; }
        // Convert to length-delimited
        let name = unsafe { std::ffi::CStr::from_ptr(name) }.to_str();
        // Should be safe to simply unwrap as the names are identifier, unlikely to raise any UTF-8 issue
        let name = name.unwrap();
        Some(name)
    }
    fn input_id_debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.input_id_display(f)
    }
    fn input_id_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.name() {
            Some(name) => write!(f, "{}", name),
            None => write!(f, "{}({:#X})", Self::TYPE_NAME, self.as_raw()),
        }
    }
}

pub struct InputIdDeserializeVisitor<Id: InputId>(std::marker::PhantomData<Id>);

impl<Id: InputId> InputIdDeserializeVisitor<Id> {
    pub fn new() -> Self { InputIdDeserializeVisitor(std::marker::PhantomData) }
    fn visit_integer<Err>(self, value: u16) -> Result<Id, Err> where Err: de::Error {
        Ok(Id::from_raw(value))
    }
    fn visit_integer_checking_bounds<Int, Err>(self, integer: Int) -> Result<Id, Err>
    where
        u16: std::convert::TryFrom<Int, Error = std::num::TryFromIntError>,
        Int: Copy + fmt::Display,
        Err: de::Error,
    {
        match u16::try_from(integer) {
            Ok(integer) => self.visit_integer(integer),
            Err(err)  => {
                let msg = format!("Integer value {} for input code is out of range: {}", integer, err);
                Err(Err::custom(msg))
            },
        }
    }
}

impl<Id: InputId> Default for InputIdDeserializeVisitor<Id> {
    fn default() -> Self { Self::new() }
}

impl<'de, Id: InputId> Visitor<'de> for InputIdDeserializeVisitor<Id> {
    type Value = Id;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid input name or its unsigned integer code")
    }

    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E> where E: de::Error {
        self.visit_integer(u16::from(value))
    }

    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E> where E: de::Error {
        self.visit_integer(value)
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E> where E: de::Error {
        self.visit_integer_checking_bounds(value)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> where E: de::Error {
        self.visit_integer_checking_bounds(value)
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E> where E: de::Error {
        self.visit_integer_checking_bounds(value)
    }

    fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E> where E: de::Error {
        self.visit_integer_checking_bounds(value)
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E> where E: de::Error {
        self.visit_integer_checking_bounds(value)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> where E: de::Error {
        self.visit_integer_checking_bounds(value)
    }

    serde::serde_if_integer128! {
        fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E> where E: de::Error {
            self.visit_integer_checking_bounds(value)
        }
    
        fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E> where E: de::Error {
            self.visit_integer_checking_bounds(value)
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: de::Error {
        match Id::from_name(value) {
            Some(value) => Ok(value),
            None => Err(E::custom(format!("Unknown input code name: {}", value))),
        }
    }
}
