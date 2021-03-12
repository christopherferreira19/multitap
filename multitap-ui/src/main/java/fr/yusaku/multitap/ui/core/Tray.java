package fr.yusaku.multitap.ui.core;

import javafx.application.Platform;

public final class Tray {

    public enum Status {
        Passive(0),
        Active(1),
        Attention(2);

        private final int nativeValue;

        Status(int nativeValue) {
            this.nativeValue = nativeValue;
        }
    }

    private final long handle;
    private Status status;

    public Tray(TrayCallbacks callbacks) {
        this.handle = Core.trayInit(callbacks);
        Core.registerHandle(this, this.handle);
        setStatus(Status.Active);
    }

    public void show() {
        Core.trayShow(handle);
    }

    public void setStatus(Status status) {
        this.status = status;
        Core.traySetStatus(handle, status.nativeValue);
    }

}
