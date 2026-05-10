package io.paperrs.shim;

import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;
import java.nio.charset.StandardCharsets;
import java.util.logging.Level;
import java.util.logging.Logger;

public final class LogUpcall {

    private static final Level[] LEVELS = {
            Level.SEVERE,
            Level.WARNING,
            Level.INFO,
            Level.FINE,
            Level.FINER,
    };

    private final Logger logger;

    public LogUpcall(Logger logger) {
        this.logger = logger;
    }

    public MemorySegment asNativePointer(Arena arena) {
        try {
            // Java-reflection view of logFromNative's signature.
            MethodType javaSignature = MethodType.methodType(
                    void.class, // return
                    int.class, // level
                    MemorySegment.class, // targetPtr
                    int.class, // targetLen
                    MemorySegment.class, // msgPtr
                    int.class // msgLen
            );

            // Reflective handle to logFromNative, bound to this instance.
            MethodHandle handler = MethodHandles.lookup()
                    .findVirtual(LogUpcall.class, "logFromNative", javaSignature)
                    .bindTo(this);

            // The native ABI description of logFromNative for Panama.
            FunctionDescriptor nativeSignature = FunctionDescriptor.ofVoid(
                    ValueLayout.JAVA_INT,
                    ValueLayout.ADDRESS,
                    ValueLayout.JAVA_INT,
                    ValueLayout.ADDRESS,
                    ValueLayout.JAVA_INT);

            // Wrap the handle as a native function pointer.
            return Linker.nativeLinker().upcallStub(handler, nativeSignature, arena);
        } catch (NoSuchMethodException | IllegalAccessException e) {
            throw new RuntimeException("Failed to wire log upcall", e);
        }
    }

    public void logFromNative(int level, MemorySegment targetPtr, int targetLen, MemorySegment msgPtr, int msgLen) {
        String target = readString(targetPtr, targetLen);
        String message = readString(msgPtr, msgLen);
        int idx = Math.max(0, Math.min(level, LEVELS.length - 1));
        logger.log(LEVELS[idx], "[" + target + "] " + message);
    }

    private static String readString(MemorySegment ptr, int len) {
        if (len <= 0)
            return "";
        byte[] bytes = ptr.reinterpret(len).toArray(ValueLayout.JAVA_BYTE);
        return new String(bytes, StandardCharsets.UTF_8);
    }
}
