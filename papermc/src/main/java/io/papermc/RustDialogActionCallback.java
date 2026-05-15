package io.papermc;

import java.lang.ref.Cleaner;

import io.papermc.paper.dialog.DialogResponseView;
import io.papermc.paper.registry.data.dialog.action.DialogActionCallback;
import net.kyori.adventure.audience.Audience;

/**
 * Adapts a Rust-side closure (identified by a long id) to Paper's
 * {@link DialogActionCallback} functional interface.
 *
 * Each instance registers a {@link Cleaner} action that calls
 * {@link #bridgeDrop} on GC, so the Rust closure is released when the Java
 * bridge becomes unreachable. The cleaner lambda must NOT capture {@code this};
 * capturing the primitive id is sufficient and keeps the instance eligible for
 * GC.
 */
public final class RustDialogActionCallback implements DialogActionCallback {
    private static final Cleaner CLEANER = Cleaner.create();

    private final long id;

    public RustDialogActionCallback(long id) {
        this.id = id;
        final long capturedId = id;
        CLEANER.register(this, () -> bridgeDrop(capturedId));
    }

    @Override
    public void accept(DialogResponseView response, Audience audience) {
        bridgeDispatch(id, response, audience);
    }

    private static native void bridgeDispatch(long id, Object response, Object audience);

    private static native void bridgeDrop(long id);
}
