# Softify Next-Gen (Release)

An ad-free, paywall-free, high-performance music streaming application built from scratch with **Rust Core** and **Kotlin Compose Multiplatform**.

## Key Highlights

* **320 Kbps High-Fidelity Audio**: Pure studio CD-quality 320 Kbps AAC audio streams across playback and offline downloads.
* **Pure Rust Core Engine**: Clean, high-throughput network engine, cryptographic decryptors, and search catalog with zero garbage-collection pauses.
* **Mozilla UniFFI Bridge**: Type-safe FFI interface bindings connecting Rust directly to Kotlin.
* **Native Platform Integration**:
  * **Android**: Jetpack Compose UI + native AndroidX `Media3` (`ExoPlayer`) background `MediaSessionService` (lockscreen, notification audio controls, Bluetooth routing).
  * **Desktop**: Jetpack Compose for Desktop + Linux MPRIS D-Bus integration.

## Clean Engines & Protocols

1. **JioSaavn Studio Master (Primary)**: DES-ECB encrypted media URL resolution with automated `_320.mp4` CD-quality studio stream extraction.
2. **Spotify Pathfinder**: Anonymous session minting and rich catalog search.
3. **YouTube Music InnerTube (`WEB_REMIX`)**: Topic-channel verified fallback for rare international releases.

## Developer & AI Engineering Guide

* **[AGENTS.md](file:///home/sarthak/Desktop/Projects/Softify%20Release/AGENTS.md)**: The definitive, self-contained architecture specification.
* **Strict Isolation Protocol**: All code in this repository is built fresh from the clean specifications documented in `AGENTS.md`. Accessing legacy repositories is forbidden to prevent code pollution and hallucinations.
