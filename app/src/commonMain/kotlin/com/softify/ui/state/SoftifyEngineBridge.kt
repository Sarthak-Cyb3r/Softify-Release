package com.softify.ui.state

import com.softify.core.SoftifyEngineService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.softify_models.Lyrics
import uniffi.softify_models.Mix
import uniffi.softify_models.Mood
import uniffi.softify_models.StreamResult
import uniffi.softify_models.TrackMetadata
import uniffi.softify_models.UpdateInfo
import uniffi.softify_models.WrappedStats
import java.io.File

/**
 * Thread-safe Bridge between Kotlin Multiplatform UI and Rust Core Engine.
 */
object SoftifyEngineBridge {
    private var engineInstance: SoftifyEngineService? = null

    fun getEngine(): SoftifyEngineService {
        return engineInstance ?: synchronized(this) {
            engineInstance ?: run {
                val dbDir = File(System.getProperty("user.home"), ".softify")
                if (!dbDir.exists()) dbDir.mkdirs()
                val dbPath = File(dbDir, "softify.db").absolutePath

                try {
                    // Try to load libsoftify_core from local target or system library path
                    val libPath = System.getProperty("softify.lib.path") ?: "dist/linux/softify-linux-x86_64/libsoftify_core.so"
                    val file = File(libPath)
                    if (file.exists()) {
                        System.setProperty("jna.library.path", file.parentFile.absolutePath)
                    }
                } catch (_: Exception) {}

                try {
                    SoftifyEngineService(dbPath)
                } catch (e: Exception) {
                    // Fallback to in-memory if disk db fails
                    SoftifyEngineService(null)
                }
            }.also { engineInstance = it }
        }
    }

    suspend fun search(query: String): List<TrackMetadata> = withContext(Dispatchers.IO) {
        try {
            getEngine().search(query)
        } catch (e: Exception) {
            emptyList()
        }
    }

    suspend fun searchCache(query: String, limit: UInt = 50u): List<TrackMetadata> = withContext(Dispatchers.IO) {
        try {
            getEngine().searchCache(query, limit)
        } catch (e: Exception) {
            emptyList()
        }
    }

    suspend fun resolveStream(track: TrackMetadata): Result<StreamResult> = withContext(Dispatchers.IO) {
        try {
            Result.success(getEngine().resolveStream(track))
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun getDailyMixes(): List<Mix> = withContext(Dispatchers.IO) {
        try {
            getEngine().getDailyMixes()
        } catch (e: Exception) {
            emptyList()
        }
    }

    suspend fun getMoodMix(mood: Mood): Mix? = withContext(Dispatchers.IO) {
        try {
            getEngine().getMoodMix(mood)
        } catch (e: Exception) {
            null
        }
    }

    suspend fun getWeeklyDiscovery(): Mix? = withContext(Dispatchers.IO) {
        try {
            getEngine().getWeeklyDiscovery()
        } catch (e: Exception) {
            null
        }
    }

    suspend fun getPlaybackHistory(limit: UInt = 20u): List<TrackMetadata> = withContext(Dispatchers.IO) {
        try {
            getEngine().getPlaybackHistory(limit)
        } catch (e: Exception) {
            emptyList()
        }
    }

    suspend fun getLyrics(title: String, artist: String): Lyrics? = withContext(Dispatchers.IO) {
        try {
            getEngine().getLyricsContainer(title, artist)
        } catch (e: Exception) {
            null
        }
    }

    suspend fun getWrappedStats(): WrappedStats? = withContext(Dispatchers.IO) {
        try {
            getEngine().getWrappedStats()
        } catch (e: Exception) {
            null
        }
    }

    suspend fun checkForUpdates(currentVersion: String = "2.0.0"): UpdateInfo? = withContext(Dispatchers.IO) {
        try {
            getEngine().checkForUpdates(currentVersion)
        } catch (e: Exception) {
            null
        }
    }

    suspend fun downloadTrack(track: TrackMetadata): Result<String> = withContext(Dispatchers.IO) {
        try {
            val destDir = File(System.getProperty("user.home"), ".softify/downloads")
            if (!destDir.exists()) destDir.mkdirs()
            val path = getEngine().downloadTrack(track, destDir.absolutePath)
            Result.success(path)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
