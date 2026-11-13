package com.softify.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography

/**
 * Studio Master Playback Control Deck
 */
@Composable
fun PlaybackControls(
    isPlaying: Boolean,
    onPlayPauseClick: () -> Unit,
    onPreviousClick: () -> Unit,
    onNextClick: () -> Unit,
    isShuffle: Boolean = false,
    isRepeat: Boolean = false,
    onShuffleClick: (() -> Unit)? = null,
    onRepeatClick: (() -> Unit)? = null,
    modifier: Modifier = Modifier
) {
    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(20.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        // Shuffle toggle
        if (onShuffleClick != null) {
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(CircleShape)
                    .clickable { onShuffleClick() },
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = "🔀",
                    style = SoftifyTypography.BodyPrimary.copy(
                        color = if (isShuffle) SoftifyColors.AccentPrimary else SoftifyColors.TextSecondary
                    )
                )
            }
        }

        // Previous button
        Box(
            modifier = Modifier
                .size(42.dp)
                .clip(CircleShape)
                .springPress(onClick = onPreviousClick),
            contentAlignment = Alignment.Center
        ) {
            Text("⏮", style = SoftifyTypography.TitleMedium)
        }

        // Central Play / Pause button
        Box(
            modifier = Modifier
                .size(60.dp)
                .clip(CircleShape)
                .background(SoftifyColors.AccentPrimary)
                .springPress(targetScale = 0.94f, onClick = onPlayPauseClick),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = if (isPlaying) "❚❚" else "▶",
                style = SoftifyTypography.TitleMedium.copy(color = SoftifyColors.TextPrimary)
            )
        }

        // Next button
        Box(
            modifier = Modifier
                .size(42.dp)
                .clip(CircleShape)
                .springPress(onClick = onNextClick),
            contentAlignment = Alignment.Center
        ) {
            Text("⏭", style = SoftifyTypography.TitleMedium)
        }

        // Repeat toggle
        if (onRepeatClick != null) {
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(CircleShape)
                    .clickable { onRepeatClick() },
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = "🔁",
                    style = SoftifyTypography.BodyPrimary.copy(
                        color = if (isRepeat) SoftifyColors.AccentPrimary else SoftifyColors.TextSecondary
                    )
                )
            }
        }
    }
}
