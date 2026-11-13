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
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import com.softify.ui.components.AudioFidelityBadge
import com.softify.ui.components.DailyMixCard
import com.softify.ui.components.TrackRow
import com.softify.ui.state.PlayerViewModel
import com.softify.ui.state.SoftifyEngineBridge
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography
import kotlinx.coroutines.launch
import uniffi.softify_models.Mood
import uniffi.softify_models.TrackMetadata
import java.util.Calendar

/**
 * Screen 1: Home & Discovery Hub
 */
@Composable
fun HomeScreen(
    viewModel: PlayerViewModel,
    modifier: Modifier = Modifier
) {
    val uiState by viewModel.uiState.collectAsState()
    val scope = rememberCoroutineScope()
    var selectedMood by remember { mutableStateOf<Mood?>(null) }
    var moodTracks by remember { mutableStateOf<List<TrackMetadata>>(emptyList()) }

    // Dynamic greeting based on current local hour
    val greeting = remember {
        val hour = Calendar.getInstance().get(Calendar.HOUR_OF_DAY)
        when (hour) {
            in 5..11 -> "Good Morning"
            in 12..16 -> "Good Afternoon"
            in 17..21 -> "Good Evening"
            else -> "Good Night"
        }
    }

    LaunchedEffect(selectedMood) {
        if (selectedMood != null) {
            scope.launch {
                val mix = SoftifyEngineBridge.getMoodMix(selectedMood!!)
                moodTracks = mix?.tracks ?: emptyList()
            }
        } else {
            moodTracks = emptyList()
        }
    }

    LazyColumn(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 24.dp),
        contentPadding = PaddingValues(bottom = 120.dp)
    ) {
        // Top Greeting Header
        item {
            Spacer(modifier = Modifier.height(24.dp))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column {
                    Text(text = "$greeting, Studio Listener", style = SoftifyTypography.DisplayHero)
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(
                        text = "✨ Studio Master 320k Stream Active",
                        style = SoftifyTypography.Caption.copy(color = SoftifyColors.AccentStudio)
                    )
                }
                AudioFidelityBadge()
            }
            Spacer(modifier = Modifier.height(28.dp))
        }

        // Daily Mixes Carousel
        item {
            Text(text = "FOR YOU · DAILY MIXES", style = SoftifyTypography.TitleMedium)
            Spacer(modifier = Modifier.height(14.dp))
            if (uiState.dailyMixes.isNotEmpty()) {
                LazyRow(
                    horizontalArrangement = Arrangement.spacedBy(16.dp),
                    contentPadding = PaddingValues(end = 24.dp)
                ) {
                    items(uiState.dailyMixes) { mix ->
                        DailyMixCard(
                            mix = mix,
                            onPlayClick = {
                                if (mix.tracks.isNotEmpty()) {
                                    viewModel.playTrack(mix.tracks.first(), mix.tracks)
                                }
                            },
                            onClick = {
                                if (mix.tracks.isNotEmpty()) {
                                    viewModel.playTrack(mix.tracks.first(), mix.tracks)
                                }
                            }
                        )
                    }
                }
            } else {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(100.dp)
                        .clip(RoundedCornerShape(12.dp))
                        .background(SoftifyColors.Surface2)
                        .padding(16.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Text("Daily mixes are curating based on your taste profile...", style = SoftifyTypography.Caption)
                }
            }
            Spacer(modifier = Modifier.height(28.dp))
        }

        // Mood Waves Section
        item {
            Text(text = "MOOD WAVES", style = SoftifyTypography.TitleMedium)
            Spacer(modifier = Modifier.height(12.dp))
            Row(
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                val moods = listOf(
                    Mood.CHILL to "🌙 Chill",
                    Mood.WORKOUT to "🔥 Workout",
                    Mood.FOCUS to "⚡ Focus",
                    Mood.PARTY to "🎉 Party"
                )
                moods.forEach { (mood, label) ->
                    val isSelected = selectedMood == mood
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(999.dp))
                            .background(if (isSelected) SoftifyColors.AccentPrimary else SoftifyColors.Surface3)
                            .border(1.dp, SoftifyColors.GlassHighlight, RoundedCornerShape(999.dp))
                            .clickable {
                                selectedMood = if (isSelected) null else mood
                            }
                            .padding(horizontal = 14.dp, vertical = 8.dp)
                    ) {
                        Text(
                            text = label,
                            style = SoftifyTypography.CaptionBold.copy(
                                color = if (isSelected) SoftifyColors.TextPrimary else SoftifyColors.TextSecondary
                            )
                        )
                    }
                }
            }
            Spacer(modifier = Modifier.height(20.dp))
        }

        // Display Mood Tracks if active
        if (selectedMood != null && moodTracks.isNotEmpty()) {
            item {
                Text(
                    text = "MOOD PLAYLIST · ${selectedMood?.name}",
                    style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.AccentStudio)
                )
                Spacer(modifier = Modifier.height(8.dp))
            }
            items(moodTracks) { track ->
                TrackRow(
                    track = track,
                    isPlaying = uiState.currentTrack?.id == track.id && uiState.isPlaying,
                    onClick = { viewModel.playTrack(track, moodTracks) },
                    onLikeClick = { viewModel.toggleLike(track) },
                    onMuteClick = { viewModel.muteTrack(track) }
                )
                Spacer(modifier = Modifier.height(6.dp))
            }
        }

        // Recently Played Studio Tracks
        item {
            Spacer(modifier = Modifier.height(12.dp))
            Text(text = "RECENTLY PLAYED · STUDIO MASTER QUALITY", style = SoftifyTypography.TitleMedium)
            Spacer(modifier = Modifier.height(14.dp))
        }

        if (uiState.historyTracks.isNotEmpty()) {
            items(uiState.historyTracks) { track ->
                TrackRow(
                    track = track,
                    isPlaying = uiState.currentTrack?.id == track.id && uiState.isPlaying,
                    onClick = { viewModel.playTrack(track, uiState.historyTracks) },
                    onLikeClick = { viewModel.toggleLike(track) },
                    onMuteClick = { viewModel.muteTrack(track) }
                )
                Spacer(modifier = Modifier.height(6.dp))
            }
        } else {
            item {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(120.dp)
                        .clip(RoundedCornerShape(12.dp))
                        .background(SoftifyColors.Surface2)
                        .padding(16.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = "Play your first studio master recording from Search or Daily Mixes",
                        style = SoftifyTypography.Caption
                    )
                }
            }
        }
    }
}
