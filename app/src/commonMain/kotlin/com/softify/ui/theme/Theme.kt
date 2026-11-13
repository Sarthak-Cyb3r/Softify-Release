package com.softify.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable

private val DarkColorScheme = darkColorScheme(
    primary = SoftifyColors.AccentPrimary,
    onPrimary = SoftifyColors.TextPrimary,
    secondary = SoftifyColors.AccentStudio,
    onSecondary = SoftifyColors.Surface0,
    background = SoftifyColors.Surface0,
    onBackground = SoftifyColors.TextPrimary,
    surface = SoftifyColors.Surface1,
    onSurface = SoftifyColors.TextPrimary,
    surfaceVariant = SoftifyColors.Surface2,
    onSurfaceVariant = SoftifyColors.TextSecondary,
    outline = SoftifyColors.GlassHighlight
)

@Composable
fun SoftifyTheme(
    content: @Composable () -> Unit
) {
    MaterialTheme(
        colorScheme = DarkColorScheme,
        content = content
    )
}
