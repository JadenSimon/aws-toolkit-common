pluginManagement {
    repositories {
        mavenCentral()
        maven("https://plugins.gradle.org/m2/")
        google()
        gradlePluginPortal()
    }
}

rootProject.name = "software.amazon.smithy.rust.codegen.toolkits"
enableFeaturePreview("GRADLE_METADATA")

include("codegen")
include("codegen-server")
include("rust-runtime")

project(":codegen").projectDir = File("/Users/sijaden/telemetry/rs-smithy/smithy-rs/codegen")
project(":codegen-server").projectDir = File("/Users/sijaden/telemetry/rs-smithy/smithy-rs/codegen-server")
project(":rust-runtime").projectDir = File("/Users/sijaden/telemetry/rs-smithy/smithy-rs/rust-runtime")
