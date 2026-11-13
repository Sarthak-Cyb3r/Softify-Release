package com.softify.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.unit.dp
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography

/**
 * High-precision Audio Scrubber Timeline
 */
@Composable
fun ScrubberTimeline(
    positionMs: Long,
    durationMs: Long,
    onSeek: (Long) -> Unit,
    modifier: Modifier = Modifier
) {
    var widthPx by remember { mutableFloatStateOf(1f) }
    val progress = if (durationMs > 0) (positionMs.toFloat() / durationMs.toFloat()).coerceIn(0f, 1f) else 0f

    Column(modifier = modifier.fillMaxWidth()) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(18.dp)
                .onSizeChanged { widthPx = it.width.toFloat() }
                .pointerInput(durationMs) {
                    detectTapGestures { offset ->
                        val newProgress = (offset.x / widthPx).coerceIn(0f, 1f)
                        onSeek((newProgress * durationMs).toLong())
                    }
                }
                .pointerInput(durationMs) {
                    detectDragGestures { change, _ ->
                        change.consume()
                        val newProgress = (change.position.x / widthPx).coerceIn(0f, 1f)
                        onSeek((newProgress * durationMs).toLong())
                    }
                },
            contentAlignment = Alignment.CenterStart
        ) {
            // Background track
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(4.dp)
                    .clip(RoundedCornerShape(2.dp))
                    .background(Color.White.copy(alpha = 0.08f))
            )

            // Played track
            Box(
                modifier = Modifier
                    .fillMaxWidth(fraction = progress)
                    .height(4.dp)
                    .clip(RoundedCornerShape(2.dp))
                    .background(SoftifyColors.AccentPrimary)
            )

            // Scrubber knob thumb
            Box(
                modifier = Modifier
                    .padding(start = (progress * (widthPx - 32)).coerceAtLeast(0f).dp / 2)
                    .size(10.dp)
                    .clip(CircleShape)
                    .background(SoftifyColors.TextPrimary)
            )
        }

        // Timecodes
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            val curSeconds = (positionMs / 1000)
            val durSeconds = (durationMs / 1000)
            Text(
                text = "%d:%02d".format(curSeconds / 60, curSeconds % 60),
                style = SoftifyTypography.CodeMeta
            )
            Spacer(modifier = Modifier.weight(1f))
            Text(
                text = "%d:%02d".format(durSeconds / 60, durSeconds % 60),
                style = SoftifyTypography.CodeMeta
            )
        }
    }
}
