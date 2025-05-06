import gobley.gradle.GobleyHost
import gobley.gradle.Variant
import gobley.gradle.cargo.dsl.jvm
import gobley.gradle.cargo.dsl.linux
import gobley.gradle.rust.targets.RustPosixTarget
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.plugin.KotlinSourceSetTree
import java.util.Properties

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.kotlinSerialization)
    alias(libs.plugins.androidLibrary)

    id("dev.gobley.cargo") version "0.2.0"
    id("dev.gobley.uniffi") version "0.2.0"
    id("dev.gobley.rust") version "0.2.0"
    kotlin("plugin.atomicfu") version libs.versions.kotlin
    id("maven-publish")
}

cargo {
    packageDirectory = layout.projectDirectory.dir("rust")

    jvmVariant = Variant.Release
    nativeVariant = Variant.Release

    // Use cross when building Linux
    val home = System.getProperty("user.home")
    val crossFile = File("$home/.cargo/bin/cross")
    builds {
        linux {
            variants {
                buildTaskProvider.configure {
                    cargo = crossFile
                }
            }
        }
    }

    builds.jvm {
        embedRustLibrary = true
        if (GobleyHost.Platform.MacOS.isCurrent) {
            // Don't build for linux or windows on MacOS (mainly for github actions purposes)
            val exclude = listOf(
                RustPosixTarget.MinGWX64,
                RustPosixTarget.LinuxArm64,
                RustPosixTarget.LinuxX64
            )
            embedRustLibrary = !exclude.contains(rustTarget)
        }
        if (rustTarget == RustPosixTarget.MinGWX64) {
            variants {
                dynamicLibraries.set(listOf("askar_uniffi.dll"))
            }
        }
    }
}

uniffi {
    generateFromLibrary {
        packageName = "askar_uniffi"
        cdylibName = "askar_uniffi"
        this@generateFromLibrary.disableJavaCleaner = true
    }
}

// Stub secrets to let the project sync and build without the publication values set up
ext["githubUsername"] = null
ext["githubToken"] = null

val secretPropsFile = project.rootProject.file("local.properties")
if (secretPropsFile.exists()) {
    secretPropsFile.reader().use {
        Properties().apply {
            load(it)
        }
    }.onEach { (name, value) ->
        ext[name.toString()] = value
    }
} else {
    ext["githubUsername"] = System.getenv("GITHUB_ACTOR")
    ext["githubToken"] = System.getenv("GITHUB_TOKEN")
}

fun getExtraString(name: String) = ext[name]?.toString()

publishing {
    repositories {
        maven {
            name = "github"
            setUrl("https://maven.pkg.github.com/LF-Decentralized-Trust-labs/aries-uniffi-wrappers")
            credentials {
                username = getExtraString("githubUsername")
                password = getExtraString("githubToken")
            }
        }
    }

    publications.withType<MavenPublication> {
        if (this@withType.name == "jvm") {
            listOf(
                "win32-x86-64",
                "linux-x86-64",
                "linux-aarch64",
                "darwin-aarch64",
                "darwin-x86-64"
            ).forEach { target ->
                val file = file("build/libs/${project.name}-$version-$target.jar")
                if (file.exists()) {
                    artifact(file) {
                        classifier = target
                    }
                }
            }
        }
        pom {
            name.set("Askar Uniffi Kotlin")
            description.set("Kotlin MPP wrapper around aries askar uniffi")
            url.set("https://github.com/LF-Decentralized-Trust-labs/aries-uniffi-wrappers")

            scm {
                url.set("https://github.com/LF-Decentralized-Trust-labs/aries-uniffi-wrappers")
            }
        }
    }
}

kotlin {
    jvmToolchain(17)
    applyDefaultHierarchyTemplate()

    androidTarget {
        publishLibraryVariants("release")
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_11)
        }
        instrumentedTestVariant.sourceSetTree.set(KotlinSourceSetTree.test)
        unitTestVariant.sourceSetTree.set(KotlinSourceSetTree.unitTest)
    }

    jvm {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
            freeCompilerArgs.add("-Xdebug")
        }

        testRuns["test"].executionTask.configure {
            useJUnitPlatform()
        }
    }

    macosX64()

    macosArm64()

    iosX64()

    iosSimulatorArm64()

    iosArm64()

    sourceSets {
        val commonMain by getting {
            dependencies {
                implementation(libs.kotlinx.serialization.json)
            }
        }

        val commonTest by getting {
            dependencies {
                implementation(kotlin("test"))
                implementation(libs.kotlinx.coroutines.core)
            }
        }

        val androidMain by getting {

        }

        val jvmMain by getting {

        }

        val nativeMain by getting {
        }

        all {
            languageSettings.optIn("kotlin.RequiresOptIn")
            languageSettings.optIn("kotlinx.cinterop.ExperimentalForeignApi")
        }
    }
}


android {
    sourceSets["androidTest"].manifest.srcFile("src/androidTest/AndroidManifest.xml")
    namespace = "askar_uniffi"
    compileSdk = 35
    ndkVersion = "26.1.10909125"

    defaultConfig {
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        minSdk = 24

        testOptions {
            execution = "ANDROIDX_TEST_ORCHESTRATOR"
        }

    }

    dependencies {
        androidTestImplementation("androidx.test:rules:1.5.0")
        androidTestImplementation("androidx.test:runner:1.5.0")
        androidTestUtil("androidx.test:orchestrator:1.4.2")
    }
}