plugins {
    alias(libs.plugins.androidLibrary).apply(false)
    alias(libs.plugins.kotlinMultiplatform).apply(false)
    alias(libs.plugins.kotlinJvm) apply false
}

allprojects{
    group = "org.hyperledger"
    version = "0.3.1"
}



tasks.register("publishAllToMavenLocal"){
    dependsOn("anoncreds_uniffi:publishToMavenLocal")
    dependsOn("askar_uniffi:publishToMavenLocal")
    dependsOn("indy_vdr_uniffi:publishToMavenLocal")
}