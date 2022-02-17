import java.util.Properties // this is from `buildSrc`

plugins {
    kotlin("jvm") version "1.3.72" apply false
    id("software.amazon.smithy").version("0.6.0")
    id("org.jetbrains.dokka") version "0.10.0"
}

val smithyVersion: String by project

tasks["jar"].enabled = false


// this is from `buildSrc`
// Load properties manually to avoid hard coding smithy version
val props = Properties().apply {
    file("./gradle.properties").inputStream().use { load(it) }
}


buildscript {
    val smithyVersion: String by project
    val kotlinVersion: String by project // from rust runtime

    // from rust runtime
    repositories {
        google()
    }

    dependencies {
        classpath("software.amazon.smithy:smithy-aws-traits:$smithyVersion")
        classpath("software.amazon.smithy:smithy-cli:$smithyVersion")

        // from rust runtime
        classpath("org.jetbrains.kotlin:kotlin-gradle-plugin:$kotlinVersion")
    }
}

allprojects {
    repositories {
        mavenLocal()
        mavenCentral()
        google()
    }
}

//enableFeaturePreview("GRADLE_METADATA")

dependencies {
    // buildSrc

    implementation(project(":codegen"))
    implementation(project(":codegen-server"))

    //implementation(project(":aws:sdk-codegen"))
    implementation("software.amazon.smithy:smithy-protocol-test-traits:$smithyVersion")
    implementation("software.amazon.smithy:smithy-aws-traits:$smithyVersion")
    implementation("software.amazon.smithy:smithy-aws-iam-traits:$smithyVersion")
    implementation("software.amazon.smithy:smithy-aws-cloudformation-traits:$smithyVersion")
}
