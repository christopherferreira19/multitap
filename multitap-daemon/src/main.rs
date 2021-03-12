pub mod device;
pub mod port;
pub mod plug;
pub mod autoplug;
pub mod error;
pub mod daemon;

use std::io::Write;
use multitap_core::config;
pub use multitap_core::log::*;
use multitap_daemon_core::eventloop;

#[cfg(target_os = "linux")]
pub use multitap_daemon_linux as backend;
#[cfg(target_os = "android")]
pub use multitap_daemon_android as backend;

mod prelude {
    pub use std::io::Write;
    pub use multitap_core::{config, protocol, input::{self, ShortDisplay}};
    pub use multitap_core::log::*;
    pub use multitap_daemon_core::{eventloop, DeviceId, PhysDevice, VirtDevice, ClientWrite, ClientWriteFactory};
    pub use super::{backend, device, port, plug, autoplug, error, daemon};
    pub use port::PortId;
    pub use plug::PlugId;
    pub use error::Result;
}

fn main() -> error::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .format(|buf, record| writeln!(buf, "[{}][{}] {}", record.level(), record.target(), record.args()))
        .parse_default_env()
        .init();

    let mut eventloop = eventloop::EventLoop::new().unwrap();

    loop {
        warn!("Starting");
        warn!("Loading configuration");
        let config = config::Config::read().unwrap();
        let mut daemon = daemon::Daemon::new(config);

        warn!("Running");
        use eventloop::EventLoopResult::*;
        match eventloop.run(&mut daemon).unwrap() {
            Reload => {
                warn!("Reloading");
                continue
            },
            Term => {
                warn!("Terminating");
                break
            },
        }
    }

    Ok(())
}
