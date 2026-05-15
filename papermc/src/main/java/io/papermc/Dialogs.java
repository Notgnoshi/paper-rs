package io.papermc;

import io.papermc.paper.dialog.Dialog;
import io.papermc.paper.registry.data.dialog.DialogBase;
import io.papermc.paper.registry.data.dialog.type.DialogType;

/**
 * Bridges Paper's lambda-based dialog construction surface to a direct (base,
 * type) signature
 * callable from Rust without needing to construct a Java Consumer.
 */
public final class Dialogs {
    private Dialogs() {
    }

    public static Dialog create(DialogBase base, DialogType type) {
        return Dialog.create(b -> b.empty().base(base).type(type));
    }
}
