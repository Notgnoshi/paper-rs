package io.papermc;

import java.util.Locale;

import org.bukkit.Bukkit;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.plugin.java.JavaPlugin;

import io.papermc.paper.event.server.ServerResourcesReloadedEvent;

/**
 * Base JavaPlugin that consumer Rust plugins point {@code main:} at in their
 * {@code plugin.yml}. Owns the JNI bootstrap, the Rust tracing-subscriber
 * install, the /reload self-cycle, and the Rust-side enable/disable calls.
 *
 * Plugin authors implementing the Rust {@code papermc::Plugin} trait do not
 * need to subclass this; the same instance serves every Rust plugin that
 * declares this class as its {@code main}.
 */
public class RustReloadablePlugin extends JavaPlugin implements Listener {

    @Override
    public void onEnable() {
        // TODO: Should we normalize - to _?
        String pluginKey = getName().toLowerCase(Locale.ROOT);
        String loaderPath = NativeLoader.locate("libpapermc_loader.so", "papermc.loader.path");
        String pluginPath = NativeLoader.locate("lib" + pluginKey + "_plugin.so",
                "papermc.loader.plugin.path." + pluginKey);

        NativeLoader.load(loaderPath);
        RustTracingSubscriber.install(getLogger());
        RustPlugin.on_enable(pluginPath, this);

        getServer().getPluginManager().registerEvents(this, this);
    }

    @Override
    public void onDisable() {
        RustPlugin.on_disable();
    }

    /**
     * Hook /reload so it cycles us. Defer one tick so the event-handling stack
     * unwinds before we disable ourselves. Re-enable runs in the same scheduled
     * task.
     */
    @EventHandler
    public void onResourcesReloaded(ServerResourcesReloadedEvent event) {
        getLogger().info("ServerResourcesReloadedEvent (cause=" + event.getCause() + "): cycling " + getName());
        Bukkit.getScheduler().runTaskLater(this, () -> {
            Bukkit.getPluginManager().disablePlugin(this);
            Bukkit.getPluginManager().enablePlugin(this);
        }, 1L);
    }
}
