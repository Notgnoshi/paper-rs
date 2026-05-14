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

// Paths to the two Rust cdylibs the dev server loads.
val loaderLib: String = (project.findProperty("loader-lib") as String?)
    ?: rootProject.layout.projectDirectory
        .file("target/release/libpapermc_loader.so").asFile.absolutePath
val pluginLib: String = (project.findProperty("plugin-lib") as String?)
    ?: rootProject.layout.projectDirectory
        .file("target/release/libdisco_plugin.so").asFile.absolutePath

tasks.runServer {
    minecraftVersion(mcVersion)
    runDirectory.set(rootProject.layout.projectDirectory.dir("run"))
    systemProperty("papermc.loader.path", loaderLib)
    systemProperty("papermc.loader.plugin.path.disco", pluginLib)
    environment("RUST_LOG", System.getenv("RUST_LOG") ?: "DEBUG")
    // Auto-accept Mojang's EULA for the dev server (https://www.minecraft.net/en-us/eula).
    doFirst {
        runDirectory.get().asFile.mkdirs()
        runDirectory.get().file("eula.txt").asFile.writeText("eula=true\n")
    }
}
