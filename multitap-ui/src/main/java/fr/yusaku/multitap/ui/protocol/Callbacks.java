package fr.yusaku.multitap.ui.protocol;

import fr.yusaku.multitap.ui.Log;
import fr.yusaku.multitap.ui.MultitapUI;
import fr.yusaku.multitap.ui.core.ClientCallbacks;
import fr.yusaku.multitap.ui.core.JNICallback;
import javafx.application.Platform;

import java.util.List;
import java.util.logging.Level;
import java.util.logging.Logger;

import static java.util.stream.Collectors.toList;

public class Callbacks implements ClientCallbacks {

    private static final Logger LOG = Log.get(Callbacks.class);

    private final MultitapUI app;

    public Callbacks(MultitapUI app) {
        this.app = app;
    }

    @Override
    @JNICallback
    public void init(List<String> portsStr) {
        LOG.info("Init " + portsStr);

        var ports = portsStr.stream().map(PortRef::fromString).collect(toList());
        var event = Events.Init.of(ports);
        Platform.runLater(() -> app.onClientInit(event));
    }

    @Override
    @JNICallback
    public void onPlugged(String deviceStr, List<String> portsStr, String adapter) {
        LOG.info("Plugged " + deviceStr + " into " + portsStr + " using " + adapter);

        var device = DeviceRef.fromString(deviceStr);
        var ports = portsStr.stream().map(PortRef::fromString).collect(toList());
        var event = Events.Plugged.of(device, ports, adapter);
        Platform.runLater(() -> app.onClientPlugged(event));
    }

    @Override
    @JNICallback
    public void onUnplug(List<String> portsStr) {
        LOG.info("Unplug " + portsStr);

        var ports = portsStr.stream().map(PortRef::fromString).collect(toList());
        var event = Events.Unplug.of(ports);
        Platform.runLater(() -> app.onClientUnplug(event));
    }
}
