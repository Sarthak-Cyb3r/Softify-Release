package com.softify.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
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
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography
import uniffi.softify_models.Mix

/**
 * Daily Mix Frosted Glass Card
 */
@Composable
fun DailyMixCard(
    mix: Mix,
    onPlayClick: () -> Unit,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .width(160.dp)
            .clickable { onClick() }
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(1f)
                .clip(RoundedCornerShape(16.dp))
                .background(SoftifyColors.Surface2)
                .border(1.dp, SoftifyColors.GlassHighlight, RoundedCornerShape(16.dp))
        ) {
            // Artwork montage
            if (mix.tracks.isNotEmpty() && !mix.tracks.first().coverUrl.isNullOrEmpty()) {
                AsyncImage(
                    model = mix.tracks.first().coverUrl,
                    contentDescription = mix.title,
                    modifier = Modifier.fillMaxSize()
                )
            } else {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(SoftifyColors.Surface3),
                    contentAlignment = Alignment.Center
                ) {
                    Text("🎧", style = SoftifyTypography.TitleLarge)
                }
            }

            // Top-right 320k badge
            Box(
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(8.dp)
            ) {
                AudioFidelityBadge()
            }

            // Floating Quick Play button at bottom right
            Box(
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(10.dp)
                    .size(36.dp)
                    .clip(CircleShape)
                    .background(SoftifyColors.AccentPrimary)
                    .clickable { onPlayClick() },
                contentAlignment = Alignment.Center
            ) {
                Text("▶", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.TextPrimary))
            }
        }

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = mix.title,
            style = SoftifyTypography.BodyEmphasis,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis
        )

        Text(
            text = mix.description,
            style = SoftifyTypography.Caption,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis
        )
    }
}
