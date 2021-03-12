pub mod device;
pub mod port;
pub mod adapter;

use log::*;
use std::fs::File;

#[derive(Debug)]
pub struct Config {
    pub devices:  Vec<device::Device>,
    pub ports:    Vec<port::Port>,
    pub adapters: Vec<adapter::Adapter>,
}

#[derive(Debug)]
pub enum Error {
    NoConfigDirErr,
    FileError(std::io::Error),
    RonError(std::path::PathBuf, ron::de::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::FileError(err)
    }
}

impl Error {
    fn from_ron(path: std::path::PathBuf, err: ron::de::Error) -> Error {
       Error::RonError(path, err)
    }
}

type Result<T> = std::result::Result<T, Error>;

macro_rules! read_directory {
    ($func:ident, $name:ident, $type:ty) => {
        pub fn $func(directory: &std::path::Path) -> Result<Vec<$type>> {
            use std::path::{Path, PathBuf};
            fn rec(base: &Path, path: &PathBuf, components: &mut Vec<$type>) -> Result<()> {
                let fullpath = base.join(path);
                if fullpath.is_dir() {
                    for entry in fullpath.read_dir()? {
                        let entry = entry?;
                        let subpath = path.join(entry.file_name());
                        rec(base, &subpath, components)?;
                    }
                }
                else if fullpath.is_file() {
                    let mut component: $type = match ron::de::from_reader(File::open(&fullpath)?) {
                        Ok(component) => component,
                        Err(err)      => return Err(Error::from_ron(fullpath, err)),
                    };

                    let path = path.with_extension("");
                    component.name.push_str(path.to_str().unwrap());
                    info!(concat!("[", stringify!($name), " \"{}\"] Config read {:?}"), component.name, fullpath);
                    components.push(component);
                }

                Ok(())
            }

            let mut components = Vec::new();
            rec(directory, &PathBuf::new(), &mut components)?;
            Ok(components)
        }
    };
}

impl Config {

    read_directory!(read_devices_directory, Device, device::Device);
    read_directory!(read_ports_directory, Port, port::Port);
    read_directory!(read_adapters_directory, Adapter, adapter::Adapter);

    pub fn read() -> Result<Config> {
        let conf_dir = std::path::Path::new("conf/");
        if !conf_dir.is_dir() {
            return Err(Error::NoConfigDirErr);
        }

        let devices  = Self::read_devices_directory(&conf_dir.join("devices"))?;
        let ports    = Self::read_ports_directory(&conf_dir.join("ports"))?;
        let adapters = Self::read_adapters_directory(&conf_dir.join("adapters"))?;

        Ok(Config { devices, ports, adapters })
    }

    pub fn find_device_by_name<S: AsRef<str>>(&self, name: S) -> Option<&device::Device> {
        self.devices.iter().find(|d| &d.name == name.as_ref())
    }

    pub fn find_device_by_spec(&self, spec: &device::DeviceSpec) -> Option<&device::Device> {
        self.devices.iter().find(|d| d.find(spec))
    }

    pub fn find_adapter_by_name<S: AsRef<str>>(&self, name: S) -> Option<&adapter::Adapter> {
        self.adapters.iter().find(|m| &m.name == name.as_ref())
    }
}
