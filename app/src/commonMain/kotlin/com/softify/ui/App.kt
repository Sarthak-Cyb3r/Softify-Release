package com.softify.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
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
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.softify.ui.components.AudioFidelityBadge
import com.softify.ui.components.PlaybackControls
import com.softify.ui.components.ScrubberTimeline
import com.softify.ui.screens.EqualizerScreen
import com.softify.ui.screens.HomeScreen
import com.softify.ui.screens.ImportScreen
import com.softify.ui.screens.LibraryScreen
import com.softify.ui.screens.PlayerScreen
import com.softify.ui.screens.SearchScreen
import com.softify.ui.screens.SettingsScreen
import com.softify.ui.screens.WrappedScreen
import com.softify.ui.state.NavigationTab
import com.softify.ui.state.PlayerViewModel
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTheme
import com.softify.ui.theme.SoftifyTypography

/**
 * Root Compose Multiplatform Application Shell
 */
@Composable
fun SoftifyApp(
    viewModel: PlayerViewModel = remember { PlayerViewModel() }
) {
    val uiState by viewModel.uiState.collectAsState()

    SoftifyTheme {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(SoftifyColors.Surface0)
        ) {
            Row(modifier = Modifier.fillMaxSize()) {
                // Persistent Left Glass Sidebar (240dp)
                Column(
                    modifier = Modifier
                        .width(240.dp)
                        .fillMaxHeight()
                        .background(SoftifyColors.Surface1)
                        .border(
                            width = 1.dp,
                            color = SoftifyColors.GlassHighlight,
                            shape = RoundedCornerShape(0.dp)
                        )
                        .padding(20.dp)
                ) {
                    // App Brand Wordmark
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier.padding(bottom = 28.dp)
                    ) {
                        Box(
                            modifier = Modifier
                                .size(34.dp)
                                .clip(RoundedCornerShape(8.dp))
                                .background(SoftifyColors.AccentPrimary),
                            contentAlignment = Alignment.Center
                        ) {
                            Text("S", style = SoftifyTypography.TitleMedium.copy(color = SoftifyColors.TextPrimary))
                        }
                        Spacer(modifier = Modifier.width(10.dp))
                        Column {
                            Text("SOFTIFY", style = SoftifyTypography.TitleMedium)
                            Text("STUDIO MASTER", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.AccentStudio))
                        }
                    }

                    // Navigation Items
                    val navItems = listOf(
                        NavigationTab.HOME to ("🏠  Home" to "Browse Studio Mixes"),
                        NavigationTab.SEARCH to ("🔍  Search" to "FTS5 Catalog"),
                        NavigationTab.LIBRARY to ("📚  Library" to "Liked & Offline"),
                        NavigationTab.IMPORT to ("🔗  Import" to "Spotify / YouTube"),
                        NavigationTab.WRAPPED to ("✨  Wrapped" to "Year in Review"),
                        NavigationTab.EQUALIZER to ("🎚️  Equalizer" to "10-Band Acoustics"),
                        NavigationTab.SETTINGS to ("⚙️  Settings" to "Preferences & Update")
                    )

                    navItems.forEach { (tab, labels) ->
                        val (title, subtitle) = labels
                        val isSelected = uiState.currentTab == tab

                        Column(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(10.dp))
                                .background(if (isSelected) SoftifyColors.Surface3 else SoftifyColors.Surface2.copy(alpha = 0f))
                                .clickable { viewModel.setTab(tab) }
                                .padding(horizontal = 14.dp, vertical = 10.dp)
                        ) {
                            Text(
                                text = title,
                                style = SoftifyTypography.BodyEmphasis.copy(
                                    color = if (isSelected) SoftifyColors.AccentPrimary else SoftifyColors.TextPrimary
                                )
                            )
                            Text(
                                text = subtitle,
                                style = SoftifyTypography.Caption
                            )
                        }
                        Spacer(modifier = Modifier.height(4.dp))
                    }

                    Spacer(modifier = Modifier.weight(1f))

                    // Fidelity Footer Indicator
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(10.dp))
                            .background(SoftifyColors.Surface2)
                            .padding(12.dp)
                    ) {
                        Column {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Box(
                                    modifier = Modifier
                                        .size(6.dp)
                                        .clip(CircleShape)
                                        .background(SoftifyColors.StatusOnline)
                                )
                                Spacer(modifier = Modifier.width(6.dp))
                                Text("320 Kbps CD Master", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.AccentStudio))
                            }
                            Spacer(modifier = Modifier.height(2.dp))
                            Text("DES-ECB Studio Decryptor", style = SoftifyTypography.Caption)
                        }
                    }
                }

                // Dynamic Center Content Surface
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxHeight()
                ) {
                    when (uiState.currentTab) {
                        NavigationTab.HOME -> HomeScreen(viewModel)
                        NavigationTab.SEARCH -> SearchScreen(viewModel)
                        NavigationTab.PLAYER -> PlayerScreen(viewModel)
                        NavigationTab.LIBRARY -> LibraryScreen(viewModel)
                        NavigationTab.IMPORT -> ImportScreen(viewModel)
                        NavigationTab.WRAPPED -> WrappedScreen(viewModel)
                        NavigationTab.EQUALIZER -> EqualizerScreen(viewModel)
                        NavigationTab.SETTINGS -> SettingsScreen(viewModel)
                    }
                }
            }

            // Persistent Bottom Player Bar (88dp)
            if (uiState.currentTab != NavigationTab.PLAYER && uiState.currentTrack != null) {
                val track = uiState.currentTrack!!

                Row(
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .height(88.dp)
                        .background(SoftifyColors.Surface1)
                        .border(1.dp, SoftifyColors.GlassHighlight, RoundedCornerShape(0.dp))
                        .padding(horizontal = 24.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    // Left Track Info with Cover
                    Row(
                        modifier = Modifier
                            .width(280.dp)
                            .clickable { viewModel.setTab(NavigationTab.PLAYER) },
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Box(
                            modifier = Modifier
                                .size(50.dp)
                                .clip(RoundedCornerShape(8.dp))
                                .background(SoftifyColors.Surface3)
                        ) {
                            if (!track.coverUrl.isNullOrEmpty()) {
                                AsyncImage(
                                    model = track.coverUrl,
                                    contentDescription = track.title,
                                    modifier = Modifier.fillMaxSize()
                                )
                            }
                        }

                        Spacer(modifier = Modifier.width(12.dp))

                        Column {
                            Text(
                                text = track.title,
                                style = SoftifyTypography.BodyEmphasis,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                            Text(
                                text = track.artist,
                                style = SoftifyTypography.Caption,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                    }

                    // Center Scrubber & Controls
                    Column(
                        modifier = Modifier.weight(1f).padding(horizontal = 32.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        PlaybackControls(
                            isPlaying = uiState.isPlaying,
                            onPlayPauseClick = { viewModel.togglePlayPause() },
                            onPreviousClick = { viewModel.previousTrack() },
                            onNextClick = { viewModel.nextTrack() }
                        )

                        Spacer(modifier = Modifier.height(4.dp))

                        ScrubberTimeline(
                            positionMs = uiState.positionMs,
                            durationMs = uiState.durationMs,
                            onSeek = { viewModel.seekTo(it) }
                        )
                    }

                    // Right Controls (Audio Badge & Fullscreen Expand)
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(14.dp)
                    ) {
                        AudioFidelityBadge()

                        Box(
                            modifier = Modifier
                                .size(36.dp)
                                .clip(CircleShape)
                                .background(SoftifyColors.Surface3)
                                .clickable { viewModel.setTab(NavigationTab.PLAYER) },
                            contentAlignment = Alignment.Center
                        ) {
                            Text("⤢", style = SoftifyTypography.TitleMedium)
                        }
                    }
                }
            }
        }
    }
}
