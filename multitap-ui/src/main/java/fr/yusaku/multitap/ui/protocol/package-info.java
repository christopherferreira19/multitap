@Value.Style(
        typeAbstract = "*Def",
        typeImmutable = "*",
        typeImmutableEnclosing = "*",
        visibility = Value.Style.ImplementationVisibility.PUBLIC,
        builtinContainerAttributes = false,
        defaults = @Value.Immutable(builder = false)
)
package fr.yusaku.multitap.ui.protocol;

import org.immutables.value.Value;