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

    val home = System.getProperty("user.home")


    builds{
        linux{
            val crossFile = File("$home/.cargo/bin/cross")
            variants{
                buildTaskProvider.configure {
                    cargo = crossFile
                }
            }
        }
        appleMobile{
            release.buildTaskProvider.configure{
                additionalEnvironment.put("IPHONEOS_DEPLOYMENT_TARGET", "10.0")
            }
        }
        android {
            if(GobleyHost.Platform.Windows.isCurrent){
                val crossFile = File("$home/.cargo/bin/cross.exe")
                variants{
                    buildTaskProvider.configure {
                        cargo = crossFile
                    }
                }
            }
            dynamicLibraries.addAll("c++_shared")
        }
        jvm{
            embedRustLibrary = true
            if (GobleyHost.Platform.MacOS.isCurrent){
                // Don't build for MinGWX64 on MacOS
                val exclude = listOf(
                    RustPosixTarget.MinGWX64,
                    RustPosixTarget.LinuxArm64,
                    RustPosixTarget.LinuxX64
                )
                embedRustLibrary = !exclude.contains(rustTarget)
            }
            if(GobleyHost.Platform.Windows.isCurrent){
                variants {
                    buildTaskProvider.configure {
                        dynamicLibraries.set(listOf("indy_vdr_uniffi.dll"))
                    }
                }
            }
        }
    }
}

uniffi{
    // See https://github.com/gobley/gobley/discussions/105
    bindgenFromGitBranch(
        repository = "https://github.com/paxbun/gobley",
        branch = "tmp/uniffi-0.28.3-coff",
    )
    generateFromLibrary{
        packageName = "indy_vdr_uniffi"
        cdylibName = "indy_vdr_uniffi"
        if (GobleyHost.Platform.Windows.isCurrent) {
            build = RustWindowsTarget.X64
            variant = Variant.Release
        }
        this@generateFromLibrary.disableJavaCleaner = true
    }
}

// Stub secrets to let the project sync and build without the publication values set up
ext["githubUsername"] = null
ext["githubToken"] = null

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

publishing{
    repositories{
        maven{
            name = "github"
            setUrl("https://maven.pkg.github.com/indicio-tech/aries-uniffi-wrappers")
            credentials {
                username = getExtraString("githubUsername")
                password = getExtraString("githubToken")
            }
        }
    }

    publications.withType<MavenPublication> {
        // Add artifacts from windows/linux builds to JVM target
        if(this@withType.name == "jvm"){
            listOf("win32-x86-64","linux-x86-64","linux-aarch64").forEach{ target ->
                val file = file("build/libs/${project.name}-$version-$target.jar")
                if(file.exists()){
                    artifact(file){
                        classifier = target
                    }
                }
            }
        }

        pom {
            name.set("Indy VDR Uniffi Kotlin")
            description.set("Kotlin MPP wrapper around indy vdr uniffi")
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
                implementation(libs.kotlinx.serialization.json)
            }
        }

        val commonTest by getting {
            dependencies{
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

android{
    sourceSets["androidTest"].manifest.srcFile("src/androidTest/AndroidManifest.xml")
    namespace = "indy_vdr_uniffi"
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