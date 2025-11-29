// 🚀 VoxelCraiMod - Fabric Mod Build Configuration
// Воксельное освещение на основе SH-паттернов (Spherical Harmonics)
// MIT License - 2024

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

val minecraftVersion = "1.21.3"
val yarnMappings = "1.21.3+build.2"
val loaderVersion = "0.16.7"
val fabricVersion = "0.106.1+1.21.3"

repositories {
    mavenCentral()
    maven("https://maven.fabricmc.net/")
    maven("https://maven.shedaniel.me/")
    maven("https://maven.terraformersmc.com/releases/")
    // 🌟 Iris/Sodium репозитории
    maven("https://api.modrinth.com/maven") {
        name = "Modrinth"
        content {
            includeGroup("maven.modrinth")
        }
    }
}

dependencies {
    // 🎮 Базовые зависимости Fabric
    minecraft("com.mojang:minecraft:$minecraftVersion")
    mappings("net.fabricmc:yarn:$yarnMappings:v2")
    modImplementation("net.fabricmc:fabric-loader:$loaderVersion")
    
    // 📦 Fabric API модули
    modImplementation("net.fabricmc.fabric-api:fabric-api:$fabricVersion")
    
    // 🔧 Опциональные зависимости закомментированы
    // Iris и Sodium подключаются пользователем отдельно
    // modCompileOnly("maven.modrinth:iris:1.7.3+1.21.1")
    // modCompileOnly("maven.modrinth:sodium:mc1.21.1-0.6.0-beta.2")
    
    // 🧪 Тестирование
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.0")
}

tasks {
    processResources {
        inputs.property("version", project.version)
        duplicatesStrategy = DuplicatesStrategy.EXCLUDE
        
        filesMatching("fabric.mod.json") {
            expand(
                "version" to project.version,
                "minecraft_version" to minecraftVersion,
                "loader_version" to loaderVersion
            )
        }
    }
    
    withType<JavaCompile> {
        options.encoding = "UTF-8"
        options.release.set(21)
    }
    
    java {
        withSourcesJar()
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }
    
    jar {
        from("LICENSE-MIT") {
            rename { "${it}_${base.archivesName.get()}" }
        }
        
        manifest {
            attributes(
                "Implementation-Title" to "VoxelCraiMod",
                "Implementation-Version" to project.version,
                "Implementation-Vendor" to "VoxelCrai Team"
            )
        }
    }
    
    test {
        useJUnitPlatform()
    }
}

// 📤 Публикация артефактов
publishing {
    publications {
        create<MavenPublication>("mavenJava") {
            from(components["java"])
            
            pom {
                name.set("VoxelCraiMod")
                description.set("Voxel lighting with SH patterns for Minecraft")
                url.set("https://github.com/voxelcrai/voxelcrai-mod")
                
                licenses {
                    license {
                        name.set("MIT License")
                        url.set("https://opensource.org/licenses/MIT")
                    }
                }
            }
        }
    }
    
    repositories {
        maven {
            name = "local"
            url = uri(layout.buildDirectory.dir("repo"))
        }
    }
}
