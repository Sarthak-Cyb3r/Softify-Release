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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
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
import androidx.compose.ui.unit.dp
import com.softify.ui.components.TrackRow
import com.softify.ui.state.PlayerViewModel
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography

/**
 * Screen 4: Library, Liked Songs & Offline Cache
 */
@Composable
fun LibraryScreen(
    viewModel: PlayerViewModel,
    modifier: Modifier = Modifier
) {
    val uiState by viewModel.uiState.collectAsState()
    var selectedTab by remember { mutableStateOf("Liked Songs") }

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 24.dp)
    ) {
        Spacer(modifier = Modifier.height(24.dp))

        // Title and Create Playlist action
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(text = "Your Studio Library", style = SoftifyTypography.DisplayHero)

            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(999.dp))
                    .background(SoftifyColors.AccentPrimary)
                    .clickable { /* Create playlist dialog */ }
                    .padding(horizontal = 14.dp, vertical = 8.dp)
            ) {
                Text("+ New Playlist", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.TextPrimary))
            }
        }

        Spacer(modifier = Modifier.height(20.dp))

        // Segmented Switcher Pills
        Row(
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            modifier = Modifier.fillMaxWidth()
        ) {
            listOf("Liked Songs (${uiState.likedTracks.size})", "Playlists (4)", "Offline Cache").forEach { tabLabel ->
                val base = tabLabel.substringBefore(" ")
                val isSelected = selectedTab.startsWith(base)
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(999.dp))
                        .background(if (isSelected) SoftifyColors.Surface3 else SoftifyColors.Surface2)
                        .border(1.dp, if (isSelected) SoftifyColors.AccentPrimary else SoftifyColors.GlassHighlight, RoundedCornerShape(999.dp))
                        .clickable { selectedTab = tabLabel }
                        .padding(horizontal = 16.dp, vertical = 8.dp)
                ) {
                    Text(
                        text = tabLabel,
                        style = SoftifyTypography.CaptionBold.copy(
                            color = if (isSelected) SoftifyColors.TextPrimary else SoftifyColors.TextSecondary
                        )
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(20.dp))

        // Display Tracks according to selected category
        when {
            selectedTab.startsWith("Liked") -> {
                if (uiState.likedTracks.isNotEmpty()) {
                    LazyColumn(
                        contentPadding = PaddingValues(bottom = 120.dp),
                        verticalArrangement = Arrangement.spacedBy(6.dp)
                    ) {
                        items(uiState.likedTracks) { track ->
                            TrackRow(
                                track = track,
                                isPlaying = uiState.currentTrack?.id == track.id && uiState.isPlaying,
                                onClick = { viewModel.playTrack(track, uiState.likedTracks) },
                                onLikeClick = { viewModel.toggleLike(track) },
                                onMuteClick = { viewModel.muteTrack(track) }
                            )
                        }
                    }
                } else {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(180.dp)
                            .clip(RoundedCornerShape(16.dp))
                            .background(SoftifyColors.Surface2)
                            .padding(24.dp),
                        contentAlignment = Alignment.Center
                    ) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Text("❤️", style = SoftifyTypography.DisplayHero)
                            Spacer(modifier = Modifier.height(8.dp))
                            Text("No liked tracks yet. Tap the heart icon on any song to save it here.", style = SoftifyTypography.Caption)
                        }
                    }
                }
            }

            selectedTab.startsWith("Playlists") -> {
                LazyColumn(
                    contentPadding = PaddingValues(bottom = 120.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    val defaultPlaylists = listOf(
                        "Midnight Focus Studio" to "42 Tracks · 2 hrs 48 mins",
                        "Hi-Res Acoustic Sessions" to "28 Tracks · 1 hr 54 mins",
                        "Daily Workout Mix" to "35 Tracks · 2 hrs 10 mins",
                        "Late Night Synthwave" to "50 Tracks · 3 hrs 22 mins"
                    )
                    items(defaultPlaylists) { (name, info) ->
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(14.dp))
                                .background(SoftifyColors.Surface2)
                                .border(1.dp, SoftifyColors.GlassHighlight, RoundedCornerShape(14.dp))
                                .padding(16.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(48.dp)
                                    .clip(RoundedCornerShape(10.dp))
                                    .background(SoftifyColors.Surface3),
                                contentAlignment = Alignment.Center
                            ) {
                                Text("📁", style = SoftifyTypography.TitleMedium)
                            }
                            Spacer(modifier = Modifier.width(16.dp))
                            Column {
                                Text(name, style = SoftifyTypography.BodyEmphasis)
                                Text(info, style = SoftifyTypography.Caption)
                            }
                        }
                    }
                }
            }

            else -> {
                // Offline Cache View
                Column {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(14.dp))
                            .background(SoftifyColors.Surface2)
                            .padding(16.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Box(
                            modifier = Modifier
                                .size(10.dp)
                                .clip(CircleShape)
                                .background(SoftifyColors.StatusOnline)
                        )
                        Spacer(modifier = Modifier.width(10.dp))
                        Text("96 Tracks Cached Locally · 1.42 GB M4A (320 Kbps Studio Master)", style = SoftifyTypography.CaptionBold)
                    }

                    Spacer(modifier = Modifier.height(16.dp))

                    LazyColumn(
                        contentPadding = PaddingValues(bottom = 120.dp),
                        verticalArrangement = Arrangement.spacedBy(6.dp)
                    ) {
                        items(uiState.historyTracks) { track ->
                            TrackRow(
                                track = track,
                                isOffline = true,
                                isPlaying = uiState.currentTrack?.id == track.id && uiState.isPlaying,
                                onClick = { viewModel.playTrack(track, uiState.historyTracks) }
                            )
                        }
                    }
                }
            }
        }
    }
}
