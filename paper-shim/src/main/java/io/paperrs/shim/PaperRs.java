package io.paperrs.shim;

import org.bukkit.plugin.Plugin;

/**
 * Generic JNI dispatch surface. All Java -> Rust calls funnel through these
 * static native methods, which are linked to symbols in
 * {@code libpaper_loader.so}.
 *
 * Plugin authors should not need to touch this class; it's the contract between
 * paper-shim (Java) and paper-loader (Rust). Adding plugin functionality is a
 * pure-Rust activity (register handlers via paper-rs); this class never grows.
 */
public final class PaperRs {

    private PaperRs() {
    }

    /**
     * Bootstrap Rust side. {@code corePath} is an absolute path to the core
     * cdylib (e.g. {@code libdisco_core.so}). Throws RuntimeException on failure.
     */
    public static native void init(String corePath, Plugin plugin);

    /** Tear down the Rust side. Stage 3 will make this dlclose the core .so. */
    public static native void shutdown();

    /** Invoked by RustEventExecutor when Bukkit fires a registered event. */
    public static native void dispatchEvent(long handlerId, Object event);

    /** Invoked by RustCommand when Bukkit dispatches a registered command. */
    public static native boolean dispatchCommand(long handlerId, Object sender, String[] args);

    /** Invoked by RustCommand for tab completion. May return null. */
    public static native Object dispatchTabComplete(long handlerId, Object sender, String[] args);
}
