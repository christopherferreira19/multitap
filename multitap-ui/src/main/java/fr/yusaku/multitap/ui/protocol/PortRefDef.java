package fr.yusaku.multitap.ui.protocol;

import org.immutables.value.Value;

@Value.Immutable(copy = false, intern = true)
abstract class PortRefDef {
    PortRefDef() {}
    @Value.Parameter
    public abstract String getName();
    @Value.Parameter
    public abstract int getIndex();
    public String toString() {
        return getName() + ":" + getIndex();
    }

    public static PortRef fromString(String string) {
        var split = string.split(":");
        var name = split[0];
        var index = Integer.parseInt(split[1]);
        return PortRef.of(name, index);
    }
}
