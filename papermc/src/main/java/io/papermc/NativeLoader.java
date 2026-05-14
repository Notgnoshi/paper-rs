package io.papermc;

import java.nio.file.Files;
import java.nio.file.Path;

public final class NativeLoader {

    private NativeLoader() {
    }

    /**
     * Locate the absolute path of a native library by reading
     * {@code systemPropertyKey} from the JVM system properties. Throws if the
     * property is unset.
     *
     * {@code libName} is presently unused; it identifies the library by its
     * canonical filename and exists so future jar-resource extraction can find
     * the bundled artifact.
     */
    public static String locate(String libName, String systemPropertyKey) {
        String value = System.getProperty(systemPropertyKey);
        if (value == null) {
            throw new IllegalStateException(
                    "Missing " + systemPropertyKey + " system property (for " + libName + ")");
        }
        return value;
    }

    public static void load(String path) {
        Path resolved = Path.of(path).toAbsolutePath();
        if (!Files.exists(resolved)) {
            throw new IllegalStateException("Native library not found at: " + resolved);
        }
        System.load(resolved.toString());
    }
}
