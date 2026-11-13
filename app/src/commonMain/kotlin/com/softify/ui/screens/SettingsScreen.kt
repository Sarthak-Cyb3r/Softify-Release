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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
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
import com.softify.ui.state.PlayerViewModel
import com.softify.ui.state.SoftifyEngineBridge
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography
import kotlinx.coroutines.launch
import uniffi.softify_models.UpdateInfo

/**
 * Screen 8: Settings, Storage Cleaner & GitHub Updater
 */
@Composable
fun SettingsScreen(
    viewModel: PlayerViewModel,
    modifier: Modifier = Modifier
) {
    val uiState by viewModel.uiState.collectAsState()
    val scope = rememberCoroutineScope()
    var updateInfo by remember { mutableStateOf<UpdateInfo?>(null) }
    var isCheckingUpdate by remember { mutableStateOf(false) }

    LazyColumn(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 24.dp),
        contentPadding = PaddingValues(bottom = 120.dp)
    ) {
        item {
            Spacer(modifier = Modifier.height(24.dp))
            Text(text = "System Settings", style = SoftifyTypography.DisplayHero)
            Spacer(modifier = Modifier.height(24.dp))
        }

        // Section 1: Audio Streaming Fidelity
        item {
            Text(text = "AUDIO STREAMING FIDELITY", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.TextSecondary))
            Spacer(modifier = Modifier.height(10.dp))

            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(14.dp))
                    .background(SoftifyColors.Surface2)
                    .border(1.dp, SoftifyColors.GlassHighlight, RoundedCornerShape(14.dp))
                    .padding(16.dp)
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column {
                        Text("Stream Quality Baseline", style = SoftifyTypography.BodyEmphasis)
                        Text("Guaranteed 320 Kbps studio master rewrite", style = SoftifyTypography.Caption)
                    }
                    AudioFidelityBadge()
                }

                Spacer(modifier = Modifier.height(14.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column {
                        Text("Decryption Engine", style = SoftifyTypography.BodyEmphasis)
                        Text("Native Rust DES-ECB hardware decryptor", style = SoftifyTypography.Caption)
                    }
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(6.dp))
                            .background(SoftifyColors.StatusOnline.copy(alpha = 0.15f))
                            .padding(horizontal = 8.dp, vertical = 4.dp)
                    ) {
                        Text("ACTIVE", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.StatusOnline))
                    }
                }
            }

            Spacer(modifier = Modifier.height(24.dp))
        }

        // Section 2: Muted Tracks Management ("Never Play Again")
        item {
            Text(text = "BLOCKED & MUTED TRACKS", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.TextSecondary))
            Spacer(modifier = Modifier.height(10.dp))

            if (uiState.mutedTrackIds.isNotEmpty()) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(14.dp))
                        .background(SoftifyColors.Surface2)
                        .padding(16.dp)
                ) {
                    Text(
                        text = "${uiState.mutedTrackIds.size} tracks blocked from all Daily Mixes & Auto-Play",
                        style = SoftifyTypography.Caption
                    )
                    Spacer(modifier = Modifier.height(10.dp))

                    uiState.mutedTrackIds.forEach { trackId ->
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(vertical = 4.dp),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text("Track #$trackId", style = SoftifyTypography.CodeMeta)
                            Box(
                                modifier = Modifier
                                    .clip(RoundedCornerShape(6.dp))
                                    .background(SoftifyColors.Surface3)
                                    .clickable { viewModel.unblockTrack(trackId) }
                                    .padding(horizontal = 10.dp, vertical = 4.dp)
                            ) {
                                Text("Unblock ✕", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.StatusDanger))
                            }
                        }
                    }
                }
            } else {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(14.dp))
                        .background(SoftifyColors.Surface2)
                        .padding(16.dp)
                ) {
                    Text("No tracks blocked. Songs you mark \"Never Play Again\" appear here.", style = SoftifyTypography.Caption)
                }
            }

            Spacer(modifier = Modifier.height(24.dp))
        }

        // Section 3: Storage & Cache Management
        item {
            Text(text = "STORAGE & LOCAL CACHE", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.TextSecondary))
            Spacer(modifier = Modifier.height(10.dp))

            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(14.dp))
                    .background(SoftifyColors.Surface2)
                    .padding(16.dp)
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column {
                        Text("Offline Audio Cache", style = SoftifyTypography.BodyEmphasis)
                        Text("1.42 GB · 96 Studio M4A Files", style = SoftifyTypography.Caption)
                    }
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(8.dp))
                            .background(SoftifyColors.Surface3)
                            .clickable { /* Purge cache */ }
                            .padding(horizontal = 12.dp, vertical = 6.dp)
                    ) {
                        Text("Clear Cache", style = SoftifyTypography.CaptionBold)
                    }
                }

                Spacer(modifier = Modifier.height(14.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column {
                        Text("SQLite FTS5 Catalog Index", style = SoftifyTypography.BodyEmphasis)
                        Text("12.8 MB database", style = SoftifyTypography.Caption)
                    }
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(8.dp))
                            .background(SoftifyColors.Surface3)
                            .clickable { /* Rebuild index */ }
                            .padding(horizontal = 12.dp, vertical = 6.dp)
                    ) {
                        Text("Rebuild Index", style = SoftifyTypography.CaptionBold)
                    }
                }
            }

            Spacer(modifier = Modifier.height(24.dp))
        }

        // Section 4: GitHub Releases Updater
        item {
            Text(text = "APP UPDATES & RELEASE", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.TextSecondary))
            Spacer(modifier = Modifier.height(10.dp))

            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(14.dp))
                    .background(SoftifyColors.Surface2)
                    .border(1.dp, SoftifyColors.GlassHighlight, RoundedCornerShape(14.dp))
                    .padding(16.dp)
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column {
                        Text("Installed Version: v2.0.0-STABLE", style = SoftifyTypography.BodyEmphasis)
                        Text("GitHub Releases Auto-Updater Channel", style = SoftifyTypography.Caption)
                    }
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(8.dp))
                            .background(SoftifyColors.AccentPrimary)
                            .clickable(enabled = !isCheckingUpdate) {
                                isCheckingUpdate = true
                                scope.launch {
                                    updateInfo = SoftifyEngineBridge.checkForUpdates("2.0.0")
                                    isCheckingUpdate = false
                                }
                            }
                            .padding(horizontal = 12.dp, vertical = 6.dp)
                    ) {
                        Text(
                            text = if (isCheckingUpdate) "Checking..." else "Check Update 🔄",
                            style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.TextPrimary)
                        )
                    }
                }

                if (updateInfo != null) {
                    Spacer(modifier = Modifier.height(12.dp))
                    if (updateInfo!!.hasUpdate) {
                        Text(
                            text = "New version available: ${updateInfo!!.latestVersion}",
                            style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.AccentStudio)
                        )
                    } else {
                        Text(
                            text = "✓ You are running the latest studio version.",
                            style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.StatusOnline)
                        )
                    }
                }
            }
        }
    }
}
