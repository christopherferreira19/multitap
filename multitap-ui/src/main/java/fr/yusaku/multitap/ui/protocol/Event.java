package fr.yusaku.multitap.ui.protocol;

import org.immutables.value.Value;

import java.util.List;

public interface Event {
}

@Value.Enclosing
abstract class EventsDef {

    protected EventsDef() {}

    @Value.Immutable
    public static abstract class Init implements Event {
        Init() {}
        @Value.Parameter
        public abstract List<PortRef> getPorts();
    }

    @Value.Immutable
    public static abstract class Plugged implements Event {
        Plugged() {}
        @Value.Parameter
        public abstract DeviceRef getDevice();
        @Value.Parameter
        public abstract List<PortRef> getPorts();
        @Value.Parameter
        public abstract String getAdapter();
    }

    @Value.Immutable
    public static abstract class Unplug implements Event {
        Unplug() {}
        @Value.Parameter
        public abstract List<PortRef> getPorts();
    }
}
