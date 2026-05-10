package io.disco.plugin;

import org.bukkit.plugin.java.JavaPlugin;

import io.paperrs.shim.NativeLoader;
import io.paperrs.shim.PaperFfiLogger;
import io.paperrs.shim.PaperRs;

public final class DiscoPlugin extends JavaPlugin {

    @Override
    public void onEnable() {
        getLogger().info("Disco PoC enabled.");

        String loaderPath = System.getProperty("paper.loader.path");
        if (loaderPath == null) {
            throw new IllegalStateException("Missing paper.loader.path system property");
        }
        String corePath = System.getProperty("disco.core.path");
        if (corePath == null) {
            throw new IllegalStateException("Missing disco.core.path system property");
        }
        NativeLoader.load(loaderPath);
        getLogger().info("Loaded paper-loader: " + loaderPath);

        PaperFfiLogger.install(getLogger());
        PaperRs.init(corePath, this);
    }

    @Override
    public void onDisable() {
        PaperRs.shutdown();
        getLogger().info("Disco PoC disabled.");
    }
}
