package io.disco.plugin;

import org.bukkit.command.PluginCommand;
import org.bukkit.plugin.java.JavaPlugin;

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

        PaperFfiLogger.install(getLogger());

        PluginCommand hello = getCommand("hello");
        if (hello == null) {
            throw new IllegalStateException("/hello command missing from plugin.yml");
        }
        hello.setExecutor(new HelloCommand());

        discoStart(this);
    }

    private static native void discoStart(Object plugin);

    @Override
    public void onDisable() {
        getLogger().info("Disco PoC disabled.");
    }
}
