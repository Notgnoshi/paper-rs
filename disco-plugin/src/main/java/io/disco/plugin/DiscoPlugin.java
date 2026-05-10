package io.disco.plugin;

import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;

import org.bukkit.plugin.java.JavaPlugin;

import io.paperrs.shim.LogUpcall;
import io.paperrs.shim.NativeLoader;

public final class DiscoPlugin extends JavaPlugin {

    @Override
    public void onEnable() {
        getLogger().info("Disco PoC enabled.");

        String path = System.getProperty("disco.native-lib");
        if (path == null) {
            throw new IllegalStateException("Missing disco.native-lib system property");
        }
        NativeLoader.load(path);
        getLogger().info("Loaded native lib: " + path);

        SymbolLookup lookup = SymbolLookup.loaderLookup();

        // Native function pointer Rust will call to deliver tracing events.
        MemorySegment log_handler = new LogUpcall(getLogger()).asNativePointer(Arena.global());

        // Downcall handles for the Rust functions we invoke.
        MethodHandle init = downcall(lookup, "disco_init", FunctionDescriptor.ofVoid(ValueLayout.ADDRESS));
        MethodHandle ping = downcall(lookup, "disco_ping", FunctionDescriptor.of(ValueLayout.JAVA_INT));

        try {
            init.invoke(log_handler);
            int result = (int) ping.invoke();
            getLogger().info("disco_ping() returned: " + result);
        } catch (Throwable t) {
            throw new RuntimeException("native call failed", t);
        }
    }

    @Override
    public void onDisable() {
        getLogger().info("Disco PoC disabled.");
    }

    /** Resolve a native symbol and bind it as a callable handle. */
    private static MethodHandle downcall(SymbolLookup lookup, String name, FunctionDescriptor desc) {
        MemorySegment addr = lookup.find(name)
                .orElseThrow(() -> new IllegalStateException(name + " symbol not found"));
        return Linker.nativeLinker().downcallHandle(addr, desc);
    }
}
