package io.disco.plugin;

import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;

import org.bukkit.plugin.java.JavaPlugin;

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

        MethodHandle ping = Linker.nativeLinker().downcallHandle(
                SymbolLookup.loaderLookup().find("disco_ping")
                        .orElseThrow(() -> new IllegalStateException("disco_ping symbol not found")),
                FunctionDescriptor.of(ValueLayout.JAVA_INT));
        try {
            int result = (int) ping.invoke();
            getLogger().info("disco_ping() returned: " + result);
        } catch (Throwable t) {
            throw new RuntimeException("disco_ping invocation failed", t);
        }
    }

    @Override
    public void onDisable() {
        getLogger().info("Disco PoC disabled.");
    }
}
