package fr.yusaku.multitap.ui.protocol;

import org.immutables.value.Value;

@Value.Immutable(copy = false, intern = true)
abstract class DeviceRefDef {
    DeviceRefDef() {}
    @Value.Parameter
    public abstract String getName();
    @Value.Parameter
    public abstract int getIndex();
    public String toString() {
        return getName() + ":" + getIndex();
    }

    public static DeviceRef fromString(String string) {
        var split = string.split(":");
        var name = split[0];
        var index = Integer.parseInt(split[1]);
        return DeviceRef.of(name, index);
    }
}
