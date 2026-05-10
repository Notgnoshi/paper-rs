package io.disco.plugin;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;

import org.bukkit.command.PluginCommand;
import org.bukkit.plugin.java.JavaPlugin;

import io.disco.ffi.DiscoFfi;
import io.disco.ffi.LoggerFnPtr;
import io.paperrs.shim.NativeLoader;
import io.paperrs.shim.PaperFfiLogger;

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

        // Native function pointer Rust will call to deliver tracing events.
        PaperFfiLogger logger = new PaperFfiLogger(getLogger());
        MemorySegment logger_ptr = LoggerFnPtr.allocate(logger::dispatch, Arena.global());

        DiscoFfi.disco_init(logger_ptr);
        int result = DiscoFfi.disco_ping();
        getLogger().info("disco_ping() returned: " + result);

        PluginCommand hello = getCommand("hello");
        if (hello == null) {
            throw new IllegalStateException("/hello command missing from plugin.yml");
        }
        hello.setExecutor(new HelloCommand());

        getServer().getPluginManager().registerEvents(new SheepListener(), this);
    }

    @Override
    public void onDisable() {
        getLogger().info("Disco PoC disabled.");
    }
}
