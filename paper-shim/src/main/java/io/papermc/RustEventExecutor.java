package io.papermc;

import org.bukkit.event.Event;
import org.bukkit.event.Listener;
import org.bukkit.plugin.EventExecutor;

/**
 * Generic dispatcher: implements both Listener (Bukkit's marker) and
 * EventExecutor so a single instance covers both arguments to
 * PluginManager.registerEvent.
 *
 * The handlerId is an opaque token; Rust maintains the registry of handlers and
 * looks up the right closure when dispatch is called.
 */
public final class RustEventExecutor implements EventExecutor, Listener {

    private final long handlerId;

    public RustEventExecutor(long handlerId) {
        this.handlerId = handlerId;
    }

    @Override
    public void execute(Listener listener, Event event) {
        RustPlugin.dispatch_event(handlerId, event);
    }
}
