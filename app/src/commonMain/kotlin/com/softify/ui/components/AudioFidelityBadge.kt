package com.softify.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
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
import androidx.compose.ui.unit.dp
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography

/**
 * 320 Kbps Studio Master Audio Fidelity Badge
 * Emphasizes studio-grade CD quality AAC stream.
 */
@Composable
fun AudioFidelityBadge(
    modifier: Modifier = Modifier,
    bitrateText: String = "320 KBPS",
    codecText: String = "AAC"
) {
    Box(
        modifier = modifier
            .clip(RoundedCornerShape(6.dp))
            .background(SoftifyColors.AccentStudio.copy(alpha = 0.12f))
            .border(1.dp, SoftifyColors.AccentStudio.copy(alpha = 0.35f), RoundedCornerShape(6.dp))
            .padding(horizontal = 6.dp, vertical = 2.dp),
        contentAlignment = Alignment.Center
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Box(
                modifier = Modifier
                    .size(5.dp)
                    .clip(CircleShape)
                    .background(SoftifyColors.AccentStudio)
            )
            Spacer(modifier = Modifier.width(4.dp))
            Text(
                text = "$bitrateText · $codecText",
                style = SoftifyTypography.CodeMeta.copy(
                    color = SoftifyColors.AccentStudio
                )
            )
        }
    }
}
