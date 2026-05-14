package io.papermc;

import java.nio.file.Files;
import java.nio.file.Path;

public final class NativeLoader {

    private NativeLoader() {
    }

    public static void load(String path) {
        Path resolved = Path.of(path).toAbsolutePath();
        if (!Files.exists(resolved)) {
            throw new IllegalStateException("Native library not found at: " + resolved);
        }
        System.load(resolved.toString());
    }
}
