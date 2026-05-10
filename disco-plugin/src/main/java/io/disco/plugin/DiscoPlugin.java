package io.disco.plugin;

import org.bukkit.Bukkit;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.plugin.java.JavaPlugin;

import io.papermc.paper.event.server.ServerResourcesReloadedEvent;
import io.paperrs.shim.NativeLoader;
import io.paperrs.shim.PaperFfiLogger;
import io.paperrs.shim.PaperRs;

public final class DiscoPlugin extends JavaPlugin {

    @Override
    public void onEnable() {
        getLogger().info("onEnable: starting Disco PoC");

        String loaderPath = System.getProperty("paper.loader.path");
        if (loaderPath == null) {
            throw new IllegalStateException("Missing paper.loader.path system property");
        }
        String corePath = System.getProperty("disco.core.path");
        if (corePath == null) {
            throw new IllegalStateException("Missing disco.core.path system property");
        }
        NativeLoader.load(loaderPath);
        getLogger().info("onEnable: paper-loader loaded from " + loaderPath);

        PaperFfiLogger.install(getLogger());
        getLogger().info("onEnable: calling PaperRs.init with core=" + corePath);
        PaperRs.init(corePath, this);

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
        getLogger().info("onDisable: calling PaperRs.shutdown");
        PaperRs.shutdown();
        getLogger().info("onDisable: complete");
    }
}
