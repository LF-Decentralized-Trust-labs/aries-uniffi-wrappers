import org.jetbrains.kotlin.gradle.plugin.KotlinSourceSetTree
import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget
import java.util.*

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.kotlinSerialization)
    alias(libs.plugins.androidLibrary)
    id("maven-publish")
}

repositories {
    mavenCentral()
    google()
}

buildscript{
    repositories{
        mavenCentral()
        google()
    }
    dependencies{
        classpath("org.jetbrains.kotlinx:atomicfu-gradle-plugin:0.22.0")
    }
}

apply(plugin = "kotlinx-atomicfu")

dependencies {
    testImplementation("junit:junit:4.13.2")
    testImplementation("org.junit.jupiter:junit-jupiter:5.8.1")
    testImplementation("org.testng:testng:7.1.0")
    testImplementation("org.testng:testng:7.1.0")
}

val vdrDir = file("../../indy-vdr")
val uniffiBindings = vdrDir.resolve("out/kmpp-uniffi")
val jniLibs = uniffiBindings.resolve("jniLibs")

val processBinaries = tasks.register("processBinaries", Copy::class) {
    val directory = buildDir
        .resolve("processedResources")
        .resolve("jvm")
        .resolve("main")

    from(uniffiBindings.resolve("macos-native").resolve("dynamic"))
    include("*.dylib")
    into(directory)
}

tasks.withType<ProcessResources>{
    dependsOn(processBinaries)
}

// Stub secrets to let the project sync and build without the publication values set up
ext["githubUsername"] = null
ext["githubToken"] = null
ext["indyVdrVersion"] = "0.1.0"
ext["wrapperVersion"] = "1"

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
version = "${getExtraString("indyVdrVersion")}-wrapper.${getExtraString("wrapperVersion")}"

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
            name.set("Indy VDR Uniffi Kotlin")
            description.set("Kotlin MPP wrapper around indy vdr uniffi")
            url.set("https://github.com/hyperledger/aries-uniffi-wrappers")

            scm{
                url.set("https://github.com/hyperledger/aries-uniffi-wrappers")
            }
        }
    }
}

private enum class PlatformType {
    APPLE,
    ANDROID
}

kotlin {
    jvmToolchain(17)
    applyDefaultHierarchyTemplate()

    fun addLibs(libDirectory: String, target: KotlinNativeTarget) {
        target.compilations.getByName("main") {
            val uniffi by cinterops.creating {
                val headerDir = uniffiBindings.resolve("nativeInterop/cinterop/headers/indy_vdr_uniffi")
                this.includeDirs(headerDir)
                packageName("indy_vdr_uniffi.cinterop")
                extraOpts("-libraryPath", libDirectory)
            }
        }

        target.binaries.all {
            linkerOpts("-L${libDirectory}", "-lindy_vdr_uniffi")
            linkerOpts("-Wl,-framework,Security")
        }

        target.binaries{
            sharedLib{
                baseName = "indy_vdr_uniffi"
            }
        }
    }


    androidTarget{
        publishLibraryVariants("release")
        compilations.all{
            kotlinOptions.jvmTarget = "1.8"
        }
        instrumentedTestVariant.sourceSetTree.set(KotlinSourceSetTree.test)
        unitTestVariant.sourceSetTree.set(KotlinSourceSetTree.unitTest)
    }

    jvm{
        compilations.all{
            kotlinOptions.jvmTarget = "1.8"
            this.kotlinOptions {
                freeCompilerArgs += listOf("-Xdebug")
            }
        }

        testRuns["test"].executionTask.configure{
            useJUnitPlatform()
        }
    }

    macosX64{
        val libDirectory = "${uniffiBindings}/macos-native/static"
        addLibs(libDirectory, this)
    }

    macosArm64{
        val libDirectory = "${uniffiBindings}/macos-native/static"
        addLibs(libDirectory, this)
    }

    iosX64 {
        val libDirectory = "${vdrDir}/target/x86_64-apple-ios/release"
        addLibs(libDirectory, this)
    }

    iosSimulatorArm64 {
        val libDirectory = "${vdrDir}/target/aarch64-apple-ios-sim/release"
        addLibs(libDirectory, this)
    }

    iosArm64 {
        val libDirectory = "${vdrDir}/target/aarch64-apple-ios/release"
        addLibs(libDirectory, this)
    }
    
    sourceSets {
        val commonMain by getting {
            val commonDir = uniffiBindings.resolve("commonMain").resolve("kotlin")
            val file = commonDir.resolve("indy_vdr_uniffi").resolve("indy_vdr_uniffi.common.kt")
            val find = Regex("\\t|(?:\\s{4})class ([a-zA-Z]{2,50})\\(\\n.{0,50}\\n.{0,20}: ErrorCode\\(\\) \\{(?:.|\\n){0,100}?(?:\\t|(?:\\s{4}))\\}")
            val contents = file.readText().replace(find){
                "class ${it.groupValues[1]}(override val message: kotlin.String): ErrorCode()"
            }
            file.writeText(contents)
            kotlin.srcDir(commonDir)
            dependencies {
                implementation("com.squareup.okio:okio:3.2.0")
                implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.5.1")
                implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.0-RC")
            }
        }

        val commonTest by getting {
            dependencies{
                implementation(kotlin("test"))
                implementation("org.jetbrains.kotlinx:kotlinx-datetime:0.4.0")
            }
        }

        val androidMain by getting {
            kotlin.srcDir(uniffiBindings.resolve("jvmMain").resolve("kotlin"))
            dependencies{
                implementation("net.java.dev.jna:jna:5.7.0@aar")
                implementation("org.jetbrains.kotlinx:atomicfu:0.22.0")
            }
        }

        val jvmMain by getting {
            kotlin.srcDir(uniffiBindings.resolve("jvmMain").resolve("kotlin"))
            dependencies{
                implementation("net.java.dev.jna:jna:5.13.0")
            }
        }

        val nativeMain by getting {
            kotlin.srcDir(uniffiBindings.resolve("nativeMain").resolve("kotlin"))
        }

        all {
            languageSettings.optIn("kotlin.RequiresOptIn")
            languageSettings.optIn("kotlinx.cinterop.ExperimentalForeignApi")
        }
    }
}


android{
    sourceSets["main"].jniLibs.srcDir(jniLibs)
    sourceSets["androidTest"].manifest.srcFile("src/androidTest/AndroidManifest.xml")
    namespace = "indy_vdr_uniffi"
    compileSdk = 33

    ndkVersion = "25.1.8937393"

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