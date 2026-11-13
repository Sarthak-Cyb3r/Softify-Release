package com.softify.app

import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.WindowState
import androidx.compose.ui.window.application
import com.softify.ui.SoftifyApp

/**
 * Desktop Application Entry Point (Linux, macOS, Windows)
 */
fun main() = application {
    Window(
        onCloseRequest = ::exitApplication,
        title = "Softify — Studio Master 320k Audio",
        state = WindowState(width = 1240.dp, height = 820.dp)
    ) {
        SoftifyApp()
    }
}
