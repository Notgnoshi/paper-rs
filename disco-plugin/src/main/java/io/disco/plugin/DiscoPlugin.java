package io.disco.plugin;

import io.paperrs.shim.NativeLoader;
import org.bukkit.plugin.java.JavaPlugin;

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
    }

    @Override
    public void onDisable() {
        getLogger().info("Disco PoC disabled.");
    }
}
