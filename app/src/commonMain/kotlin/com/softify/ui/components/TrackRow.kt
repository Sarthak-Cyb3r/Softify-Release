package com.softify.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography
import uniffi.softify_models.TrackMetadata

/**
 * High-fidelity Track Row Component
 */
@Composable
fun TrackRow(
    track: TrackMetadata,
    isPlaying: Boolean = false,
    isOffline: Boolean = false,
    onClick: () -> Unit,
    onLikeClick: (() -> Unit)? = null,
    onMuteClick: (() -> Unit)? = null,
    modifier: Modifier = Modifier
) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(if (isPlaying) SoftifyColors.Surface3 else Color.Transparent)
            .clickable { onClick() }
            .padding(horizontal = 12.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        // Album Cover Thumbnail
        Box(
            modifier = Modifier
                .size(44.dp)
                .clip(RoundedCornerShape(8.dp))
                .background(SoftifyColors.Surface3)
        ) {
            if (!track.coverUrl.isNullOrEmpty()) {
                AsyncImage(
                    model = track.coverUrl,
                    contentDescription = track.title,
                    modifier = Modifier.size(44.dp)
                )
            } else {
                Text(
                    text = "🎵",
                    modifier = Modifier.align(Alignment.Center)
                )
            }
        }

        Spacer(modifier = Modifier.width(12.dp))

        // Title and Artist Column
        Column(modifier = Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = track.title,
                    style = SoftifyTypography.BodyEmphasis.copy(
                        color = if (isPlaying) SoftifyColors.AccentPrimary else SoftifyColors.TextPrimary
                    ),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                if (isOffline) {
                    Spacer(modifier = Modifier.width(6.dp))
                    Box(
                        modifier = Modifier
                            .size(6.dp)
                            .clip(CircleShape)
                            .background(SoftifyColors.StatusOnline)
                    )
                }
            }
            Text(
                text = track.artist,
                style = SoftifyTypography.Caption,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }

        Spacer(modifier = Modifier.width(8.dp))

        // Studio Quality Badge
        AudioFidelityBadge()

        Spacer(modifier = Modifier.width(12.dp))

        // Duration formatted mm:ss
        val minutes = track.durationSeconds / 60
        val seconds = track.durationSeconds % 60
        Text(
            text = "%d:%02d".format(minutes, seconds),
            style = SoftifyTypography.CodeMeta
        )

        Spacer(modifier = Modifier.width(8.dp))

        // Like shortcut
        if (onLikeClick != null) {
            Box(
                modifier = Modifier
                    .size(28.dp)
                    .clip(CircleShape)
                    .clickable { onLikeClick() },
                contentAlignment = Alignment.Center
            ) {
                Text("❤️", style = SoftifyTypography.Caption)
            }
        }

        // Mute shortcut ("Never Play Again")
        if (onMuteClick != null) {
            Box(
                modifier = Modifier
                    .size(28.dp)
                    .clip(CircleShape)
                    .clickable { onMuteClick() },
                contentAlignment = Alignment.Center
            ) {
                Text("🚫", style = SoftifyTypography.Caption)
            }
        }
    }
}
