# AGENTS.md — Softify Next-Gen Engineering Specification

> **MANDATORY INSTRUCTION FOR ALL AI AGENTS & ENGINEERS**:
> 1. **STRICT ISOLATION PROTOCOL**: You are **STRICTLY FORBIDDEN** from accessing, reading, or copying from the legacy `/softify` directory. Doing so will import legacy bugs, dirty code, and antipatterns.
> 2. **SELF-CONTAINED SPECIFICATION**: Everything you need—all API endpoints, DES decryption ciphers, 320 Kbps stream rewriting logic, candidate scoring math, and data schemas—is 100% fully documented in this file.
> 3. **320 KBPS AUDIO FIDELITY FIRST**: Softify Next-Gen guarantees pure studio-grade **320 Kbps AAC / High-Bitrate Audio** across all streaming and offline download pipelines.
> 4. **BACKEND-FIRST FOCUS (CRITICAL)**: **DO NOT WRITE ANY FRONTEND OR UI CODE AT THIS STAGE.** All immediate effort must be 100% focused on building the **Rust Core Engine** (`softify-core`), its cryptographic decryptors, the 320 Kbps stream resolvers, SQLite FTS5 database, the UniFFI bridge, and the `softify-cli` terminal runner. Frontend (Kotlin Compose Multiplatform) will be built later only after the backend engine is fully tested and verified via `cargo test` and CLI.
> 5. **OFFLOAD BUILDS & TESTS TO GITHUB CI**: The local development machine has constrained resources. Avoid heavy local release builds or repeated local test compilation loops. Make atomic git commits and push to GitHub (`main`) so GitHub Actions (`.github/workflows/ci.yml`) compiles and executes the test suites on high-performance cloud runners. Check CI status via `gh run list` / `gh run view`.
> 6. **ZERO TEMPORARY .MD COMMITS (STRICT)**: **NEVER COMMIT OR PUSH TEMPORARY .MD FILES TO GITHUB.** Do not commit prompt files (`INITIAL_PROMPT.md`), scratchpads, meeting logs, review notes, or draft summaries. The Git history and repository must stay completely clean and professional. Only permanent architectural specifications (`README.md`, `AGENTS.md`, and official docs) are permitted in Git.

---

## 1. Project Mission & High-Level Architecture

Softify Next-Gen is an ultra-fast, minimalist, ad-free music streaming app built with:
* **Core Engine (Pure Rust)**: Handles all network IO, encryption/decryption, reverse-engineered catalog engines, SQLite FTS5 search, and stream resolution.
* **FFI Bridge (Mozilla UniFFI)**: Generates type-safe Kotlin bindings with zero manual JNI boilerplate.
* **UI & Platform Layer (Kotlin Compose Multiplatform)**:
  * **Android**: Jetpack Compose UI + native AndroidX `Media3` (`ExoPlayer`) with background `MediaSessionService`, lockscreen controls, and notification audio channels.
  * **Desktop (Linux/macOS/Windows)**: Compose for Desktop + Linux MPRIS D-Bus player controls.

```
┌───────────────────────────────────────────────────────────────────┐
│              KOTLIN COMPOSE MULTIPLATFORM (UI & OS)               │
│  • Android: Jetpack Compose + AndroidX Media3 (ExoPlayer)         │
│  • Desktop: Compose for Desktop + Linux MPRIS D-Bus Engine        │
└─────────────────────────────────▲─────────────────────────────────┘
                                  │ UniFFI Bridge (Type-safe Rust/Kotlin)
┌─────────────────────────────────▼─────────────────────────────────┐
│                       RUST CORE ENGINE                            │
│  • JioSaavn 320 Kbps Studio Master Resolver (DES-ECB Decryptor)   │
│  • Spotify Pathfinder GraphQL & Anonymous Session Engine          │
│  • YouTube Music InnerTube (WEB_REMIX) Resolver                   │
│  • SQLite + FTS5 Catalog & Playback History Index                 │
└───────────────────────────────────────────────────────────────────┘
```

---

## 2. 320 Kbps Studio Audio & Clean Engine Specifications

