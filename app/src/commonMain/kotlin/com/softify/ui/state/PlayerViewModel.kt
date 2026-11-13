package com.softify.ui.state

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import uniffi.softify_models.Lyrics
import uniffi.softify_models.Mix
import uniffi.softify_models.Mood
import uniffi.softify_models.StreamResult
import uniffi.softify_models.TrackMetadata

enum class NavigationTab {
    HOME,
    SEARCH,
    PLAYER,
    LIBRARY,
    IMPORT,
    WRAPPED,
    EQUALIZER,
    SETTINGS
}

data class PlayerUiState(
    val currentTrack: TrackMetadata? = null,
    val currentStream: StreamResult? = null,
    val isPlaying: Boolean = false,
    val positionMs: Long = 0L,
    val durationMs: Long = 0L,
    val lyrics: Lyrics? = null,
    val queue: List<TrackMetadata> = emptyList(),
    val queueIndex: Int = 0,
    val likedTracks: List<TrackMetadata> = emptyList(),
    val mutedTrackIds: Set<String> = emptySet(),
    val dailyMixes: List<Mix> = emptyList(),
    val historyTracks: List<TrackMetadata> = emptyList(),
    val currentTab: NavigationTab = NavigationTab.HOME,
    val isShuffle: Boolean = false,
    val isRepeat: Boolean = false,
    val preampDb: Float = 0f,
    val equalizerBands: List<Float> = listOf(0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f),
    val equalizerPreset: String = "Flat"
)

class PlayerViewModel(
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.Main)
) {
    private val _uiState = MutableStateFlow(PlayerUiState())
    val uiState: StateFlow<PlayerUiState> = _uiState.asStateFlow()

    private var playbackTickerJob: Job? = null

    init {
        loadInitialData()
    }

    fun setTab(tab: NavigationTab) {
        _uiState.value = _uiState.value.copy(currentTab = tab)
    }

    fun loadInitialData() {
        scope.launch {
            val mixes = SoftifyEngineBridge.getDailyMixes()
            val history = SoftifyEngineBridge.getPlaybackHistory(20u)
            _uiState.value = _uiState.value.copy(
                dailyMixes = mixes,
                historyTracks = history
            )
        }
    }

    fun playTrack(track: TrackMetadata, queue: List<TrackMetadata> = listOf(track)) {
        scope.launch {
            // Check if track is muted
            if (_uiState.value.mutedTrackIds.contains(track.id)) {
                return@launch
            }

            val queueIndex = queue.indexOfFirst { it.id == track.id }.coerceAtLeast(0)
            _uiState.value = _uiState.value.copy(
                currentTrack = track,
                queue = queue,
                queueIndex = queueIndex,
                positionMs = 0L,
                durationMs = track.durationSeconds.toLong() * 1000L,
                isPlaying = true
            )

            // Resolve 320 Kbps stream in background
            val streamRes = SoftifyEngineBridge.resolveStream(track)
            if (streamRes.isSuccess) {
                _uiState.value = _uiState.value.copy(currentStream = streamRes.getOrNull())
            }

            // Fetch synchronized / Romanized lyrics
            val lyrics = SoftifyEngineBridge.getLyrics(track.title, track.artist)
            _uiState.value = _uiState.value.copy(lyrics = lyrics)

            startPlaybackTicker()
        }
    }

    fun togglePlayPause() {
        val willPlay = !_uiState.value.isPlaying
        _uiState.value = _uiState.value.copy(isPlaying = willPlay)
        if (willPlay) {
            startPlaybackTicker()
        } else {
            playbackTickerJob?.cancel()
        }
    }

    fun seekTo(positionMs: Long) {
        val bounded = positionMs.coerceIn(0L, _uiState.value.durationMs)
        _uiState.value = _uiState.value.copy(positionMs = bounded)
    }

    fun nextTrack() {
        val state = _uiState.value
        if (state.queue.isNotEmpty()) {
            val nextIdx = if (state.isShuffle) {
                state.queue.indices.random()
            } else {
                (state.queueIndex + 1) % state.queue.size
            }
            playTrack(state.queue[nextIdx], state.queue)
        }
    }

    fun previousTrack() {
        val state = _uiState.value
        if (state.queue.isNotEmpty()) {
            val prevIdx = if (state.queueIndex > 0) state.queueIndex - 1 else state.queue.size - 1
            playTrack(state.queue[prevIdx], state.queue)
        }
    }

    fun toggleLike(track: TrackMetadata) {
        val currentLikes = _uiState.value.likedTracks.toMutableList()
        val exists = currentLikes.any { it.id == track.id }
        if (exists) {
            currentLikes.removeAll { it.id == track.id }
        } else {
            currentLikes.add(0, track)
        }
        _uiState.value = _uiState.value.copy(likedTracks = currentLikes)
    }

    fun muteTrack(track: TrackMetadata) {
        val mutes = _uiState.value.mutedTrackIds.toMutableSet()
        mutes.add(track.id)
        _uiState.value = _uiState.value.copy(mutedTrackIds = mutes)

        // If currently playing track was muted, skip to next track immediately
        if (_uiState.value.currentTrack?.id == track.id) {
            nextTrack()
        }
    }

    fun unblockTrack(trackId: String) {
        val mutes = _uiState.value.mutedTrackIds.toMutableSet()
        mutes.remove(trackId)
        _uiState.value = _uiState.value.copy(mutedTrackIds = mutes)
    }

    fun setEqualizerBand(index: Int, gainDb: Float) {
        val currentBands = _uiState.value.equalizerBands.toMutableList()
        if (index in currentBands.indices) {
            currentBands[index] = gainDb.coerceIn(-12f, 12f)
            _uiState.value = _uiState.value.copy(
                equalizerBands = currentBands,
                equalizerPreset = "Custom"
            )
        }
    }

    fun setEqualizerPreset(presetName: String) {
        val bands = when (presetName) {
            "Bass Boost" -> listOf(6.0f, 5.0f, 3.5f, 1.5f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f, 2.0f)
            "Acoustic" -> listOf(3.5f, 3.0f, 2.0f, 1.0f, 1.5f, 1.5f, 2.5f, 3.5f, 3.0f, 2.0f)
            "Vocal Polish" -> listOf(-1.5f, -1.0f, 0.0f, 1.5f, 3.5f, 4.0f, 3.5f, 2.0f, 1.0f, 0.5f)
            "Electronic" -> listOf(5.0f, 4.0f, 2.0f, 0.0f, -1.0f, 2.0f, 1.0f, 2.5f, 4.5f, 5.0f)
            else -> listOf(0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f)
        }
        _uiState.value = _uiState.value.copy(
            equalizerPreset = presetName,
            equalizerBands = bands
        )
    }

    fun setPreamp(preampDb: Float) {
        _uiState.value = _uiState.value.copy(preampDb = preampDb.coerceIn(-12f, 12f))
    }

    private fun startPlaybackTicker() {
        playbackTickerJob?.cancel()
        playbackTickerJob = scope.launch {
            while (isActive && _uiState.value.isPlaying) {
                delay(250)
                val curPos = _uiState.value.positionMs + 250
                if (curPos >= _uiState.value.durationMs && _uiState.value.durationMs > 0) {
                    if (_uiState.value.isRepeat) {
                        seekTo(0L)
                    } else {
                        nextTrack()
                    }
                } else {
                    _uiState.value = _uiState.value.copy(positionMs = curPos)
                }
            }
        }
    }
}
