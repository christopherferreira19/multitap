use std::fs::File;
use std::process::Command;
use std::io::{self, Write};

fn main() {
    println!("cargo:rerun-if-changed=constants/evdev-constants.py");
    println!("cargo:rerun-if-changed=constants/input.h");
    println!("cargo:rerun-if-changed=constants/input-event-codes.h");
    println!("cargo:rerun-if-changed=src/ids-gen.rs");

	let result = Command::new("constants/evdev-constants.py")
            .args(&[
            	"constants/input.h",
            	"constants/input-event-codes.h"
            ])
            .output()
            .expect("failed to execute process");

    let mut output = File::create("src/ids-gen.rs").expect("failed to create/open output file");
    output.write_all(&result.stdout).unwrap();
    io::stderr().write_all(&result.stderr).unwrap();
}
