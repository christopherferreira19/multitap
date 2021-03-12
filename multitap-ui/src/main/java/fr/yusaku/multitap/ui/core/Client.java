package fr.yusaku.multitap.ui.core;

public final class Client {

    public static void reset() {
        Core.clientReset();
    }

    public static void monitor(ClientCallbacks callbacks) {
        var thread = new Thread(() -> Core.clientMonitor(callbacks));
        thread.setDaemon(true);
        thread.start();
    }
}

