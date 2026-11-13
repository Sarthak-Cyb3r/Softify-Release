package com.softify.app

import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.WindowState
import androidx.compose.ui.window.application
import com.softify.ui.SoftifyApp
import java.io.File

/**
 * Desktop Application Entry Point (Linux, macOS, Windows)
 */
fun main() {
    val projectRoot = File(".").absoluteFile
    val candidateDirs = listOf(
        File(projectRoot, "dist/linux/softify-linux-x86_64"),
        File(projectRoot, "app/src/desktopMain/resources/linux-x86-64"),
        File(System.getProperty("user.home"), ".local/lib")
    )
    for (dir in candidateDirs) {
        if (File(dir, "libsoftify_core.so").exists()) {
            System.setProperty("jna.library.path", dir.absolutePath)
            break
        }
    }

    application {
        Window(
            onCloseRequest = ::exitApplication,
            title = "Softify — Studio Master 320k Audio",
            state = WindowState(width = 1240.dp, height = 820.dp)
        ) {
            SoftifyApp()
        }
    }
}
