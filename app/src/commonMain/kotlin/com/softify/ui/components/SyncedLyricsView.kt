package com.softify.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import com.softify.ui.theme.SoftifyColors
import com.softify.ui.theme.SoftifyTypography
import uniffi.softify_models.Lyrics

/**
 * Parsed synchronized lyric line
 */
data class SyncedLine(
    val timeMs: Long,
    val text: String
)

/**
 * Parses LRC format strings into structured SyncedLine list.
 */
fun parseLrc(lrcText: String?): List<SyncedLine> {
    if (lrcText.isNullOrEmpty()) return emptyList()
    val lines = mutableListOf<SyncedLine>()
    val regex = Regex("\\[(\\d{2}):(\\d{2})\\.(\\d{2,3})\\](.*)")

    lrcText.lines().forEach { line ->
        val match = regex.find(line.trim())
        if (match != null) {
            val min = match.groupValues[1].toLongOrNull() ?: 0L
            val sec = match.groupValues[2].toLongOrNull() ?: 0L
            val msPart = match.groupValues[3]
            val ms = if (msPart.length == 2) msPart.toLong() * 10 else msPart.toLong()
            val totalMs = (min * 60 + sec) * 1000 + ms
            val text = match.groupValues[4].trim()
            if (text.isNotEmpty()) {
                lines.add(SyncedLine(totalMs, text))
            }
        }
    }
    return lines.sortedBy { it.timeMs }
}

/**
 * Real-time Karaoke Synchronized Lyrics Component
 */
@Composable
fun SyncedLyricsView(
    lyrics: Lyrics?,
    currentPositionMs: Long,
    onSeekToTimestamp: (Long) -> Unit,
    modifier: Modifier = Modifier
) {
    if (lyrics == null || (lyrics.plain.isNullOrEmpty() && lyrics.synced.isNullOrEmpty())) {
        Box(
            modifier = modifier.fillMaxSize(),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = "No lyrics available for this studio recording",
                style = SoftifyTypography.BodyPrimary.copy(color = SoftifyColors.TextSecondary)
            )
        }
        return
    }

    val syncedLines = parseLrc(lyrics.synced)
    val listState = rememberLazyListState()

    // Determine current active lyric line index
    val activeIndex = syncedLines.indexOfLast { it.timeMs <= currentPositionMs }

    // Auto-scroll to center active line
    LaunchedEffect(activeIndex) {
        if (activeIndex >= 0) {
            listState.animateScrollToItem((activeIndex - 2).coerceAtLeast(0))
        }
    }

    if (syncedLines.isNotEmpty()) {
        LazyColumn(
            state = listState,
            modifier = modifier
                .fillMaxSize()
                .padding(horizontal = 24.dp)
        ) {
            item { Spacer(modifier = Modifier.height(60.dp)) }

            itemsIndexed(syncedLines) { index, line ->
                val isActive = index == activeIndex
                Text(
                    text = line.text,
                    style = if (isActive) SoftifyTypography.LyricsActive else SoftifyTypography.LyricsIdle,
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .clickable { onSeekToTimestamp(line.timeMs) }
                        .padding(vertical = 12.dp)
                )
            }

            item { Spacer(modifier = Modifier.height(120.dp)) }
        }
    } else {
        // Plain lyrics fallback with Romanized indicator
        LazyColumn(
            modifier = modifier
                .fillMaxSize()
                .padding(24.dp)
        ) {
            if (lyrics.hasRomanized) {
                item {
                    Text(
                        text = "✨ Romanized Lyrics Active",
                        style = SoftifyTypography.CaptionBold.copy(color = SoftifyColors.AccentStudio),
                        modifier = Modifier.padding(bottom = 16.dp)
                    )
                }
            }
            item {
                Text(
                    text = lyrics.plain ?: "",
                    style = SoftifyTypography.BodyPrimary,
                    lineHeight = SoftifyTypography.LyricsIdle.lineHeight
                )
            }
        }
    }
}
