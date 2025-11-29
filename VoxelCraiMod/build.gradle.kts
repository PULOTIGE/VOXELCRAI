// VoxelCraiMod - Fabric Mod Build Configuration
// 🚀 КОНФИГУРАЦИЯ СБОРКИ МОДА

plugins {
    id("fabric-loom") version "1.7-SNAPSHOT"
    id("maven-publish")
    java
}

version = "1.0.0"
group = "net.voxelcrai"

base {
    archivesName.set("voxelcrai-mod")
}

repositories {
    mavenCentral()
    maven("https://maven.fabricmc.net/")
    maven("https://maven.shedaniel.me/")
    maven("https://maven.terraformersmc.com/releases/")
    // Iris/Sodium
    maven("https://api.modrinth.com/maven")
}

val minecraftVersion = "1.21.3"
val yarnMappings = "1.21.3+build.2"
val loaderVersion = "0.16.9"
val fabricVersion = "0.108.0+1.21.3"

dependencies {
    minecraft("com.mojang:minecraft:$minecraftVersion")
    mappings("net.fabricmc:yarn:$yarnMappings:v2")
    modImplementation("net.fabricmc:fabric-loader:$loaderVersion")
    
    // Fabric API
    modImplementation("net.fabricmc.fabric-api:fabric-api:$fabricVersion")
    
    // Iris (optional runtime)
    modCompileOnly("maven.modrinth:iris:1.7.3+1.21.1")
    
    // Sodium (optional runtime)  
    modCompileOnly("maven.modrinth:sodium:mc1.21.1-0.6.0-beta.2")
}

java {
    sourceCompatibility = JavaVersion.VERSION_21
    targetCompatibility = JavaVersion.VERSION_21
    withSourcesJar()
}

tasks.withType<JavaCompile> {
    options.encoding = "UTF-8"
    options.release.set(21)
}

tasks.processResources {
    inputs.property("version", project.version)
    
    filesMatching("fabric.mod.json") {
        expand("version" to project.version)
    }
}

tasks.jar {
    from("LICENSE") {
        rename { "${it}_${base.archivesName.get()}" }
    }
}

loom {
    runConfigs.configureEach {
        ideConfigGenerated(true)
    }
}
