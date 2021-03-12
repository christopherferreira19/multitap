use super::evdev;

use multitap_core::{input, config};
use multitap_daemon_core::PhysDevice as PhysDeviceTrait;

pub use error::Error;
pub use create::create;

pub struct PhysDevice {
    driver: String,
    device: evdev::Device
}

impl From<(String, evdev::Device)> for PhysDevice {
    fn from((driver, device): (String, evdev::Device)) -> Self { Self { driver, device } }
}

impl PhysDeviceTrait for PhysDevice {
    type Error = error::Error;

    fn spec(&self) -> config::device::DeviceSpec {
        config::device::DeviceSpec {
            vendor:  self.device.vendor_id() as _,
            product: self.device.product_id() as _,
            driver: self.driver.clone(),
        }
    }

    fn axis_info(&self, axis: input::AxisId) -> Option<input::AxisInfo> {
        let code = {
            let event_type = evdev_rs::enums::EventType::EV_ABS as u32;
            let event_code = axis.0 as u32;
            evdev_rs::enums::EventCode::EV_UNK { event_type, event_code }
        };
        match self.device.abs_info(&code) {
            Some(info) => Some(input::AxisInfo {
                value: info.value,
                min:   info.minimum,
                max:   info.maximum,
                flat:  info.flat,
            }),
            None => None
        }
    }
}

pub mod error {
    use snafu::Snafu;
    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(in super)))]
    pub enum Error {
        #[snafu(display("While opening device event file {}: {}", path.display(), source))]
        OpenEventFile { path: std::path::PathBuf, source: std::io::Error },
        #[snafu(display("While creating evdev device {}: {}", path.display(), source))]
        CreateEvdevDevice { path: std::path::PathBuf, source: std::io::Error },
        #[snafu(display("While grabbing device {}: {}", path.display(), source))]
        GrabDevice { path: std::path::PathBuf, source: std::io::Error },
        #[snafu(display("While dup'ing device fd {}: {}", path.display(), source))]
        DupEventFile { path: std::path::PathBuf, source: nix::Error },
    }
}

mod create {
    use std::path::Path;
    use std::fs::{File, OpenOptions};
    use std::os::unix::io::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    use std::io::ErrorKind::PermissionDenied;

    use snafu::ResultExt;
    use nix::libc::O_NONBLOCK;

    use multitap_core::log::*;
    use multitap_daemon_core::{Platform, eventloop::EventHandler};
    use crate::{eventloop, evdev, virt};
    use super::error;

    pub fn create<P, H>(devnode: &Path, driver: String, handler: &mut H) -> Result<Option<P::Source>, error::Error>
    where
        P: Platform,
        P::Source: From<eventloop::DeviceSource>,
        P::PhysDevice: From<(String, evdev::Device)>,
        H: EventHandler<Platform = P>,
    {
        let file = {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(O_NONBLOCK)
                .open(devnode);

            match file {
                Err(ref error) if error.kind() == PermissionDenied => {
                    trace!("Ignoring device {}: Permission denied", devnode.display());
                    return Ok(None)
                },
                other => other.with_context(|| error::OpenEventFile { path: devnode.to_owned() })?,
            }
        };

        let mut device = evdev::Device::new_from_file(file).with_context(|| error::CreateEvdevDevice { path: devnode.to_owned() })?;
        if device.vendor_id() == virt::VENDOR_ID {
            return Ok(None)
        }

        device.grab(evdev::GrabMode::Grab).with_context(|| error::GrabDevice { path: devnode.to_owned() })?;

        let file = file_dup(&device.file()).with_context(|| error::DupEventFile { path: devnode.to_owned() })?;
        
        let h_device = P::PhysDevice::from((driver, device));
        let id = handler.on_device_add(h_device);
        if let Some(id) = id {
            let device = evdev::Device::new_from_file(file).with_context(|| error::CreateEvdevDevice { path: devnode.to_owned() })?;
            let device = eventloop::DeviceSource { device, id };
            let device = P::Source::from(device);
            Ok(Some(device))
        }
        else {
            Ok(None)
        }
    }

    fn file_dup(file: &File) -> Result<File, nix::Error> {
        use std::os::unix::io::FromRawFd;
        
        let newfd = nix::unistd::dup(file.as_raw_fd())?;
        let newfile = unsafe { File::from_raw_fd(newfd) };
        Ok(newfile)
    }
}

impl Drop for PhysDevice {
    fn drop(&mut self) {
        if self.device.grab(evdev::GrabMode::Ungrab).is_err() {
            // Best effort
        }
    }
}
