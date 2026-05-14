package io.disco.plugin;

import org.bukkit.Bukkit;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.plugin.java.JavaPlugin;

import io.papermc.NativeLoader;
import io.papermc.RustPlugin;
import io.papermc.RustTracingSubscriber;
import io.papermc.paper.event.server.ServerResourcesReloadedEvent;

public final class DiscoPlugin extends JavaPlugin {

    @Override
    public void onEnable() {
        getLogger().info("onEnable: starting Disco PoC");

        String loaderPath = System.getProperty("papermc.loader.path");
        if (loaderPath == null) {
            throw new IllegalStateException("Missing papermc.loader.path system property");
        }
        String corePath = System.getProperty("papermc.loader.plugin.path.disco");
        if (corePath == null) {
            throw new IllegalStateException("Missing papermc.loader.plugin.path.disco system property");
        }
        NativeLoader.load(loaderPath);
        getLogger().info("onEnable: papermc-loader loaded from " + loaderPath);

        RustTracingSubscriber.install(getLogger());
        getLogger().info("onEnable: calling RustPlugin.on_enable with plugin=" + corePath);
        RustPlugin.on_enable(corePath, this);

        // Hook /reload so it cycles us. ServerResourcesReloadedEvent fires
        // after Paper's /reload finishes its recipe/advancement work; we then
        // disable + re-enable to load rebuilt Rust code from disk.
        getServer().getPluginManager().registerEvents(new Listener() {
            @EventHandler
            public void onResourcesReloaded(ServerResourcesReloadedEvent event) {
                getLogger().info("ServerResourcesReloadedEvent (cause=" + event.getCause() + "): cycling Disco");
                // Defer one tick so the event-handling stack unwinds before we
                // disable ourselves. Re-enable runs in the same scheduled task.
                Bukkit.getScheduler().runTaskLater(DiscoPlugin.this, () -> {
                    Bukkit.getPluginManager().disablePlugin(DiscoPlugin.this);
                    Bukkit.getPluginManager().enablePlugin(DiscoPlugin.this);
                }, 1L);
            }
        }, this);

        getLogger().info("onEnable: complete");
    }

    @Override
    public void onDisable() {
        getLogger().info("onDisable: calling RustPlugin.on_disable");
        RustPlugin.on_disable();
        getLogger().info("onDisable: complete");
    }
}
