use serde_derive::{Serialize, Deserialize};
use structopt::StructOpt;

#[cfg(target_os = "linux")]
pub const DEFAULT_SOCK: &str = "/tmp/multitapd.sock";
#[cfg(target_os = "android")]
pub const DEFAULT_SOCK: &str = "multitapd.sock";

pub const DELIMITER: u8 = b'\0';

#[derive(StructOpt, Debug, Serialize, Deserialize)]
pub enum Command {
    #[structopt(name = "reload")]
    /// Reload the daemon
    Reload,
    #[structopt(name = "quit")]
    /// Quit the daemon
    Quit,
    #[structopt(name = "ping")]
    /// Ping the daemon
    Ping,
    #[structopt(name = "plug")]
    /// Plug
    Plug(PlugArgs),
    #[structopt(name = "reset")]
    /// Reset (Unplug all)
    Reset,
    #[structopt(name = "monitor")]
    /// Monitor
    Monitor,
}

#[derive(StructOpt, Debug, Serialize, Deserialize)]
pub struct PlugArgs {
    #[structopt(short = "d", long = "device")]
    pub devices:  Vec<String>,
    #[structopt(short = "p", long = "port")]
    pub ports: Vec<String>,
    #[structopt(short = "a", long = "adapter")]
    pub adapter: Option<String>,
}

pub type Result = std::result::Result<Response, String>;

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Ack,
    MonitorInit(MonitorInit),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorInit {
    pub ports: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Plugged(PluggedEvent),
    Unplug(UnplugEvent),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluggedEvent {
    pub device: String,
    pub ports: Vec<String>,
    pub adapter: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnplugEvent {
    pub ports: Vec<String>,
}
