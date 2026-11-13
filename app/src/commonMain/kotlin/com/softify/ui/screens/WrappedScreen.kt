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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.softify.ui.components.AudioFidelityBadge
import com.softify.ui.state.PlayerViewModel
import com.softify.ui.state.SoftifyEngineBridge
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography
import uniffi.softify_models.WrappedStats

/**
 * Screen 6: Softify Wrapped & Audio Persona
 */
@Composable
fun WrappedScreen(
    viewModel: PlayerViewModel,
    modifier: Modifier = Modifier
) {
    var stats by remember { mutableStateOf<WrappedStats?>(null) }
    var currentCardIndex by remember { mutableStateOf(0) }

    LaunchedEffect(Unit) {
        stats = SoftifyEngineBridge.getWrappedStats()
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        // Story Progress Indicators
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            for (i in 0 until 4) {
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .height(3.dp)
                        .clip(RoundedCornerShape(2.dp))
                        .background(if (i <= currentCardIndex) SoftifyColors.TextPrimary else SoftifyColors.Surface3)
                        .clickable { currentCardIndex = i }
                )
            }
        }

        Spacer(modifier = Modifier.height(20.dp))

        // Main Wrapped Presentation Glass Card
        Box(
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth()
                .clip(RoundedCornerShape(24.dp))
                .background(SoftifyColors.Surface1)
                .border(1.dp, SoftifyColors.GlassHighlight, RoundedCornerShape(24.dp))
                .padding(32.dp),
            contentAlignment = Alignment.Center
        ) {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center
            ) {
                when (currentCardIndex) {
                    0 -> {
                        Text("SOFTIFY WRAPPED 2026", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.AccentStudio))
                        Spacer(modifier = Modifier.height(14.dp))
                        Text("Total Minutes Streamed", style = SoftifyTypography.TitleMedium)
                        Spacer(modifier = Modifier.height(8.dp))
                        val totalMins = stats?.totalMinutesListened ?: 42850u
                        Text(
                            text = "%,d".format(totalMins.toLong()),
                            style = SoftifyTypography.DisplayHero.copy(color = SoftifyColors.AccentPrimary)
                        )
                        Spacer(modifier = Modifier.height(12.dp))
                        Text(
                            text = "That's 714 hours of pure uncompressed studio acoustics.",
                            style = SoftifyTypography.BodyPrimary,
                            textAlign = TextAlign.Center
                        )
                    }

                    1 -> {
                        Text("STUDIO FIDELITY RATIO", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.AccentStudio))
                        Spacer(modifier = Modifier.height(14.dp))
                        Text("99.4%", style = SoftifyTypography.DisplayHero.copy(color = SoftifyColors.AccentStudio))
                        Spacer(modifier = Modifier.height(8.dp))
                        Text("320 Kbps Studio Master AAC", style = SoftifyTypography.TitleMedium)
                        Spacer(modifier = Modifier.height(12.dp))
                        Text(
                            text = "Zero low-bitrate streams allowed. Bit-perfect studio acoustics.",
                            style = SoftifyTypography.Caption,
                            textAlign = TextAlign.Center
                        )
                    }

                    2 -> {
                        Text("TOP ARTISTS", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.AccentStudio))
                        Spacer(modifier = Modifier.height(14.dp))
                        val topArtists = stats?.topArtists ?: listOf("The Weeknd", "Arijit Singh", "Hans Zimmer", "Daft Punk", "Taylor Swift")
                        topArtists.take(5).forEachIndexed { idx, artist ->
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth(0.8f)
                                    .padding(vertical = 4.dp),
                                horizontalArrangement = Arrangement.SpaceBetween
                            ) {
                                Text("#${idx + 1}  $artist", style = SoftifyTypography.BodyEmphasis)
                                AudioFidelityBadge()
                            }
                        }
                    }

                    else -> {
                        Text("YOUR AUDIO PERSONA", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.AccentStudio))
                        Spacer(modifier = Modifier.height(14.dp))
                        Box(
                            modifier = Modifier
                                .size(72.dp)
                                .clip(CircleShape)
                                .background(SoftifyColors.AccentStudio.copy(alpha = 0.15f)),
                            contentAlignment = Alignment.Center
                        ) {
                            Text("🔮", style = SoftifyTypography.DisplayHero)
                        }
                        Spacer(modifier = Modifier.height(14.dp))
                        Text("THE NIGHT CYCLIST", style = SoftifyTypography.TitleLarge)
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "You thrive when the world sleeps. 68% of your listening happens between 11 PM and 4 AM with melodic synthwave and deep bass acoustics.",
                            style = SoftifyTypography.BodyPrimary,
                            textAlign = TextAlign.Center
                        )
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(20.dp))

        // Next Card and Share actions
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(999.dp))
                    .background(SoftifyColors.Surface3)
                    .clickable {
                        currentCardIndex = (currentCardIndex + 1) % 4
                    }
                    .padding(horizontal = 20.dp, vertical = 10.dp)
            ) {
                Text("Next Card →", style = SoftifyTypography.CaptionBold)
            }

            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(999.dp))
                    .background(SoftifyColors.AccentPrimary)
                    .clickable { /* Share Card */ }
                    .padding(horizontal = 20.dp, vertical = 10.dp)
            ) {
                Text("📤 Share Persona", style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.TextPrimary))
            }
        }
    }
}
