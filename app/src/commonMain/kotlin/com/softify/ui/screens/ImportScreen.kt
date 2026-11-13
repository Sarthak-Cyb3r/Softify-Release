package com.softify.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.unit.dp
import com.softify.ui.components.AudioFidelityBadge
import com.softify.ui.state.PlayerViewModel
import com.softify.ui.state.SoftifyEngineBridge
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import uniffi.softify_models.TrackMetadata

/**
 * Screen 5: Universal Playlist Importer (Spotify / YouTube)
 */
@Composable
fun ImportScreen(
    viewModel: PlayerViewModel,
    modifier: Modifier = Modifier
) {
    var urlInput by remember { mutableStateOf("") }
    var isResolving by remember { mutableStateOf(false) }
    var progress by remember { mutableFloatStateOf(0f) }
    var importedTracks by remember { mutableStateOf<List<TrackMetadata>>(emptyList()) }
    val scope = rememberCoroutineScope()

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 24.dp)
    ) {
        Spacer(modifier = Modifier.height(24.dp))

        Text(text = "Universal Playlist Importer", style = SoftifyTypography.DisplayHero)
        Spacer(modifier = Modifier.height(4.dp))
        Text(
            text = "Convert any Spotify or YouTube playlist into 320 Kbps CD-quality studio streams with zero rate limits",
            style = SoftifyTypography.Caption
        )

        Spacer(modifier = Modifier.height(20.dp))

        // Glass URL input box
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(56.dp)
                .clip(RoundedCornerShape(12.dp))
                .background(SoftifyColors.Surface2)
                .border(1.dp, SoftifyColors.GlassHighlight, RoundedCornerShape(12.dp))
                .padding(horizontal = 16.dp),
            contentAlignment = Alignment.CenterStart
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text("🔗", style = SoftifyTypography.TitleMedium)
                Spacer(modifier = Modifier.width(10.dp))

                BasicTextField(
                    value = urlInput,
                    onValueChange = { urlInput = it },
                    textStyle = SoftifyTypography.BodyPrimary.copy(color = SoftifyColors.TextPrimary),
                    cursorBrush = SolidColor(SoftifyColors.AccentPrimary),
                    singleLine = true,
                    modifier = Modifier.weight(1f),
                    decorationBox = { innerTextField ->
                        if (urlInput.isEmpty()) {
                            Text(
                                text = "Paste Spotify or YouTube playlist link...",
                                style = SoftifyTypography.BodyPrimary.copy(color = SoftifyColors.TextTertiary)
                            )
                        }
                        innerTextField()
                    }
                )

                // Start Import Button
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(8.dp))
                        .background(if (urlInput.isNotEmpty() && !isResolving) SoftifyColors.AccentPrimary else SoftifyColors.Surface3)
                        .clickable(enabled = urlInput.isNotEmpty() && !isResolving) {
                            isResolving = true
                            progress = 0.1f
                            scope.launch {
                                // Simulate progressive multi-candidate resolution
                                delay(600)
                                progress = 0.45f
                                val mockTracks = SoftifyEngineBridge.search("Top Hits")
                                delay(500)
                                progress = 1.0f
                                importedTracks = mockTracks
                                isResolving = false
                            }
                        }
                        .padding(horizontal = 14.dp, vertical = 8.dp)
                ) {
                    Text(
                        text = if (isResolving) "Resolving..." else "Import",
                        style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.TextPrimary)
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(20.dp))

        // Progress Bar
        if (isResolving || progress > 0f) {
            Column(modifier = Modifier.fillMaxWidth()) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(
                        text = if (progress < 1.0f) "Resolving Studio Master streams..." else "Resolution Complete (${importedTracks.size} Tracks)",
                        style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.AccentStudio)
                    )
                    Text(
                        text = "${(progress * 100).toInt()}%",
                        style = SoftifyTypography.CodeMeta
                    )
                }

                Spacer(modifier = Modifier.height(8.dp))

                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(6.dp)
                        .clip(RoundedCornerShape(3.dp))
                        .background(SoftifyColors.Surface3)
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth(fraction = progress)
                            .height(6.dp)
                            .clip(RoundedCornerShape(3.dp))
                            .background(SoftifyColors.AccentStudio)
                    )
                }
            }
            Spacer(modifier = Modifier.height(20.dp))
        }

        // Matched Tracks List
        if (importedTracks.isNotEmpty()) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "RESOLVED CANDIDATES (320 KBPS AAC)",
                    style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.TextSecondary)
                )
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(8.dp))
                        .background(SoftifyColors.AccentPrimary)
                        .clickable {
                            if (importedTracks.isNotEmpty()) {
                                viewModel.playTrack(importedTracks.first(), importedTracks)
                            }
                        }
                        .padding(horizontal = 12.dp, vertical = 6.dp)
                ) {
                    Text("Play All ▶", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.TextPrimary))
                }
            }

            Spacer(modifier = Modifier.height(10.dp))

            LazyColumn(
                contentPadding = PaddingValues(bottom = 120.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                items(importedTracks) { track ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(10.dp))
                            .background(SoftifyColors.Surface2)
                            .padding(12.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text("✓", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.StatusOnline))
                        Spacer(modifier = Modifier.width(10.dp))
                        Column(modifier = Modifier.weight(1f)) {
                            Text(track.title, style = SoftifyTypography.BodyEmphasis)
                            Text(track.artist, style = SoftifyTypography.Caption)
                        }
                        AudioFidelityBadge()
                    }
                }
            }
        }
    }
}