### Engine 1: JioSaavn Studio Audio Master (Primary: 320 Kbps CD-Quality AAC)
Provides original studio master audio recordings with zero movie dialogue, zero sound effects, and direct CDN streaming.

* **Search Endpoint**:
  ```http
  GET https://www.jiosaavn.com/api.php?__call=search.getResults&_format=json&_marker=0&api_version=4&ctx=web6dot0&n=15&p=1&q={QUERY}
  User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36
  ```

* **320 Kbps DES Decryption Algorithm**:
  1. Extract `more_info.encrypted_media_url` from the song payload.
  2. Base64-decode the encrypted string into raw bytes.
  3. Decrypt using **DES (Data Encryption Standard)**:
     * **Cipher Key**: `38346591` (ASCII bytes: `[51, 56, 51, 52, 54, 53, 57, 49]`)
     * **Mode**: ECB (Electronic Codebook)
     * **Padding**: PKCS5 / PKCS7
  4. UTF-8 decode the decrypted bytes to get the raw stream URL (typically ending in `_96.mp4` or `_160.mp4`).
  5. **The 320 Kbps Upgrade Rewrite**:
     ```rust
     // Replace lower bitrates with 320kbps studio master
     let stream_url_320 = raw_decrypted_url
         .replace("_96.mp4", "_320.mp4")
         .replace("_160.mp4", "_320.mp4");
     ```
  6. Return `StreamInfo` with container `'m4a'`, codec `'aac'`, bitrate `320000`.

* **Candidate Filtering & Verification Rules**:
  * **Artist Extraction**: Collect artists from `more_info.artistMap.primary_artists` and `more_info.artistMap.featured_artists`. If empty (common for Western/English songs), extract the artist prefix from `subtitle` before ` - ` (e.g. `"Blackway, Black Caviar - Gym Rat"` $\to$ `"Blackway, Black Caviar"`).
  * **Actor Disqualification**: Discard any entry in `artistMap.artists` where `role == "starring"`.
  * **Variant Disqualification**: Disqualify candidates containing `remix`, `mashup`, `dj`, `lofi`, `slowed`, `reverb`, `acoustic`, `cover`, `karaoke` unless requested by the user query.
  * **Duration Proximity**: Candidates within $\pm 6\text{s}$ of the official track duration receive a $+40$ point bonus. Reject candidates with duration discrepancies $> 90\text{s}$.

---

### Engine 2: Spotify Pathfinder & Anonymous Authentication
Provides the primary metadata, playlist discovery, and album artwork engine.

* **Anonymous Token Generation**:
  ```http
  GET https://open.spotify.com/get_access_token?reason=transport&productType=web_player
  User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36
  ```
  * Response JSON contains: `clientId`, `accessToken`, and `accessTokenExpirationTimestampMs`.
  * Cache token in memory; automatically refresh 5 minutes before expiration.

* **Pathfinder Search Endpoint**:
  ```http
  GET https://api-partner.spotify.com/pathfinder/v1/query?operationName=searchDesktop&variables={"searchTerm":"{QUERY}","offset":0,"limit":20,"numberOfTopResults":5}&extensions={"persistedQuery":{"version":1,"sha256Hash":"12c1a6f114c0a50157303c72b8314ff5a7f2924fa68ff212e3e53e4a9e52ce64"}}
  Authorization: Bearer {ACCESS_TOKEN}
  ```

* **Unicode Non-Breaking Space (`\u00A0`) Normalization Law**:
  * Spotify metadata frequently encodes multi-artist separators as `,\u00A0` (byte `0xC2 0xA0`).
  * **MANDATORY**: Always replace `\u00A0` and all Unicode whitespace (`\u2000`–`\u200B`, `\u202F`, `\u3000`) with standard ASCII space `' '` before tokenizing. Never strip punctuation without space replacement!

---

### Engine 3: YouTube Music InnerTube (`WEB_REMIX`) (Secondary Fallback)
Serves as verified fallback for rare tracks not present in the JioSaavn catalog.

* **Search Endpoint**:
  ```http
  POST https://music.youtube.com/youtubei/v1/search?prettyPrint=false
  Content-Type: application/json
  User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36
  Referer: https://music.youtube.com/
  ```

