package io.papermc;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;

public final class NativeLoader {

    private NativeLoader() {
    }

    /**
     * Locate a native library and stage it at a per-plugin temp path, returning
     * that path.
     *
     * Source resolution order:
     * 1. The JVM system property {@code systemPropertyKey}, if set: its value
     * is treated as the path to the source .so on disk. Preferred for dev
     * workflows where {@code cargo build} produces the .so directly. The
     * source is re-copied to the temp path on every call so {@code /reload}
     * picks up rebuilt bytes.
     * 2. Otherwise, a jar resource at {@code native/<libName>} loaded via
     * {@code resourceClassLoader}. Used when the plugin is distributed as
     * a self-contained jar that bundles its native libraries. The resource
     * is extracted only when the temp file does not yet exist; subsequent
     * calls within the same process reuse the previously-extracted copy.
     * (Paper closes the plugin's URLClassLoader during {@code disablePlugin},
     * so {@code getResourceAsStream} can return null on a subsequent
     * {@code onEnable}. Reusing the staged file avoids that path entirely.)
     *
     * The per-plugin temp directory satisfies the JVM's
     * "one-native-library-per-ClassLoader" rule when multiple papermc plugins
     * are present on a server: each plugin's copy lives at a path unique to
     * that plugin, so {@code System.load} won't conflict across them.
     */
    public static String locate(
            String libName,
            String systemPropertyKey,
            String pluginName,
            ClassLoader resourceClassLoader) {
        Path target = Path.of(
                System.getProperty("java.io.tmpdir"), "papermc", pluginName, libName);
        try {
            Files.createDirectories(target.getParent());
            String propValue = System.getProperty(systemPropertyKey);
            if (propValue != null) {
                Path source = Path.of(propValue);
                if (!Files.exists(source)) {
                    throw new IllegalStateException(
                            systemPropertyKey + " points at " + source + " which does not exist");
                }
                Files.copy(source, target, StandardCopyOption.REPLACE_EXISTING);
            } else if (!Files.exists(target)) {
                String resourcePath = "native/" + libName;
                try (InputStream in = resourceClassLoader.getResourceAsStream(resourcePath)) {
                    if (in == null) {
                        throw new IllegalStateException("Native library " + libName + " not available via "
                                + systemPropertyKey + " or jar resource '" + resourcePath + "'");
                    }
                    Files.copy(in, target);
                }
            }
        } catch (IOException e) {
            throw new IllegalStateException("Failed to stage native library " + libName + " at " + target, e);
        }
        return target.toAbsolutePath().toString();
    }

    public static void load(String path) {
        Path resolved = Path.of(path).toAbsolutePath();
        if (!Files.exists(resolved)) {
            throw new IllegalStateException("Native library not found at: " + resolved);
        }
        System.load(resolved.toString());
    }
}
