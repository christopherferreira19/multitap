use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use jni::JNIEnv;
use jni::errors::Result as JNIResult;
use jni::signature::{JavaType, Primitive};
use jni::sys::jint;
use jni::objects::{GlobalRef, JClass, JObject};

use multitap_core::{
    log::*,
    protocol,
    json::from_slice as json_read,
    json::to_writer as json_write,
};

struct Callbacks<'a> {
    env: JNIEnv<'a>,
    gref: GlobalRef,
}

#[no_mangle]
pub extern "system" fn Java_fr_yusaku_multitap_ui_core_Core_clientReset(
    _env: JNIEnv,
    _class: JClass,
) {
    let mut sock = UnixStream::connect(protocol::DEFAULT_SOCK).unwrap();
    json_write(&mut sock, &protocol::Command::Reset).unwrap();
    sock.write(&[protocol::DELIMITER]).unwrap();

    let mut reader = BufReader::new(sock);
    let (size, buf) = read_from(&mut reader);
    match size {
        0 => return,
        size => {
            trace!("JSON Read: {}", std::str::from_utf8(&buf[0..(size-1)]).unwrap());
            let response = json_read::<protocol::Result>(&buf[..(size-1)]).unwrap();
            match response {
                Ok(protocol::Response::Ack) => { info!("Got response Ack"); },
                unexpected => error!("Unexpected response to Monitor command. Expected MonitorInit, got {:?}", unexpected)
            }
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_fr_yusaku_multitap_ui_core_Core_clientMonitor(
    env: JNIEnv,
    _class: JClass,
    callbacks: JObject,
) {
    let gref = env.new_global_ref(callbacks).unwrap();
    let callbacks = Callbacks { env, gref };

    let mut sock = UnixStream::connect(protocol::DEFAULT_SOCK).unwrap();
    json_write(&mut sock, &protocol::Command::Monitor).unwrap();
    sock.write(&[protocol::DELIMITER]).unwrap();

    let mut reader = BufReader::new(sock);
    let (size, buf) = read_from(&mut reader);
    match size {
        0 => return,
        size => {
            trace!("JSON Read: {}", std::str::from_utf8(&buf[0..(size-1)]).unwrap());
            let response = json_read::<protocol::Result>(&buf[..(size-1)]).unwrap();
            match response {
                Ok(protocol::Response::MonitorInit(init)) => {
                    info!("Got response MonitorInit({:?})", init);
                    callbacks.init(init);
                },
                unexpected => error!("Unexpected response to Monitor command. Expected MonitorInit, got {:?}", unexpected)
            }
            
        }
    }

    monitor(callbacks, reader);
}

fn read_from<R: BufRead>(read: &mut R) -> (usize, Vec<u8>) {
    let mut buf = Vec::with_capacity(4096);
    let size = read.read_until(protocol::DELIMITER, &mut buf).unwrap();
    (size, buf)
}

fn monitor<R: BufRead>(callbacks: Callbacks, mut read: R) {
    loop {
        let (size, buf) = read_from(&mut read);
        match size {
            0 => {
                info!("Connection closed");
                break
            },
            size => {
                trace!("JSON Read: {}", std::str::from_utf8(&buf[0..(size-1)]).unwrap());
                let event = json_read::<protocol::Event>(&buf[..(size-1)]).unwrap();
                warn!("{:?}", event);
                match event {
                    protocol::Event::Plugged(plugged) => callbacks.on_plugged(plugged),
                    protocol::Event::Unplug(unplug) => callbacks.on_unplug(unplug), 
                }
            },
        }
    }
}

impl<'a> Callbacks<'a> {

    fn init(&self, init: protocol::MonitorInit) {
        let ports = newStringArrayList(self.env, &init.ports[..])
            .expect("Couldn't create java string!");

        let res = self.env.call_method(
            &self.gref,
            "init",
            "(Ljava/util/List;)V",
            &[ports.into()],
        );

        if let Err(err) = res {
            error!("Error call_method {:?}", err);
            self.env.exception_describe().unwrap();
        }
    }

    fn on_plugged(&self, event: protocol::PluggedEvent) {
        let device = self.env
            .new_string(&event.device)
            .expect("Couldn't create java string!");
        let ports = newStringArrayList(self.env, &event.ports[..])
            .expect("Couldn't create java string!");
        let adapter = self.env
            .new_string(&event.adapter)
            .expect("Couldn't create java string!");
        let res = self.env.call_method(
            &self.gref,
            "onPlugged",
            "(Ljava/lang/String;Ljava/util/List;Ljava/lang/String;)V",
            &[device.into(), ports.into(), adapter.into()],
        );

        if let Err(err) = res {
            error!("Error call_method {:?}", err);
            self.env.exception_describe().unwrap();
        }
    }

    fn on_unplug(&self, event: protocol::UnplugEvent) {
        let ports = newStringArrayList(self.env, &event.ports[..])
            .expect("Couldn't create java string!");
        let res = self.env.call_method(
            &self.gref,
            "onUnplug",
            "(Ljava/util/List;)V",
            &[ports.into()],
        );

        if let Err(err) = res {
            error!("Error call_method {:?}", err);
            self.env.exception_describe().unwrap();
        }
    }
}

#[allow(non_snake_case)]
fn newStringArrayList<'a>(env: JNIEnv<'a>, vec: &[String]) -> JNIResult<JObject<'a>> {
    let java_util_ArrayList = env.find_class("java/util/ArrayList")?;
    let java_util_ArrayList__init__ = env.get_method_id(java_util_ArrayList, "<init>", "(I)V")?;
    let java_util_ArrayList_add = env.get_method_id(java_util_ArrayList, "add", "(Ljava/lang/Object;)Z")?;

    let size: jint = (vec.len() as i32).into();
    let result = env.new_object_unchecked(java_util_ArrayList, java_util_ArrayList__init__, &[size.into()])?;
    for item in vec {
        let bool = JavaType::Primitive(Primitive::Boolean);
        let element = env
            .new_string(&item)
            .expect("Couldn't create java string!");
        env.call_method_unchecked(result, java_util_ArrayList_add, bool, &[element.into()])?;
    }

    return Ok(result);
}
