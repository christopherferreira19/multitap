package fr.yusaku.multitap.ui;

import javafx.geometry.Pos;
import javafx.scene.control.Label;
import javafx.scene.layout.Pane;
import javafx.scene.text.Font;
import javafx.scene.text.TextAlignment;
import fr.yusaku.multitap.ui.protocol.DeviceRef;
import fr.yusaku.multitap.ui.protocol.PortRef;

public class PortNode extends Pane {

    static Font CONTROLLERCONS = Font.loadFont(
            MultitapUI.class.getResourceAsStream("controllercons-outline.ttf"), 100
    );

    private final PortRef ref;
    private DeviceRef device;

    private final Label controller;

    public static PortNode empty(PortRef portRef) {
        return new PortNode(portRef);
    }

    private PortNode(PortRef ref) {
        this.ref = ref;
        this.device = null;
        this.controller = new Label(ref.toString());

        setMinSize(200, 200);
        setPrefSize(200, 200);
        setMaxSize(200, 200);
        getStyleClass().add("port");

        controller.setPrefSize(200, 200);
        controller.setAlignment(Pos.CENTER);
        controller.setTextAlignment(TextAlignment.CENTER);
        getChildren().add(controller);
    }

    public PortRef getRef() {
        return ref;
    }

    public void plug(DeviceRef device) {
        this.device = device;
        var icon = ControllerIcon.of(device);

        controller.setText(icon.getText());
        controller.setFont(CONTROLLERCONS);
        controller.setRotate(icon.getRotate());
    }

    public void unplug() {
        this.device = null;
    }
}
