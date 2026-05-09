plugins {
    `java-library`
    // https://plugins.gradle.org/plugin/xyz.jpenilla.run-paper
    id("xyz.jpenilla.run-paper") version "3.0.2"
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
    compileOnly("io.papermc.paper:paper-api:$mcVersion.build.+")
}

tasks.runServer {
    minecraftVersion(mcVersion)
    runDirectory.set(rootProject.layout.projectDirectory.dir("run"))
    // Auto-accept Mojang's EULA for the dev server (https://www.minecraft.net/en-us/eula).
    doFirst {
        runDirectory.get().asFile.mkdirs()
        runDirectory.get().file("eula.txt").asFile.writeText("eula=true\n")
    }
}
