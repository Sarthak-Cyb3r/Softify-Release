package com.softify.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.unit.dp
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography

/**
 * Vertical Studio Equalizer Parametric Fader (-12 dB to +12 dB)
 */
@Composable
fun EqualizerFader(
    frequencyLabel: String,
    gainDb: Float,
    onGainChanged: (Float) -> Unit,
    modifier: Modifier = Modifier
) {
    var heightPx by remember { mutableFloatStateOf(1f) }

    // Normalized progress from 0.0 (-12dB) to 1.0 (+12dB)
    val normalized = ((gainDb + 12f) / 24f).coerceIn(0f, 1f)

    Column(
        modifier = modifier.width(44.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        // Gain readout
        Text(
            text = if (gainDb > 0) "+%.1f".format(gainDb) else "%.1f".format(gainDb),
            style = SoftifyTypography.CodeMeta
        )

        Spacer(modifier = Modifier.height(8.dp))

        // Vertical Fader Slot
        Box(
            modifier = Modifier
                .width(16.dp)
                .height(180.dp)
                .onSizeChanged { heightPx = it.height.toFloat() }
                .clip(RoundedCornerShape(8.dp))
                .background(SoftifyColors.Surface2)
                .border(1.dp, SoftifyColors.GlassHighlight, RoundedCornerShape(8.dp))
                .pointerInput(Unit) {
                    detectTapGestures { offset ->
                        val ratio = 1f - (offset.y / heightPx).coerceIn(0f, 1f)
                        val newGain = (ratio * 24f) - 12f
                        onGainChanged(newGain)
                    }
                }
                .pointerInput(Unit) {
                    detectDragGestures { change, _ ->
                        change.consume()
                        val ratio = 1f - (change.position.y / heightPx).coerceIn(0f, 1f)
                        val newGain = (ratio * 24f) - 12f
                        onGainChanged(newGain)
                    }
                },
            contentAlignment = Alignment.BottomCenter
        ) {
            // Fill bar
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .fillMaxHeight(fraction = normalized)
                    .background(SoftifyColors.AccentStudio.copy(alpha = 0.4f))
            )

            // Center 0dB detent mark
            Box(
                modifier = Modifier
                    .align(Alignment.Center)
                    .fillMaxWidth()
                    .height(1.dp)
                    .background(SoftifyColors.GlassHighlight)
            )

            // Draggable Fader Thumb Cap
            Box(
                modifier = Modifier
                    .padding(bottom = (normalized * (heightPx - 30)).coerceAtLeast(0f).dp / 2)
                    .size(24.dp, 12.dp)
                    .clip(CircleShape)
                    .background(SoftifyColors.AccentStudio)
            )
        }

        Spacer(modifier = Modifier.height(8.dp))

        // Frequency Label
        Text(
            text = frequencyLabel,
            style = SoftifyTypography.CaptionBold
        )
    }
}
