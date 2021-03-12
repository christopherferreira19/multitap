package fr.yusaku.multitap.ui.core;

public interface TrayCallbacks {

    @JNICallback
    void onTrayMenuOpen();

    @JNICallback
    void onTrayMenuQuit();
}
