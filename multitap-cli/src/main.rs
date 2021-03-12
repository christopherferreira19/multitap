use std::path::PathBuf;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use structopt::StructOpt;

use multitap_core::{
    log::*,
	protocol,
	json::from_slice as json_read,
	json::to_writer as json_write,
};

#[derive(StructOpt, Debug)]
#[structopt(name = "multitapctl")]
struct Opts {

    #[structopt(
        short = "s", long = "socket",
        default_value = protocol::DEFAULT_SOCK,
        parse(from_os_str)
    )]
    /// Daemon socket path
    socket: PathBuf,

    #[structopt(subcommand)]
    cmd: protocol::Command,
}

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .format(|buf, record| writeln!(buf, "[{}][{}] {}", record.level(), record.target(), record.args()))
        .parse_default_env()
        .init();
    let opts = Opts::from_args();

    let mut sock = UnixStream::connect(opts.socket).unwrap();
    json_write(&mut sock, &opts.cmd).unwrap();
    sock.write(&[protocol::DELIMITER]).unwrap();
    
    use protocol::Command::*;
    match opts.cmd {
        Reload | Quit => (),
    	_ => {
            let mut reader = BufReader::new(sock);
            let (size, buf) = read_from(&mut reader);
            match size {
                0 => return,
                size => {
                    trace!("JSON Read: {}", std::str::from_utf8(&buf[0..(size-1)]).unwrap());
                    let response = json_read::<protocol::Result>(&buf[..(size-1)]);
                    warn!("{:?}", response.unwrap());
                }
            }

            if let Monitor = opts.cmd {
                monitor(reader);
            }
        },
    }
}

fn read_from<R: BufRead>(read: &mut R) -> (usize, Vec<u8>) {
    let mut buf = Vec::with_capacity(4096);
    let size = read.read_until(protocol::DELIMITER, &mut buf).unwrap();
    (size, buf)
}

fn monitor<R: BufRead>(mut read: R) {
    loop {
        let (size, buf) = read_from(&mut read);
        match size {
            0 => {
                info!("Connection closed");
                break
            },
            size => {
                trace!("JSON Read: {}", std::str::from_utf8(&buf[0..(size-1)]).unwrap());
                let event = json_read::<protocol::Event>(&buf[..(size-1)]);
                warn!("{:?}", event.unwrap());
            },
        }
    }
}
