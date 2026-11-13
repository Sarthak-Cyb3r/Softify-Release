package com.softify.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
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
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.softify.ui.components.AudioFidelityBadge
import com.softify.ui.components.PlaybackControls
import com.softify.ui.components.ScrubberTimeline
import com.softify.ui.components.SyncedLyricsView
import com.softify.ui.state.NavigationTab
import com.softify.ui.state.PlayerViewModel
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography

/**
 * Screen 3: Fullscreen Studio Master Player & Synced Lyrics
 */
@Composable
fun PlayerScreen(
    viewModel: PlayerViewModel,
    modifier: Modifier = Modifier
) {
    val uiState by viewModel.uiState.collectAsState()
    val track = uiState.currentTrack
    var showLyrics by remember { mutableStateOf(false) }

    Box(
        modifier = modifier
            .fillMaxSize()
            .background(SoftifyColors.Surface0)
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 32.dp, vertical = 24.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            // Top Nav Header
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Box(
                    modifier = Modifier
                        .size(36.dp)
                        .clip(CircleShape)
                        .clickable { viewModel.setTab(NavigationTab.HOME) },
                    contentAlignment = Alignment.Center
                ) {
                    Text("⌵", style = SoftifyTypography.TitleMedium)
                }

                Text(
                    text = "NOW PLAYING STUDIO",
                    style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.TextSecondary)
                )

                Box(
                    modifier = Modifier
                        .size(36.dp)
                        .clip(CircleShape)
                        .clickable { viewModel.setTab(NavigationTab.EQUALIZER) },
                    contentAlignment = Alignment.Center
                ) {
                    Text("🎚️", style = SoftifyTypography.BodyPrimary)
                }
            }

            Spacer(modifier = Modifier.height(20.dp))

            if (track != null) {
                if (showLyrics) {
                    // Synced Lyrics View Mode
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(20.dp))
                            .background(SoftifyColors.Surface1)
                            .border(1.dp, SoftifyColors.GlassHighlight, RoundedCornerShape(20.dp))
                    ) {
                        SyncedLyricsView(
                            lyrics = uiState.lyrics,
                            currentPositionMs = uiState.positionMs,
                            onSeekToTimestamp = { viewModel.seekTo(it) }
                        )
                    }
                } else {
                    // Standard Artwork Presentation Mode
                    Box(
                        modifier = Modifier
                            .size(320.dp)
                            .clip(RoundedCornerShape(24.dp))
                            .background(SoftifyColors.Surface2)
                            .border(1.dp, SoftifyColors.GlassHighlight, RoundedCornerShape(24.dp))
                    ) {
                        if (!track.coverUrl.isNullOrEmpty()) {
                            AsyncImage(
                                model = track.coverUrl,
                                contentDescription = track.title,
                                modifier = Modifier.fillMaxSize()
                            )
                        } else {
                            Box(
                                modifier = Modifier.fillMaxSize(),
                                contentAlignment = Alignment.Center
                            ) {
                                Text("🎵", style = SoftifyTypography.DisplayHero)
                            }
                        }
                    }

                    Spacer(modifier = Modifier.height(28.dp))

                    // Track Title & Artist
                    Text(
                        text = track.title,
                        style = SoftifyTypography.TitleLarge,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        textAlign = TextAlign.Center
                    )

                    Spacer(modifier = Modifier.height(6.dp))

                    Text(
                        text = track.artist,
                        style = SoftifyTypography.BodyPrimary.copy(color = SoftifyColors.TextSecondary),
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        textAlign = TextAlign.Center
                    )

                    Spacer(modifier = Modifier.height(14.dp))

                    // Audio Fidelity HUD
                    val streamInfo = uiState.currentStream
                    val bitrateText = if (streamInfo != null) "${streamInfo.bitrate.toInt() / 1000} KBPS" else "320 KBPS"
                    val codecText = streamInfo?.codec?.uppercase() ?: "AAC"
                    AudioFidelityBadge(bitrateText = bitrateText, codecText = codecText)
                }

                Spacer(modifier = Modifier.weight(1f))

                // Scrubber Timeline
                ScrubberTimeline(
                    positionMs = uiState.positionMs,
                    durationMs = uiState.durationMs,
                    onSeek = { viewModel.seekTo(it) }
                )

                Spacer(modifier = Modifier.height(16.dp))

                // Playback Controls Deck
                PlaybackControls(
                    isPlaying = uiState.isPlaying,
                    onPlayPauseClick = { viewModel.togglePlayPause() },
                    onPreviousClick = { viewModel.previousTrack() },
                    onNextClick = { viewModel.nextTrack() },
                    isShuffle = uiState.isShuffle,
                    isRepeat = uiState.isRepeat
                )

                Spacer(modifier = Modifier.height(20.dp))

                // Bottom Action Strip
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    // Lyrics Toggle Pill
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(999.dp))
                            .background(if (showLyrics) SoftifyColors.AccentPrimary else SoftifyColors.Surface3)
                            .clickable { showLyrics = !showLyrics }
                            .padding(horizontal = 14.dp, vertical = 8.dp)
                    ) {
                        Text(
                            text = if (showLyrics) "Artwork 🖼" else "Lyrics 💬",
                            style = SoftifyTypography.CaptionBold
                        )
                    }

                    // Like Toggle
                    val isLiked = uiState.likedTracks.any { it.id == track.id }
                    Box(
                        modifier = Modifier
                            .size(38.dp)
                            .clip(CircleShape)
                            .clickable { viewModel.toggleLike(track) },
                        contentAlignment = Alignment.Center
                    ) {
                        Text(if (isLiked) "❤️" else "🤍", style = SoftifyTypography.TitleMedium)
                    }

                    // Mute Track Pill
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(999.dp))
                            .background(SoftifyColors.Surface3)
                            .clickable { viewModel.muteTrack(track) }
                            .padding(horizontal = 14.dp, vertical = 8.dp)
                    ) {
                        Text("🚫 Never Play Again", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.StatusDanger))
                    }
                }
            } else {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    Text("Select a studio track to start listening", style = SoftifyTypography.BodyPrimary)
                }
            }
        }
    }
}
