import org.jetbrains.compose.desktop.application.dsl.TargetFormat

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
    alias(libs.plugins.androidApplication)
}

kotlin {
    jvm("desktop")
    androidTarget()

    sourceSets {
        val desktopMain by getting
        val androidMain by getting

        commonMain.dependencies {
            implementation(compose.runtime)
            implementation(compose.foundation)
            implementation(compose.material3)
            implementation(compose.ui)
            implementation(compose.components.resources)
            implementation(compose.components.uiToolingPreview)
            implementation(libs.kotlinx.coroutines.core)
            implementation(libs.net.java.dev.jna)
            implementation(libs.coil.compose)
            implementation(libs.coil.network.ktor)
        }

        desktopMain.dependencies {
            implementation(compose.desktop.currentOs)
            implementation(libs.kotlinx.coroutines.swing)
        }

        androidMain.dependencies {
            implementation(libs.androidx.activity.compose)
        }
    }
}

android {
    namespace = "com.softify.app"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.softify.app"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0-beta"
    }

    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

compose.desktop {
    application {
        mainClass = "com.softify.app.DesktopMainKt"

        nativeDistributions {
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb)
            packageName = "Softify"
            packageVersion = "0.1.0"
            description = "Softify Next-Gen Studio Audio Player"
            copyright = "© 2026 Softify"
            vendor = "Softify Studio"

            linux {
                iconFile.set(project.file("src/desktopMain/resources/icon.png"))
                debMaintainer = "sarthak@softify.audio"
                menuGroup = "Audio;AudioVideo;"
                appCategory = "AudioVideo"
            }
        }
    }
}
