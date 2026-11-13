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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import com.softify.ui.components.EqualizerFader
import com.softify.ui.state.PlayerViewModel
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography

/**
 * Screen 7: 10-Band Studio Parametric Equalizer
 */
@Composable
fun EqualizerScreen(
    viewModel: PlayerViewModel,
    modifier: Modifier = Modifier
) {
    val uiState by viewModel.uiState.collectAsState()
    val frequencyLabels = listOf("32Hz", "64Hz", "125Hz", "250Hz", "500Hz", "1kHz", "2kHz", "4kHz", "8kHz", "16kHz")
    val presets = listOf("Flat", "Bass Boost", "Acoustic", "Vocal Polish", "Electronic")

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 24.dp)
    ) {
        Spacer(modifier = Modifier.height(24.dp))

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column {
                Text(text = "10-Band Studio Equalizer", style = SoftifyTypography.DisplayHero)
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = "Parametric Acoustic Shaping · Active Preset: ${uiState.equalizerPreset}",
                    style = SoftifyTypography.Caption.copy(color = SoftifyColors.AccentStudio)
                )
            }

            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(999.dp))
                    .background(SoftifyColors.Surface3)
                    .clickable { viewModel.setEqualizerPreset("Flat") }
                    .padding(horizontal = 14.dp, vertical = 8.dp)
            ) {
                Text("Reset to Flat", style = SoftifyTypography.CaptionBold)
            }
        }

        Spacer(modifier = Modifier.height(20.dp))

        // Preset selector pills
        Row(
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            modifier = Modifier.fillMaxWidth()
        ) {
            presets.forEach { preset ->
                val isSelected = uiState.equalizerPreset == preset
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(999.dp))
                        .background(if (isSelected) SoftifyColors.AccentPrimary else SoftifyColors.Surface2)
                        .border(1.dp, if (isSelected) SoftifyColors.AccentPrimary else SoftifyColors.GlassHighlight, RoundedCornerShape(999.dp))
                        .clickable { viewModel.setEqualizerPreset(preset) }
                        .padding(horizontal = 14.dp, vertical = 8.dp)
                ) {
                    Text(
                        text = preset,
                        style = SoftifyTypography.CaptionBold.copy(
                            color = if (isSelected) SoftifyColors.TextPrimary else SoftifyColors.TextSecondary
                        )
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(24.dp))

        // Preamp Slider Container
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(14.dp))
                .background(SoftifyColors.Surface2)
                .padding(horizontal = 16.dp, vertical = 12.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text("MASTER PREAMP GAIN", style = SoftifyTypography.CaptionBold)
            Text(
                text = if (uiState.preampDb > 0) "+%.1f dB".format(uiState.preampDb) else "%.1f dB".format(uiState.preampDb),
                style = SoftifyTypography.CodeMeta.copy(color = SoftifyColors.AccentStudio)
            )
        }

        Spacer(modifier = Modifier.height(24.dp))

        // 10-Band Vertical Faders Container
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(20.dp))
                .background(SoftifyColors.Surface1)
                .border(1.dp, SoftifyColors.GlassHighlight, RoundedCornerShape(20.dp))
                .padding(horizontal = 16.dp, vertical = 24.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                frequencyLabels.forEachIndexed { index, freq ->
                    val gain = uiState.equalizerBands.getOrElse(index) { 0f }
                    EqualizerFader(
                        frequencyLabel = freq,
                        gainDb = gain,
                        onGainChanged = { newGain ->
                            viewModel.setEqualizerBand(index, newGain)
                        }
                    )
                }
            }
        }
    }
}
