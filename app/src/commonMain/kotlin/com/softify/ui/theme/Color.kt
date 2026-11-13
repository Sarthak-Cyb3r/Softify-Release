package com.softify.ui.theme

import androidx.compose.ui.graphics.Color

/**
 * Softify Design Tokens - Color Palette & Glass Materials
 * Exactly according to DESIGN.md Specification
 */
object SoftifyColors {
    // Canvas & Glass Surfaces
    val Surface0 = Color(0xFF050508)
    val Surface1 = Color(0xA6121218)
    val Surface2 = Color(0x0DFFFFFF)
    val Surface3 = Color(0x1AFFFFFF)

    // Specular Bevel Highlights & Ambient Occlusion Shadows
    val GlassHighlight = Color(0x1FFFFFFF)
    val GlassShadow = Color(0x66000000)

    // Brand & Fidelity Accents
    val AccentPrimary = Color(0xFFFA243C)
    val AccentHover = Color(0xFFFC3C44)
    val AccentStudio = Color(0xFFF59E0B)

    // Typography Opacities
    val TextPrimary = Color(0xFFFFFFFF)
    val TextSecondary = Color(0xFF8E8E93)
    val TextTertiary = Color(0xFF48484A)

    // System Status
    val StatusOnline = Color(0xFF34C759)
    val StatusDanger = Color(0xFFFF453A)
}
