package fr.yusaku.multitap.ui.core;

import java.util.List;

public interface ClientCallbacks {

    @JNICallback
    void init(List<String> portsStr);

    @JNICallback
    void onPlugged(String device, List<String> ports, String adapter);

    @JNICallback
    void onUnplug(List<String> ports);
}
