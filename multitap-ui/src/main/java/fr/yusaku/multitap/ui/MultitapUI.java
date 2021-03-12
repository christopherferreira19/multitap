package fr.yusaku.multitap.ui;

import fr.yusaku.multitap.ui.core.Client;
import fr.yusaku.multitap.ui.core.JNICallback;
import fr.yusaku.multitap.ui.core.Tray;
import fr.yusaku.multitap.ui.core.TrayCallbacks;
import fr.yusaku.multitap.ui.protocol.*;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.geometry.Orientation;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.Separator;
import javafx.scene.layout.HBox;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;

public class MultitapUI extends Application implements TrayCallbacks {

    private Stage stage;
    private VBox root;
    private HBox ports;

    public void start(Stage stage) {
        Platform.setImplicitExit(false);
        Scene scene = createScene();
        Tray tray = createTray(stage);

        Client.monitor(new Callbacks(this));

        this.stage = stage;
        stage.setScene(scene);
        tray.show();
    }

    private Tray createTray(Stage stage) {
        var tray = new Tray(this);
        stage.showingProperty().addListener((o, old, showing) -> {
            tray.setStatus(showing ? Tray.Status.Attention : Tray.Status.Active);
        });
        return tray;
    }

    private Scene createScene() {
        this.ports = new HBox();
        ports.setAlignment(Pos.CENTER);
        ports.setSpacing(20);

        var label = new Label("Pick your controller");
        var separator = new Separator(Orientation.HORIZONTAL);
        var button = new Button("Reset");

        this.root = new VBox(30, ports, label, separator, button);
        root.setAlignment(Pos.CENTER);
        Scene scene = new Scene(root, 1280, 720);
        scene.getStylesheets().add(getClass().getResource("styles.css").toExternalForm());
        return scene;
    }

    @Override
    @JNICallback
    public void onTrayMenuOpen() {
        stage.show();
    }

    @Override
    @JNICallback
    public void onTrayMenuQuit() {
        Platform.exit();
    }

    public void onClientInit(Events.Init event) {
        for (PortRef port : event.getPorts()) {
            this.ports.getChildren().add(PortNode.empty(port));
        }
    }

    public void onClientPlugged(Events.Plugged event) {
        for (var portNode : ports.getChildren()) {
            var port = (PortNode) portNode;
            if (event.getPorts().contains(port.getRef())) {
                port.plug(event.getDevice());
            }
        }
    }

    public void onClientUnplug(Events.Unplug event) {
        for (var portNode : ports.getChildren()) {
            var port = (PortNode) portNode;
            if (event.getPorts().contains(port.getRef())) {
                port.unplug();
            }
        }
    }

    public static void main(String[] args) {
        launch(args);
    }
}