* **JSON Body**:
  ```json
  {
    "context": {
      "client": {
        "clientName": "WEB_REMIX",
        "clientVersion": "1.20240422.01.00",
        "hl": "en",
        "gl": "US"
      }
    },
    "query": "{TITLE} {ARTIST}",
    "params": "EgWKAQIIAWoQEAMQBBAJEAoQBRAREBAQFQ%3D%3D"
  }
  ```

* **Topic Channel Prioritization**:
  * Authors ending with `- Topic` (e.g. `The Weeknd - Topic`) represent official record-label CD audio uploads. Award $+80$ confidence points.

---

## 3. Rust Workspace Directory Structure

All Rust code lives strictly under `core/`:

```
core/
├── Cargo.toml                         # Workspace manifest
├── softify-core/                      # Shared Native Library
│   ├── Cargo.toml
│   ├── uniffi.toml                    # UniFFI interface configuration
│   └── src/
│       ├── lib.rs                     # UniFFI interface definitions
│       ├── models.rs                  # Track, StreamInfo, SearchResult, Quality
│       ├── crypto/
│       │   └── des.rs                 # Clean DES-ECB decryptor for JioSaavn
│       ├── resolvers/
│       │   ├── mod.rs                 # StreamResolver trait
│       │   ├── jiosaavn.rs            # JioSaavn 320 Kbps studio resolver
│       │   ├── spotify.rs             # Spotify Pathfinder & anonymous token
│       │   ├── youtube.rs             # YTM InnerTube WEB_REMIX resolver
│       │   └── hybrid.rs              # Smart fallback coordinator
│       ├── normalizer.rs              # Unicode space & string normalization
│       └── storage/
│           └── db.rs                  # rusqlite + FTS5 local cache
└── softify-cli/                       # Terminal CLI for Instant Verification
    ├── Cargo.toml
    └── src/
        └── main.rs                    # Test resolution & playback directly in terminal
```

---

## 4. UniFFI Interface Contract (Rust to Kotlin)

The bridge between Rust and Kotlin must remain lean and type-safe. Define high-level service interfaces rather than micro-calls:

```rust
// Core models exposed to Kotlin via UniFFI
pub struct TrackMetadata {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_seconds: u32,
    pub cover_url: Option<String>,
}

pub struct StreamResult {
    pub stream_url: String,
    pub bitrate: u32,             // 320000 for JioSaavn Studio Master
    pub container: String,        // "m4a"
    pub codec: String,            // "aac"
    pub provider: String,         // "jiosaavn_studio_320k"
}

// Service trait exposed to Kotlin
pub trait SoftifyEngine: Send + Sync {
    fn search(&self, query: String) -> Vec<TrackMetadata>;
    fn resolve_stream(&self, track: TrackMetadata) -> Result<StreamResult, String>;
    fn get_lyrics(&self, title: String, artist: String) -> Option<String>;
}
```

---

## 5. Architectural Invariants (Never Violate)

1. **NO ACCESS TO OLD REPO**: Never inspect or copy code from `/home/sarthak/Desktop/Projects/softify`.
2. **NO DART**: No `.dart` files or Flutter dependencies anywhere in this repository.
3. **AUDIO FIDELITY**: Every JioSaavn stream URL must be rewritten to `_320.mp4` for CD-quality 320 Kbps streaming.
4. **DECOUPLED AUDIO OUTPUT**:
   * On **Android**: Pass the resolved 320 Kbps stream URL directly to AndroidX `Media3` (`ExoPlayer`). Do NOT build a software audio decoder in Rust for Android.
   * On **Desktop**: Pass the stream to `libmpv` or `VLC` native bindings with MPRIS D-Bus integration.
5. **TEST-FIRST IMPLEMENTATION**: Every module in `softify-core` must have unit tests executable via `cargo test`.
6. **NO TEMPORARY MARKDOWN IN GIT**: Never commit scratchpads, prompt files, or temporary `.md` files to GitHub. Only permanent specs (`README.md`, `AGENTS.md`) belong in version control.
