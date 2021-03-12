package fr.yusaku.multitap.ui;

import fr.yusaku.multitap.ui.protocol.DeviceRef;

public enum ControllerIcon {
    NES("\uDAF0\uDC10"),
    SNES("\uDAF0\uDC17"),
    GameCube("\uDAF0\uDC03"),
    Wiimote("\uDAF0\uDC24"),
    WiiClassic("\uDAF0\uDC21"),
    WiiUPro("\uDAF0\uDC22"),
    JoyConLeftVertical("\uDAF0\uDC04"),
    JoyConRightVertical("\uDAF0\uDC05"),
    JoyConLeftHorizontal("\uDAF0\uDC04", 270),
    JoyConRightHorizontal("\uDAF0\uDC05", 90),
    JoyConCombined("\uDAF0\uDC06"),
    SwitchPro("\uDAF0\uDC19"),

    SMS("\uDAF0\uDC07"),
    Megadrive("\uDAF0\uDC08"),
    Dreamcast("\uDAF0\uDC02"),

    PSX("\uDAF0\uDC11"),
    DS2("\uDAF0\uDC12"),
    DS3("\uDAF0\uDC13"),
    DS4("\uDAF0\uDC14"),
    DS5("\uDAF0\uDC15"),

    XboxDuke("\uDAF0\uDC29"),
    XboxV2("\uDAF0\uDC26"),
    Xbox360("\uDAF0\uDC25"),
    XboxOne("\uDAF0\uDC27"),
    XboxSeries("\uDAF0\uDC28"),

    Stadia("\uDAF0\uDC18");

    private final String text;
    private final float rotate;

    ControllerIcon(String text) {
        this(text, 0);
    }

    ControllerIcon(String text, float rotate) {
        this.text = text;
        this.rotate = rotate;
    }

    public static ControllerIcon of(DeviceRef device) {
        var name = device.getName();
        name = name.replace("/", "");
        for (var icon : values()) {
            if (icon.name().equals(name)) {
                return icon;
            }
        }

        return Xbox360;
    }

    public String getText() {
        return text;
    }

    public float getRotate() {
        return rotate;
    }
}
