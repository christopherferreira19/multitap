
use super::evdev;
use evdev::{AbsInfo, LibevdevWrapper};
use evdev::uinput::UInputDevice;

use multitap_core::{config, input};
use multitap_daemon_core::VirtDevice as VirtDeviceTrait;
use super::convert::{AsEventCode, AsEvdevEvent};

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Nix(nix::Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self { Error::IO(error) }
}

impl From<nix::Error> for Error {
    fn from(error: nix::Error) -> Self { Error::Nix(error) }
}

pub const VENDOR_ID: u16 = 0x0011;

pub struct VirtDevice(pub evdev::UInputDevice);

impl VirtDeviceTrait for VirtDevice {
    type Error = Error;

    fn create(name: &str, config: &config::port::Port) -> Result<Self, Self::Error> {
        let device_name = format!("Multitap<{}>", name);
        let device = evdev_rs::UninitDevice::new().unwrap();
        device.set_name(&device_name);
        device.set_bustype(evdev_rs::enums::BusType::BUS_USB as u16);
        device.set_vendor_id(VENDOR_ID as u16);
        device.set_product_id(config.product_id as u16);
        device.set_version(config.version as u16);

        for key in config.keys.values() {
            device.enable_event_code(&key.as_event_code(), None)?;
        }

        for motion in config.motions.values() {
            device.enable_event_code(&motion.as_event_code(), None)?;
        }

        for axis in config.axes.values() {
            let info = utils::axis_to_absinfo(axis);
            device.enable_event_code(&axis.id.as_event_code(), Some(&info))?;
        }

        let uinput = UInputDevice::create_from_device(&device)?;
        utils::configure_uinput_file(uinput.as_fd().unwrap())?;

        Ok(Self(uinput))
    }

    fn emit_event(&self, event: &input::InputEvent) -> Result<(), Self::Error> {
        self.0.write_event(&event.as_evdev_event())?;
        Ok(())
    }
}

mod utils {
    use std::os::unix::io::RawFd;
    use nix::fcntl::{fcntl, FcntlArg::F_SETFL, OFlag};
    use super::*;

    pub(in super) fn axis_to_absinfo(axis: &config::port::Axis) -> AbsInfo {
        evdev_rs::AbsInfo {
            value:      0,
            minimum:    axis.min,
            maximum:    axis.max,
            fuzz:       0,
            flat:       axis.flat,
            resolution: 1,
        }
    }

    pub(in super) fn configure_uinput_file(fd: RawFd) -> Result<(), Error> {
        fcntl(fd, F_SETFL(OFlag::O_NONBLOCK))?;
        Ok(())
    }
}