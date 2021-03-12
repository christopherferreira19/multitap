use std::sync::Arc;

use jni::{JavaVM, JNIEnv};
use jni::sys::{jint, jlong};
use jni::objects::{GlobalRef, JClass, JObject};

use gtk::{GtkMenuItemExt, MenuShellExt, WidgetExt};
use libappindicator::{AppIndicator, AppIndicatorStatus};

use multitap_core::log::*;

struct Tray {
    indicator: AppIndicator,
    menu: gtk::Menu,
}

struct Callbacks {
    jvm: JavaVM,
    gref: GlobalRef,
}

#[no_mangle]
pub extern "system" fn Java_fr_yusaku_multitap_ui_core_Core_trayInit(
    env: JNIEnv,
    _class: JClass,
    callbacks: JObject,
) -> jlong {
    let jvm = env.get_java_vm().unwrap();
    let gref = env.new_global_ref(callbacks).unwrap();
    let callbacks = Arc::new(Callbacks { jvm, gref });

    unsafe {
        gtk_sys::gtk_init_check(std::ptr::null_mut(), std::ptr::null_mut());
        gtk::set_initialized();
    }

    let mut tray = Box::new(Tray {
        indicator: AppIndicator::new("Multitap", ""),
        menu: gtk::Menu::new()
    });

    tray.indicator.set_icon_theme_path("/home/aumgn/Projects/yusaku/multitap/multitap/multitap-ui/src/main/resources/fr/yusaku/multitap/ui/");
    tray.indicator.set_icon("icon");
    tray.indicator.set_attention_icon("icon-alert");

    tray.indicator.set_status(AppIndicatorStatus::Active);

    let open_item = gtk::MenuItem::with_label("Open");
    open_item.connect_activate({
        let callbacks = Arc::clone(&callbacks);
        move |_| callbacks.call("onTrayMenuOpen")
    });

    let quit_item = gtk::MenuItem::with_label("Quit");
    quit_item.connect_activate({
        let callbacks = Arc::clone(&callbacks);
        move |_| callbacks.call("onTrayMenuQuit")
    });

    tray.menu.append(&open_item);
    tray.menu.append(&quit_item);
    tray.indicator.set_menu(&mut tray.menu);

    return Box::into_raw(tray) as jlong;
}

#[no_mangle]
pub extern "system" fn Java_fr_yusaku_multitap_ui_core_Core_trayDrop(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    let handle = handle as *mut Tray;
    let tray = unsafe { Box::from_raw(handle) };
    drop(tray);
}

#[no_mangle]
pub extern "system" fn Java_fr_yusaku_multitap_ui_core_Core_trayShow(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    let handle = handle as *mut Tray;
    let tray = unsafe { Box::from_raw(handle) };
    tray.menu.show_all();

    let _ = Box::into_raw(tray);
}

#[no_mangle]
pub extern "system" fn Java_fr_yusaku_multitap_ui_core_Core_traySetStatus(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    status: jint,
) {
    let handle = handle as *mut Tray;
    let mut tray = unsafe { Box::from_raw(handle) };
    match status {
        0 => tray.indicator.set_status(AppIndicatorStatus::Passive),
        1 => tray.indicator.set_status(AppIndicatorStatus::Active),
        2 => tray.indicator.set_status(AppIndicatorStatus::Attention),
        n => error!("Invalid AppIndicatorStatus {}", n),
    }

    let _ = Box::into_raw(tray);
}

impl Callbacks {

    fn call(&self, method: &str) {
        let env = self.jvm.get_env().unwrap();
        let res = env.call_method(
            &self.gref,
            method,
            "()V",
            &[],
        );

        if let Err(err) = res {
            error!("Error calling {} {:?}", method, err);
            env.exception_describe().unwrap();
        }
    }
}
