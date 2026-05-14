plugins {
    `java-library`
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(25))
    }
}

repositories {
    maven("https://repo.papermc.io/repository/maven-public/")
}

val mcVersion = "26.1.2"
dependencies {
    compileOnly("io.papermc.paper:paper-api:$mcVersion.build.+")
}
