package io.paperrs.shim;

import java.util.logging.Level;
import java.util.logging.Logger;

/**
 * Static dispatch target invoked from Rust via JNI to deliver tracing events
 * to a java.util.logging.Logger.
 *
 * The plugin must call {@link #install(Logger)} once before Rust emits any
 * tracing events; tracing events emitted before install are dropped silently.
 *
 * Rust level mapping: 0=ERROR, 1=WARN, 2=INFO, 3=DEBUG, 4=TRACE.
 * Sub-INFO Rust events (DEBUG, TRACE) are emitted at INFO so they survive
 * Paper's appender filtering.
 */
public final class PaperFfiLogger {

    private static final Level[] JAVA_LEVELS = {
            Level.SEVERE,
            Level.WARNING,
            Level.INFO,
            Level.FINE,
            Level.FINER,
    };

    private static final String[] RUST_LEVELS = { "ERROR", "WARN", "INFO", "DEBUG", "TRACE" };

    private static volatile Logger logger;

    private PaperFfiLogger() {
    }

    public static void install(Logger l) {
        logger = l;
    }

    public static void dispatch(int level, String target, String message) {
        Logger l = logger;
        if (l == null) {
            return;
        }
        int idx = Math.max(0, Math.min(level, JAVA_LEVELS.length - 1));
        Level effective = idx <= 2 ? JAVA_LEVELS[idx] : Level.INFO;
        l.log(effective, "[" + RUST_LEVELS[idx] + ": " + target + "] " + message);
    }
}
