package fr.yusaku.multitap.ui.core;

import java.lang.ref.Cleaner;

class Core {

    static {
        CoreLibFeature.loadLibrary();
        Core.init();
    }

    private static final Cleaner CLEANER = Cleaner.create();

    private static class HandleCleaner implements Runnable {

        private final long handle;

        HandleCleaner(long handle) {
            this.handle = handle;
        }

        @Override
        public void run() {
            trayDrop(handle);
        }
    }

    static void registerHandle(Object object, long handle) {
        CLEANER.register(object, new HandleCleaner(handle));
    }

    static native void init();

    static native void clientReset();

    static native void clientMonitor(ClientCallbacks callbacks);

    static native long trayInit(TrayCallbacks callbacks);

    static native void trayDrop(long trayHandle);

    static native void trayShow(long trayHandle);

    static native void traySetStatus(long trayHandle, int status);
}
