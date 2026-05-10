package io.paperrs.shim;

import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.nio.charset.StandardCharsets;
import java.util.logging.Level;
import java.util.logging.Logger;

/**
 * Receives log events forwarded from a paper-rs cdylib via a Panama upcall and
 * routes them to a java.util.logging.Logger.
 * Levels: 0=ERROR, 1=WARN, 2=INFO, 3=DEBUG, 4=TRACE.
 *
 * Sub-INFO Rust events (DEBUG, TRACE) are emitted at INFO so they survive
 * Paper's appender
 * filtering, with the Rust level name stamped into the target prefix instead.
 */
public final class PaperFfiLogger {

    private static final Level[] LEVELS = {
            Level.SEVERE,
            Level.WARNING,
            Level.INFO,
            Level.FINE,
            Level.FINER,
    };

    private static final String[] RUST_LEVELS = { "ERROR", "WARN", "INFO", "DEBUG", "TRACE" };

    private final Logger logger;

    public PaperFfiLogger(Logger logger) {
        this.logger = logger;
    }

    public void dispatch(int level, MemorySegment targetPtr, int targetLen, MemorySegment msgPtr, int msgLen) {
        String target = readString(targetPtr, targetLen);
        String message = readString(msgPtr, msgLen);
        int idx = Math.max(0, Math.min(level, LEVELS.length - 1));
        Level effective = idx <= 2 ? LEVELS[idx] : Level.INFO;
        logger.log(effective, "[" + target + " " + RUST_LEVELS[idx] + "] " + message);
    }

    private static String readString(MemorySegment ptr, int len) {
        if (len <= 0)
            return "";
        byte[] bytes = ptr.reinterpret(len).toArray(ValueLayout.JAVA_BYTE);
        return new String(bytes, StandardCharsets.UTF_8);
    }
}
