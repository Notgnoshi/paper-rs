import java.io.File

plugins {
    `java-library`
    // https://plugins.gradle.org/plugin/xyz.jpenilla.run-paper
    id("xyz.jpenilla.run-paper") version "3.0.2"
    // https://plugins.gradle.org/plugin/com.gradleup.shadow
    id("com.gradleup.shadow") version "9.4.1"
}

repositories {
    maven("https://repo.papermc.io/repository/maven-public/")
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(25))
    }
}

// https://papermc.io/downloads/paper
val mcVersion = "26.1.2"
dependencies {
    implementation(project(":papermc"))
    compileOnly("io.papermc.paper:paper-api:$mcVersion.build.+")
}

// Paths to the two Rust cdylibs.
val loaderLibPath = rootProject.layout.projectDirectory
    .file("target/release/libpapermc_loader.so").asFile.absolutePath
val pluginLibPath = rootProject.layout.projectDirectory
    .file("target/release/libdisco_plugin.so").asFile.absolutePath

// Bundle the two cdylibs into the plugin jar as resources under `native/`. papermc's NativeLoader
// extracts them from there at runtime when no `papermc.loader.path` /
// `papermc.loader.plugin.path.disco` system property is set (i.e. for production drop-in
// distributions).
tasks.processResources {
    from(loaderLibPath) { into("native") }
    from(pluginLibPath) { into("native") }
}

tasks.runServer {
    minecraftVersion(mcVersion)
    runDirectory.set(rootProject.layout.projectDirectory.dir("run"))
    // Dev workflow: point papermc at the cargo-built .so directly so a `cargo build --release`
    // followed by `/reload` picks up the new bytes without going through jar repackaging.
    systemProperty("papermc.loader.path", loaderLibPath)
    systemProperty("papermc.loader.plugin.path.disco", pluginLibPath)
    environment("RUST_LOG", System.getenv("RUST_LOG") ?: "DEBUG")
    // Auto-accept Mojang's EULA for the dev server (https://www.minecraft.net/en-us/eula).
    doFirst {
        runDirectory.get().asFile.mkdirs()
        runDirectory.get().file("eula.txt").asFile.writeText("eula=true\n")
    }
}
