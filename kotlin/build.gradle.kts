plugins {
    alias(libs.plugins.androidLibrary).apply(false)
    alias(libs.plugins.kotlinMultiplatform).apply(false)
    alias(libs.plugins.kotlinJvm) apply false
}

allprojects{
    group = "org.hyperledger"
    version = "0.3.0-wrapper.0-PRE.0"
}