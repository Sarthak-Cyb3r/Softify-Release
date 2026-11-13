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
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
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
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.unit.dp
import com.softify.ui.components.TrackRow
import com.softify.ui.state.PlayerViewModel
import com.softify.ui.state.SoftifyEngineBridge
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import uniffi.softify_models.TrackMetadata

/**
 * Screen 2: Search & Live Catalog Exploration (SQLite FTS5 + Live JioSaavn / Spotify)
 */
@Composable
fun SearchScreen(
    viewModel: PlayerViewModel,
    modifier: Modifier = Modifier
) {
    val uiState by viewModel.uiState.collectAsState()
    val scope = rememberCoroutineScope()
    var query by remember { mutableStateOf("") }
    var isSearching by remember { mutableStateOf(false) }
    var searchResults by remember { mutableStateOf<List<TrackMetadata>>(emptyList()) }
    var activeFilter by remember { mutableStateOf("All") }

    // Debounced real-time FTS5 and network search
    LaunchedEffect(query) {
        if (query.trim().length >= 2) {
            isSearching = true
            delay(200) // 200ms debounce
            scope.launch {
                val results = SoftifyEngineBridge.search(query.trim())
                searchResults = results
                isSearching = false
            }
        } else if (query.isEmpty()) {
            searchResults = emptyList()
            isSearching = false
        }
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 24.dp)
    ) {
        Spacer(modifier = Modifier.height(24.dp))

        Text(text = "Search Catalog", style = SoftifyTypography.DisplayHero)
        Spacer(modifier = Modifier.height(16.dp))

        // Glass Search Input Field
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(52.dp)
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
                Text("🔍", style = SoftifyTypography.TitleMedium)
                Spacer(modifier = Modifier.padding(horizontal = 6.dp))

                BasicTextField(
                    value = query,
                    onValueChange = { query = it },
                    textStyle = SoftifyTypography.BodyPrimary.copy(color = SoftifyColors.TextPrimary),
                    cursorBrush = SolidColor(SoftifyColors.AccentPrimary),
                    singleLine = true,
                    modifier = Modifier.weight(1f),
                    decorationBox = { innerTextField ->
                        if (query.isEmpty()) {
                            Text(
                                text = "Search tracks, artists, YouTube links...",
                                style = SoftifyTypography.BodyPrimary.copy(color = SoftifyColors.TextTertiary)
                            )
                        }
                        innerTextField()
                    }
                )

                if (query.isNotEmpty()) {
                    Box(
                        modifier = Modifier
                            .size(24.dp)
                            .clip(RoundedCornerShape(12.dp))
                            .clickable { query = "" },
                        contentAlignment = Alignment.Center
                    ) {
                        Text("✕", style = SoftifyTypography.Caption)
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(14.dp))

        // Filter Pills Row
        Row(
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            modifier = Modifier.fillMaxWidth()
        ) {
            listOf("All", "Songs", "Artists", "Playlists").forEach { filter ->
                val isSelected = activeFilter == filter
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(999.dp))
                        .background(if (isSelected) SoftifyColors.AccentPrimary else SoftifyColors.Surface3)
                        .clickable { activeFilter = filter }
                        .padding(horizontal = 14.dp, vertical = 6.dp)
                ) {
                    Text(
                        text = filter,
                        style = SoftifyTypography.CaptionBold.copy(
                            color = if (isSelected) SoftifyColors.TextPrimary else SoftifyColors.TextSecondary
                        )
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(16.dp))

        // Results List
        if (isSearching) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(140.dp),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = "Searching Studio Masters & FTS5 Cache...",
                    style = SoftifyTypography.Caption.copy(color = SoftifyColors.AccentStudio)
                )
            }
        } else if (searchResults.isNotEmpty()) {
            LazyColumn(
                contentPadding = PaddingValues(bottom = 120.dp),
                verticalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                item {
                    Text(
                        text = "MATCHED STUDIO TRACKS (${searchResults.size})",
                        style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.TextSecondary),
                        modifier = Modifier.padding(bottom = 8.dp)
                    )
                }
                items(searchResults) { track ->
                    TrackRow(
                        track = track,
                        isPlaying = uiState.currentTrack?.id == track.id && uiState.isPlaying,
                        onClick = { viewModel.playTrack(track, searchResults) },
                        onLikeClick = { viewModel.toggleLike(track) },
                        onMuteClick = { viewModel.muteTrack(track) }
                    )
                }
            }
        } else if (query.length >= 2) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(140.dp),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = "No studio master tracks found for \"$query\"",
                    style = SoftifyTypography.Caption
                )
            }
        } else {
            // Empty state hints
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(200.dp)
                    .clip(RoundedCornerShape(16.dp))
                    .background(SoftifyColors.Surface2)
                    .padding(24.dp),
                contentAlignment = Alignment.Center
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text("🎧", style = SoftifyTypography.DisplayHero)
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "Type any song title, artist, or paste a YouTube / Spotify link",
                        style = SoftifyTypography.Caption
                    )
                }
            }
        }
    }
}
