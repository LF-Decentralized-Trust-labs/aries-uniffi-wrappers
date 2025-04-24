rootProject.name = "aries-uniffi-wrappers"

pluginManagement {
    repositories {
        google()
        gradlePluginPortal()
        mavenCentral()
    }
}

dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
    }
}

include(":askar")
project(":askar").name = "askar_uniffi"
include(":anoncreds")
project(":anoncreds").name = "anoncreds_uniffi"
include(":indy-vdr")
project(":indy-vdr").name = "indy_vdr_uniffi"