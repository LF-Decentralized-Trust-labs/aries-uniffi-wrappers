import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.plugin.KotlinSourceSetTree
import java.util.*
import gobley.gradle.cargo.dsl.jvm
import gobley.gradle.GobleyHost
import gobley.gradle.cargo.dsl.linux
import gobley.gradle.Variant
import gobley.gradle.cargo.dsl.android
import gobley.gradle.cargo.dsl.appleMobile
import gobley.gradle.rust.targets.RustPosixTarget
import gobley.gradle.rust.targets.RustTarget
import gobley.gradle.rust.targets.RustWindowsTarget

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

    // Use cross when building Linux on Mac
    if(GobleyHost.Platform.MacOS.isCurrent){
        val home = System.getProperty("user.home")
        val crossFile = File("$home/.cargo/bin/cross")
        builds{
            linux{
                this@linux.debug.buildTaskProvider.configure{
                    this@configure.cargo = crossFile
                }
                this@linux.release.buildTaskProvider.configure{
                    this@configure.cargo = crossFile
                }
            }
        }
    }
builds{
    appleMobile{
        release.buildTaskProvider.configure{
            additionalEnvironment.put("IPHONEOS_DEPLOYMENT_TARGET", "10")
        }
    }
}
    builds.jvm{
        embedRustLibrary = true
        if(GobleyHost.Platform.MacOS.isCurrent && rustTarget == RustPosixTarget.MinGWX64){
            release.dynamicLibraries.set(listOf("anoncreds_uniffi.dll"))
            debug.dynamicLibraries.set(listOf("anoncreds_uniffi.dll"))
        }
    }
}

uniffi{
    generateFromLibrary{
        packageName = "anoncreds_uniffi"
        cdylibName = "anoncreds_uniffi"
        this@generateFromLibrary.disableJavaCleaner = true
    }
}

// Stub secrets to let the project sync and build without the publication values set up
ext["githubUsername"] = null
ext["githubToken"] = null
ext["anoncredsVersion"] = "0.2.0"
ext["wrapperVersion"] = "GOBLEY"

val secretPropsFile = project.rootProject.file("local.properties")
if(secretPropsFile.exists()) {
    secretPropsFile.reader().use {
        Properties().apply {
            load(it)
        }
    }.onEach{ (name, value) ->
        ext[name.toString()] = value
    }
} else {
    ext["githubUsername"] = System.getenv("GITHUB_ACTOR")
    ext["githubToken"] = System.getenv("GITHUB_TOKEN")
}

fun getExtraString(name: String) = ext[name]?.toString()

group = "org.hyperledger"
version = "${getExtraString("anoncredsVersion")}-wrapper.${getExtraString("wrapperVersion")}"

publishing{
    repositories{
        maven{
            name = "github"
            setUrl("https://maven.pkg.github.com/hyperledger/aries-uniffi-wrappers")
            credentials {
                username = getExtraString("githubUsername")
                password = getExtraString("githubToken")
            }
        }
    }

    publications.withType<MavenPublication> {
        pom {
            name.set("Anoncreds Uniffi Kotlin")
            description.set("Kotlin MPP wrapper around anoncreds uniffi")
            url.set("https://github.com/hyperledger/aries-uniffi-wrappers")

            scm{
                url.set("https://github.com/hyperledger/aries-uniffi-wrappers")
            }
        }
    }
}


kotlin {
    jvmToolchain(17)
    applyDefaultHierarchyTemplate()

    androidTarget{
        publishLibraryVariants("release")
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_11)
        }
        instrumentedTestVariant.sourceSetTree.set(KotlinSourceSetTree.test)
        unitTestVariant.sourceSetTree.set(KotlinSourceSetTree.unitTest)
    }

    jvm{
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_17)
            freeCompilerArgs.add("-Xdebug")
        }

        testRuns["test"].executionTask.configure{
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
                implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.8.1")
            }
        }

        val commonTest by getting {
            dependencies{
                implementation(kotlin("test"))
                implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.10.2")
            }
        }

        val androidMain by getting {
//            kotlin.srcDir(uniffiBindings.resolve("jvmMain").resolve("kotlin"))
//            dependencies{
//                implementation("net.java.dev.jna:jna:5.7.0@aar")
//                implementation("org.jetbrains.kotlinx:atomicfu:0.22.0")
//            }
        }

        val jvmMain by getting {
//            kotlin.srcDir(uniffiBindings.resolve("jvmMain").resolve("kotlin"))
//            dependencies{
//                implementation("net.java.dev.jna:jna:5.17.0")
//            }
        }

        val nativeMain by getting {
//            kotlin.srcDir(uniffiBindings.resolve("nativeMain").resolve("kotlin"))
        }

        all {
            languageSettings.optIn("kotlin.RequiresOptIn")
            languageSettings.optIn("kotlinx.cinterop.ExperimentalForeignApi")
        }
    }
}


android{
    namespace = "anoncreds_uniffi"
    compileSdk = 35
    ndkVersion = "26.1.10909125"

    defaultConfig{
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