mod client;
#[cfg(target_os = "linux")]
mod tray;

use std::io::Write;

use multitap_core::log::*;

use jni::JNIEnv;
use jni::objects::JClass;

#[no_mangle]
pub extern "system" fn Java_fr_yusaku_multitap_ui_core_Core_init(
    _env: JNIEnv,
    _class: JClass,
) {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .format(|buf, record| writeln!(buf, "[{}][{}] {}", record.level(), record.target(), record.args()))
        .parse_default_env()
        .init();
}