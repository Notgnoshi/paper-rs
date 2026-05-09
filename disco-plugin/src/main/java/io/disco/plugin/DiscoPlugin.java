package io.disco.plugin;

import org.bukkit.plugin.java.JavaPlugin;

public final class DiscoPlugin extends JavaPlugin {

    @Override
    public void onEnable() {
        getLogger().info("Disco PoC enabled.");
    }

    @Override
    public void onDisable() {
        getLogger().info("Disco PoC disabled.");
    }
}
