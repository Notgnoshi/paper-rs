package io.papermc;

import org.bukkit.plugin.Plugin;

/**
 * Generic JNI dispatch surface. All Java -> Rust calls funnel through these
 * static native methods, which are linked to symbols in
 * {@code libpaper_loader.so}.
 *
 * Plugin authors should not need to touch this class; it's the contract between
 * the Java side and the Rust loader. Adding plugin functionality is a pure-Rust
 * activity (register handlers via the Rust plugin trait); this class never grows.
 */
public final class RustPlugin {

    private RustPlugin() {
    }

    /**
     * Bootstrap Rust side. {@code pluginPath} is an absolute path to the plugin
     * cdylib (e.g. {@code libdisco_plugin.so}). Throws RuntimeException on failure.
     */
    public static native void on_enable(String pluginPath, Plugin plugin);

    /** Tear down the Rust side; dlcloses the plugin .so. */
    public static native void on_disable();

    /** Invoked by RustEventExecutor when Bukkit fires a registered event. */
    public static native void dispatch_event(long handlerId, Object event);

    /** Invoked by RustCommand when Bukkit dispatches a registered command. */
    public static native boolean dispatch_command(long handlerId, Object sender, String[] args);

    /** Invoked by RustCommand for tab completion. May return null. */
    public static native Object dispatch_tab_complete(long handlerId, Object sender, String[] args);
}
