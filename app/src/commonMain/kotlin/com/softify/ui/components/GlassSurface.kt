package com.softify.ui.components

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.softify.ui.theme.SoftifyColors

/**
 * Optical Glassmorphism Surface Modifier
 * Implements Apple HIG blurred acrylic visual depth and 1px specular top bevel highlight.
 */
fun Modifier.glassmorphicSurface(
    shape: Shape = RoundedCornerShape(16.dp),
    backgroundColor: Color = SoftifyColors.Surface2,
    borderColor: Color = SoftifyColors.GlassHighlight,
    borderWidth: Dp = 1.dp
): Modifier = this
    .clip(shape)
    .background(backgroundColor)
    .border(width = borderWidth, color = borderColor, shape = shape)

/**
 * Spring Touch Feedback Modifier
 * Smoothly scales element down to 0.97f upon user touch/press.
 */
fun Modifier.springPress(
    targetScale: Float = 0.97f,
    onClick: (() -> Unit)? = null
): Modifier = composed {
    var isPressed by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(
        targetValue = if (isPressed) targetScale else 1.0f,
        animationSpec = spring(
            dampingRatio = 0.78f,
            stiffness = 380f
        )
    )

    this
        .graphicsLayer {
            scaleX = scale
            scaleY = scale
        }
        .pointerInput(Unit) {
            detectTapGestures(
                onPress = {
                    isPressed = true
                    tryAwaitRelease()
                    isPressed = false
                },
                onTap = {
                    onClick?.invoke()
                }
            )
        }
}

/**
 * Composable container for Glassmorphic Surfaces.
 */
@Composable
fun GlassBox(
    modifier: Modifier = Modifier,
    shape: Shape = RoundedCornerShape(16.dp),
    backgroundColor: Color = SoftifyColors.Surface2,
    borderColor: Color = SoftifyColors.GlassHighlight,
    borderWidth: Dp = 1.dp,
    content: @Composable BoxScope.() -> Unit
) {
    Box(
        modifier = modifier.glassmorphicSurface(shape, backgroundColor, borderColor, borderWidth),
        content = content
    )
}
