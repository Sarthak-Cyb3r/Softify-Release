# Softify Next-Gen — UI/UX Design Specification & Architecture Contract

> **Aesthetic Direction**: Apple Music macOS Sonoma / iOS 18 Liquid Glassmorphism meets Japanese Zen Minimalism  
> **Target Framework**: Kotlin Compose Multiplatform (Android & Desktop)  
> **Native Bridges**: AndroidX `Media3` (ExoPlayer) + Linux MPRIS D-Bus Engine + UniFFI Rust Core  
> **Audio Quality Baseline**: Bit-perfect 320 Kbps Studio Master AAC (`m4a`)  
> **Specification Version**: `2.0.0-STABLE`

---

## 1. Visual Identity & Design Tokens

### 1.1 Philosophy: "Liquid Studio Acoustics"
Softify Next-Gen discards noisy advertising banners, harsh algorithmic churn, and clashing saturated gradients. It models the player as a serene, dark-mode listening studio built from frosted optical acrylic, subtle specular light refractions, and real-time chromatic atmosphere sampled from active album artwork.

Every element adheres to three non-waivable principles:
1. **Dynamic Chromatic Atmosphere**: The background canvas is never static. It breathes with an ambient 3-point radial mesh glow derived from the current track's dominant palette, softened across a 60dp Gaussian blur.
2. **True Optical Glassmorphism**: Cards and floating controls do not use arbitrary solid borders or flat gray containers. They utilize multi-pass blurred acrylic (`backdrop-filter: blur(40dp)`), a 1px top specular bevel highlight simulating physical glass edge catch (`rgba(255, 255, 255, 0.12)`), and deep ambient occlusion shadows.
3. **Studio Typography & Precision Hierarchy**: Visual hierarchy is anchored in pure weight and optical spacing rather than jarring color shifts. High-fidelity audio data (bitrates, sample frequencies, codecs) is rendered strictly in tabular monospace figures.

---

### 1.2 Design Token System

All screens and Compose Multiplatform composables must bind strictly to the tokens declared below. Never use hardcoded literals in UI components.

#### A. Color Palette & Glass Materials
| Token Name | Hex / RGBA Value | Compose Color Def | Semantic Usage |
| :--- | :--- | :--- | :--- |
| `Surface-0` | `#050508` | `Color(0xFF050508)` | Deep Obsidian Base Canvas (Pitch black foundation) |
| `Surface-1` | `rgba(18, 18, 24, 0.65)` | `Color(0xA6121218)` | Persistent Shell Acrylic (Sidebar, Top Bar, Bottom Nav) with 40dp blur |
| `Surface-2` | `rgba(255, 255, 255, 0.05)` | `Color(0x0DFFFFFF)` | Floating Glass Cards, Track Item Rows, Dialog Containers |
| `Surface-3` | `rgba(255, 255, 255, 0.10)` | `Color(0x1AFFFFFF)` | Elevated Pills, Active Chips, Filter Bar Buttons, Search Inputs |
| `Glass-Highlight` | `rgba(255, 255, 255, 0.12)` | `Color(0x1FFFFFFF)` | 1px Top Bevel Border (Specular light refraction on glass edges) |
| `Glass-Shadow` | `rgba(0, 0, 0, 0.40)` | `Color(0x66000000)` | Diffused Ambient Shadow below floating glass sheets |
| `Accent-Primary` | `#FA243C` | `Color(0xFFFA243C)` | Apple Music Crimson (Primary Play Buttons, Active Sliders, Selected Tab) |
| `Accent-Hover` | `#FC3C44` | `Color(0xFFFC3C44)` | Interactive Hover/Press Tint for Primary Actions |
| `Accent-Studio` | `#F59E0B` | `Color(0xFFF59E0B)` | Studio Master Gold (320 Kbps High-Bitrate Badge, Hi-Res Indicators) |
| `Text-Primary` | `#FFFFFF` | `Color(0xFFFFFFFF)` | Track Titles, Primary Screen Headings, Modal Headlines (100%) |
| `Text-Secondary`| `#8E8E93` | `Color(0xFF8E8E93)` | Artist Names, Section Labels, Tab Bar Inactive Items (60%) |
| `Text-Tertiary` | `#48484A` | `Color(0xFF48484A)` | Secondary Durations, Track Numbers, File Codecs, Dividers (40%) |
| `Status-Online` | `#34C759` | `Color(0xFF34C759)` | Cached Offline Ready (Green frosted indicator dot) |
| `Status-Danger` | `#FF453A` | `Color(0xFFFF453A)` | Mute/Block Track, Destructive Purge Cache, Disconnect |

#### B. Typography Matrix
| Token | Typeface Family | Weight | Size | Line Height | Tracking |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `Display-Hero` | SF Pro Display / Inter Tight | Bold (700) | 34sp | 40sp | -0.6sp |
| `Title-Large` | SF Pro Display / Inter Tight | SemiBold (600)| 24sp | 30sp | -0.4sp |
| `Title-Medium` | SF Pro Display / Inter Tight | Medium (500) | 18sp | 24sp | -0.2sp |
| `Body-Primary` | SF Pro Text / Plus Jakarta Sans| Regular (400) | 15sp | 20sp | 0sp |
| `Body-Emphasis`| SF Pro Text / Plus Jakarta Sans| SemiBold (600)| 15sp | 20sp | 0sp |
| `Caption` | SF Pro Text / Plus Jakarta Sans| Regular (400) | 12sp | 16sp | +0.1sp |
| `Caption-Bold` | SF Pro Text / Plus Jakarta Sans| SemiBold (600)| 11sp | 14sp | +0.4sp |
| `Code-Meta` | SF Mono / JetBrains Mono | Medium (500) | 12sp | 16sp | 0sp (Tabular) |
| `Lyrics-Active`| SF Pro Display / Inter Tight | Bold (700) | 26sp | 34sp | -0.3sp |
| `Lyrics-Idle` | SF Pro Display / Inter Tight | Medium (500) | 22sp | 30sp | -0.2sp |

#### C. Elevation, Radius & Micro-Motion Physics
- **Radius Scale**:
  - `Radius-XS` (`6dp`): Tooltips, mini quality tags, status dots.
  - `Radius-SM` (`10dp`): Search input box, filter chips, equalizer fader caps.
  - `Radius-MD` (`16dp`): Album art cards, song list hover items, dialog popups.
  - `Radius-LG` (`24dp`): Fullscreen Now Playing container, Wrapped stats cards, Bottom Sheet drawer.
  - `Radius-Pill` (`999dp`): Primary play buttons, search badges, volume sliders, track scrubbing pills.
- **Spring Physics**:
  - All interactive tap and press states use Compose `spring(dampingRatio = 0.78f, stiffness = 380f)`.
  - Button press state dynamically scales down to `0.97f` with instant response.
  - Page transitions use cross-fade with `220ms` duration and `FastOutSlowInEasing`.
  - Sheet slide-ups animate via `spring(dampingRatio = 0.82f, stiffness = 420f)`.

---

## 2. Adaptive Framework Architecture

Softify Next-Gen supports both mobile and desktop platforms seamlessly:

```
┌────────────────────────────────────────────────────────────────────────┐
│               COMPOSE MULTIPLATFORM PRESENTATION LAYER                 │
│                                                                        │
│   DESKTOP (Linux / macOS / Windows)            ANDROID (Mobile / Tablet)│
│   • Persistent Left Glass Nav (240dp)          • Bottom Frosted Bar (68dp)│
│   • Top Search & Ambient Header (56dp)        • Gesture Pull Sheets     │
│   • Persistent Bottom Player Bar (88dp)        • Floating Mini Player (64dp)│
│   • Linux MPRIS D-Bus Integration              • AndroidX Media3 Service │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ ViewModel State Flow
┌───────────────────────────────────▼────────────────────────────────────┐
│                    UNIFIED MULTIPLATFORM STATE FLOW                    │
│   • PlayerState (Track, Position, Buffer, Bitrate, PlaybackStatus)     │
│   • LibraryState (LikedTracks, CustomPlaylists, MutedList, Offline)    │
│   • SearchState (FTS5 Realtime Results, QueryHistory, Filters)         │
│   • AmbientPaletteState (Dominant, Secondary, Vibrant, DarkVibrant)    │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ UniFFI Bridge Callbacks
┌───────────────────────────────────▼────────────────────────────────────┐
│                      RUST CORE ENGINE WORKSPACE                        │
│   softify-core • softify-resolvers • softify-storage • softify-crypto   │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Screen-by-Screen Blueprint Specifications

### Screen 1: Home & Discovery Hub (`/home`)

```
┌────────────────────────────────────────────────────────────────────────┐
│  Good Evening, Sarthak                     [ 🔍 Search ]  [ ⚙️ Settings ]│
│  ✨ Studio Master 320k Stream Active                                   │
│                                                                        │
│  FOR YOU · DAILY MIXES                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ Daily Mix 1  │  │ Daily Mix 2  │  │ Daily Mix 3  │  │ Weekly Pick  │ │
│  │ [The Weeknd] │  │ [Arijit]     │  │ [Hans Zimmer]│  │ [Discover]   │ │
│  │ ▶ 320k M4A   │  │ ▶ 320k M4A   │  │ ▶ 320k M4A   │  │ ▶ 320k M4A   │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘ │
│                                                                        │
│  MOOD WAVES                                                            │
│  [ (⚡ Focus Flow) ]  [ (🌙 Midnight Chill) ]  [ (🔥 Workout Drive) ]   │
│                                                                        │
│  RECENTLY PLAYED · STUDIO MASTER QUALITY                               │
│  ┌──┬─────────────────────────────┬──────────────┬────────────┬──────┐ │
│  │01│ Starboy                     │ The Weeknd   │ 320 KBPS   │ 3:50 │ │
│  │02│ Tum Hi Ho                   │ Arijit Singh │ 320 KBPS   │ 4:22 │ │
│  │03│ Interstellar (Main Theme)   │ Hans Zimmer  │ 320 KBPS   │ 6:40 │ │
│  └──┴─────────────────────────────┴──────────────┴────────────┴──────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

- **Component Hierarchy**:
  1. `TopAppBar`: Transparent glass app bar with dynamic greeting (`Good Morning` / `Good Afternoon` / `Good Evening`), ambient radial gradient halo behind avatar icon.
  2. `SectionHeader`: `Title-Medium` text with subtle `Text-Secondary` category counter.
  3. `DailyMixRow`: Horizontal snapping `LazyRow` featuring `DailyMixCard` composables:
     - Aspect ratio 1:1 rounded square (`Radius-MD`) with frosted glass overlay.
     - 4-quadrant dynamic album art montage.
     - Quick Play floating pill with `Accent-Primary` fill and white play triangle.
     - Gold `320 KBPS` studio badge pill pinned to top-right corner.
  4. `MoodChipsRow`: Horizontal scroll of `Surface-3` glass pills (`Radius-Pill`) with 1px `Glass-Highlight` border. Selecting a chip instantly filters dynamic recommendations without navigating away.
  5. `TrackTable`: Vertically scrollable song list rows with hover glass highlight (`Surface-2`), mono track duration (`Code-Meta`), and quick-action menu (Like, Add to Playlist, Mute Song).

---

### Screen 2: Search & Live Catalog Exploration (`/search`)

```
┌────────────────────────────────────────────────────────────────────────┐
│  [ 🔍 Search songs, artists, studio albums, YouTube links...         ] │
│                                                                        │
│  FILTERS: [ All ]  [ (Songs) ]  [ (Artists) ]  [ (Playlists) ]         │
│                                                                        │
│  MATCHED TRACKS (FTS5 INSTANT RESOLUTION)                              │
│  ┌───┬────────────────────────────┬─────────────┬───────────┬────────┐ │
│  │[🖼]│ Blinding Lights            │ The Weeknd  │ 320k M4A  │ [ ▶ ]  │ │
│  │   │ After Hours (2020)         │ Studio Mast │ AAC Studio│ [ ··· ]│ │
│  ├───┼────────────────────────────┼─────────────┼───────────┼────────┤ │
│  │[🖼]│ One Dance                  │ Drake, Kyla │ 320k M4A  │ [ ▶ ]  │ │
│  │   │ Views (2016)               │ Studio Mast │ AAC Studio│ [ ··· ]│ │
│  └───┴────────────────────────────┴─────────────┴───────────┴────────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

- **Component Hierarchy**:
  1. `SearchBarGlass`:
     - Height `52dp`, background `Surface-2`, shape `Radius-SM`.
     - 1px continuous `Glass-Highlight` border.
     - Real-time debounced (150ms) input binding into `softify-storage` SQLite FTS5 index and JioSaavn 320k network resolver.
     - Trailing clear icon (`✕`) fading in only when text length $> 0$.
  2. `FilterChipGroup`: Horizontal segmented glass bar with spring animation on selection indicator pill.
  3. `LiveSearchResultList`:
     - Infinite scrolling `LazyColumn` presenting unified search results.
     - Each item displays:
       - 48x48dp rounded cover art with cross-fade image caching.
       - Track title in `Body-Emphasis` (`Text-Primary`).
       - Artist names with Unicode non-breaking space normalization (`Text-Secondary`).
       - Studio Fidelity Badge: Frosted amber pill with `320 KBPS` label in `Code-Meta`.
       - More actions button (`···`) expanding an optical blur context menu:
         - `Play Next`
         - `Add to Playlist...`
         - `Download Offline (320k AAC)`
         - `Mute Track (Never Play Again)`: Highlighted in `Status-Danger`.

---

### Screen 3: Fullscreen Studio Master Player & Synced Lyrics (`/player`)

```
┌────────────────────────────────────────────────────────────────────────┐
│  [ ⌵ Minimize ]           NOW PLAYING STUDIO           [ 🎚️ Equalizer ]│
│                                                                        │
│                     ┌───────────────────────┐                          │
│                     │                       │                          │
│                     │                       │   LYRICS DRAWER          │
│                     │    ALBUM COVER ART    │   (KARAOKE SYNC)         │
│                     │       (380x380)       │                          │
│                     │                       │   "I said, ooh, I'm      │
│                     │                       │    blinded by the        │
│                     └───────────────────────┘    lights"               │
│                                                 (Active: White 100%)   │
│                       Blinding Lights                                  │
│                          The Weeknd             "No, I can't sleep     │
│                    [ 320 KBPS · AAC · M4A ]      until I feel your     │
│                                                  touch"                │
│       1:24 ━━━━━━━━━━━━━●────────────────────── 3:20                    │
│                                                                        │
│              [ 🔀 ]    [ ⏮ ]   [  ▶  ]   [ ⏭ ]    [ 🔁 ]               │
│                                                                        │
│       [ 😴 Sleep Timer: 30m ]    [ ❤️ Liked ]    [ 🚫 Mute Track ]     │
└────────────────────────────────────────────────────────────────────────┘
```

- **Atmospheric Background Engine**:
  - Live 2-pass background composed of the album art scaled to 150%, blurred with `blur(80dp)`, layered over `Surface-0` (`#050508`).
  - Subtle breathing gradient animation rotating slowly at 0.05 Hz.
- **Component Hierarchy**:
  1. `TopNavigationControls`: Back/Minimize chevron on left, track provider indicator in center, 10-band Equalizer shortcut on right.
  2. `AlbumArtVibrancyFrame`:
     - Centered `380x380dp` (desktop) or `300x300dp` (mobile) album art card.
     - `Radius-LG` corner curvature.
     - Ambient color drop shadow extending 32dp with 35% opacity.
  3. `MetadataBlock`:
     - Track title in `Display-Hero` typography.
     - Artist and album in `Title-Medium`.
     - `StudioMasterHUD`: Frosted amber pill displaying `320 KBPS · AAC · 44.1 kHz Studio Master`.
  4. `ScrubberTimeline`:
     - Custom liquid scrub bar. Played duration track in `Accent-Primary`, buffered track in `rgba(255, 255, 255, 0.20)`, unplayed track in `rgba(255, 255, 255, 0.08)`.
     - Current and total timecodes in `Code-Meta` (`SF Mono`).
  5. `PlaybackControlsDeck`:
     - Large central Play/Pause circle (`64dp`) filled with `Accent-Primary`, scaling to `0.94f` on click.
     - Skip Previous (`⏮`) and Skip Next (`⏭`) with smooth tap haptics.
     - Shuffle (`🔀`) and Repeat (`🔁`) toggles with subtle amber dot indicator when active.
  6. `SyncedLyricsEngine`:
     - Side drawer on Desktop; swipe-up full sheet on Mobile.
     - Supports **Romanized non-English lyrics** mapped side-by-side with localized text.
     - Auto-scrolls synchronously with millisecond-precision audio timestamps (`[mm:ss.xx]`).
     - Inactive lines rendered in `Lyrics-Idle` with 30% opacity; active singing line rendered in `Lyrics-Active` with 100% white glow.
     - Clicking any lyric timestamp instantly seeks playback to that exact audio position.

---

### Screen 4: Library, Liked Songs & Offline Cache (`/library`)

```
┌────────────────────────────────────────────────────────────────────────┐
│  Your Library                              [ + Create New Playlist ]   │
│                                                                        │
│  TABS: [ (Liked Songs 284) ]  [ (Playlists 12) ]  [ (Offline Cached 96) ]│
│                                                                        │
│  ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐   │
│  │ ❤️ Liked Tracks   │  │ ⚡ Late Night Drive│  │ 💾 Offline Studio  │   │
│  │ 284 Studio Tracks │  │ 42 Studio Tracks  │  │ 96 Cached M4A     │   │
│  │ [ Play All ▶ ]    │  │ [ Play All ▶ ]    │  │ Storage: 1.2 GB   │   │
│  └───────────────────┘  └───────────────────┘  └───────────────────┘   │
│                                                                        │
│  ALL TRACKS                                                            │
│  ┌──┬────────────────────────────┬─────────────┬───────────┬─────────┐ │
│  │01│ After Hours                │ The Weeknd  │ ● CACHED  │ 6:01    │ │
│  │02│ Bohemian Rhapsody          │ Queen       │ ● CACHED  │ 5:55    │ │
│  └──┴────────────────────────────┴─────────────┴───────────┴─────────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

- **Component Hierarchy**:
  1. `LibraryTabBar`: Multi-segment glass switcher filtering between Liked Tracks, User Playlists, and Offline Downloads.
  2. `PlaylistGrid`: 3-column responsive grid of high-fidelity glass cards:
     - Card background `Surface-2` with 1px `Glass-Highlight` top border.
     - Heart badge in `Accent-Primary` for Liked Tracks.
     - Offline cache status pill with `Status-Online` green dot indicating disk persistence.
  3. `CachedTrackRow`: Displays local download status, storage footprint, and swipe-to-delete action.

---

### Screen 5: Universal Playlist Importer (`/import`)

```
┌────────────────────────────────────────────────────────────────────────┐
│  Universal Playlist Importer                                           │
│  Import unlimited Spotify and YouTube playlists with zero rate limits  │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ 🔗 Paste Spotify or YouTube Playlist URL...             [PASTE]  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                        │
│  STATUS: RESOLVING TRACKS AT 320 KBPS                                  │
│  Progress: [████████████████████████░░░░░░] 78 / 100 Tracks Resolved    │
│                                                                        │
│  RESOLVED CANDIDATES                                                   │
│  ✓ 01. Blinding Lights · The Weeknd       -> JioSaavn Studio 320k AAC  │
│  ✓ 02. Save Your Tears · The Weeknd       -> JioSaavn Studio 320k AAC  │
│  ⚠ 03. Rare Indie B-Side · Unknown Artist -> YTM InnerTube Fallback    │
│                                                                        │
│  [ (Save as New Playlist) ]              [ (Download All for Offline) ]│
└────────────────────────────────────────────────────────────────────────┘
```

- **Component Hierarchy**:
  1. `UrlInputGlassBox`: Large input bar with auto-detect clipboard button. Supports `open.spotify.com/playlist/...`, `open.spotify.com/album/...`, and `music.youtube.com/playlist?list=...`.
  2. `LiveResolutionProgressBar`: Two-tone smooth animated progress bar showing real-time batch resolution.
  3. `CandidateAuditList`: Real-time streaming log of matched tracks with provider badges (`JioSaavn Studio 320k` or `YouTube InnerTube Fallback`).
  4. `ActionFooter`: Frosted button bar to save to local SQLite database or begin batch offline download.

---

### Screen 6: Softify Wrapped & Audio Persona (`/wrapped`)

```
┌────────────────────────────────────────────────────────────────────────┐
│  Softify Wrapped · Your Year in High-Fidelity Sound                    │
│  [ ❚❚ Pause Story ]                                     Story 2 of 5   │
│                                                                        │
│         ┌──────────────────────────────────────────────────────┐       │
│         │               YOUR AUDIO PERSONA                     │       │
│         │                                                      │       │
│         │              🔮 THE NIGHT CYCLIST                   │       │
│         │                                                      │       │
│         │  You thrive when the world sleeps. 68% of your       │       │
│         │  listening happens between 11 PM and 4 AM.           │       │
│         │                                                      │       │
│         │  • Total Time Streamed: 42,850 Minutes               │       │
│         │  • Studio Master 320k Fidelity Ratio: 99.4%          │       │
│         │  • Top Artist: The Weeknd (4,120 mins)               │       │
│         │  • Top Genre: Synthwave & Melodic R&B                │       │
│         │                                                      │       │
│         │  [ 📤 Share Wrapped Card ]   [ ▶ Play Top 100 Songs ]│       │
│         └──────────────────────────────────────────────────────┘       │
└────────────────────────────────────────────────────────────────────────┘
```

- **Component Hierarchy**:
  1. `StoryProgressBar`: Multi-segment top progress bar (5 cards: Minutes Listened, Top Artists, Top Songs, Listening Clock, Audio Persona).
  2. `WrappedGlassCard`: Fullscreen vertical glass presentation card with dynamic iridescent border and floating typography.
  3. `ShareButton`: Generates a high-resolution exportable image card formatted for sharing on social channels.

---

### Screen 7: 10-Band Studio Equalizer (`/equalizer`)

```
┌────────────────────────────────────────────────────────────────────────┐
│  10-Band Studio Parametric Equalizer                  [ Preset: Flat ⌵]│
│  [ PRESETS: (Flat)  (Bass Boost)  (Acoustic)  (Vocal Polish)  (Studio) ]│
│                                                                        │
│  PREAMP: [ +2.5 dB ] ━━━━━━━━●━━━━━━━━━━━━━━━━━━━━━━━━ (+12dB)         │
│                                                                        │
│   +12dB ───┬────┬────┬────┬────┬────┬────┬────┬────┬────┬───────────   │
│            │    │    │    │    │    │    │    │    │    │             │
│            │   [●]   │    │    │   [●]   │    │    │    │             │
│    0dB ───[●]───┼───[●]───┼───[●]───┼───[●]───┼───[●]──[●]──────────   │
│            │    │    │   [●]   │    │    │   [●]   │    │             │
│   -12dB ───┴────┴────┴────┴────┴────┴────┴────┴────┴────┴───────────   │
│           32Hz 64Hz 125Hz 250Hz 500Hz 1kHz 2kHz 4kHz 8kHz 16kHz        │
│                                                                        │
│  [ (Reset to Flat) ]                              [ (Save Custom) ]    │
└────────────────────────────────────────────────────────────────────────┘
```

- **Component Hierarchy**:
  1. `PresetScrollDeck`: Horizontal carousel of quick EQ presets.
  2. `PreampSlider`: Horizontal gain fader with zero-point detent indicator.
  3. `FaderDeck10Band`: 10 vertical precision glass faders:
     - Frequencies: 32 Hz, 64 Hz, 125 Hz, 250 Hz, 500 Hz, 1 kHz, 2 kHz, 4 kHz, 8 kHz, 16 kHz.
     - Smooth Bezier curve connecting fader nodes in real-time.
     - Live values displayed in `Code-Meta` format (`+3.2 dB`).

---

### Screen 8: Settings, Storage & GitHub Updater (`/settings`)

```
┌────────────────────────────────────────────────────────────────────────┐
│  Settings & System Preferences                                         │
│                                                                        │
│  AUDIO STREAMING FIDELITY                                              │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ Default Stream Quality: [ 320 Kbps Studio Master (AAC)         ⌵]│  │
│  │ Rewrite Low Bitrates (_96 / _160 to _320.mp4):         [ ON (✓) ]│  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                        │
│  MUTED & BLOCKED TRACKS ("NEVER PLAY AGAIN")                           │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ 2 Tracks Blocked from all Mixes & Radio                [Manage ⌵]│  │
│  │ • "Despacito" · Luis Fonsi                       [ Unblock / ✕ ] │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                        │
│  STORAGE & CACHE MANAGEMENT                                            │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ Offline Track Storage: 1.42 GB (112 Songs)      [ Clear Audio ]  │  │
│  │ SQLite FTS5 Search Database: 12.8 MB            [ Rebuild Index] │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                        │
│  APP UPDATES (GITHUB RELEASES)                                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ Current Version: v2.0.0-STABLE (Latest)       [ Check Update 🔄 ]│  │
│  │ In-App Auto Update Channel: Release Builds             [ Active ]│  │
│  └──────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

- **Component Hierarchy**:
  1. `SettingsSection`: Frosted container (`Surface-2`) grouping related preferences.
  2. `QualitySelector`: Segmented switch ensuring the 320 Kbps studio stream rewriter remains permanently enforced.
  3. `MutedTracksManager`: Interactive list of blocked tracks with instant unblock button. Blocked songs are guaranteed never to appear in recommendation mixes.
  4. `GitHubUpdaterCard`: Displays current commit hash, release tag, and one-tap update download progress.

---

## 4. Compose Multiplatform Implementation Contract

### 4.1 Custom Modifiers for Glassmorphism

```kotlin
// Glass Container Modifier with Optical Bevel Highlight
fun Modifier.glassmorphicSurface(
    shape: Shape = RoundedCornerShape(16.dp),
    backgroundColor: Color = Color(0x0DFFFFFF), // Surface-2
    borderColor: Color = Color(0x1FFFFFFF),     // Glass-Highlight
    borderWidth: Dp = 1.dp
): Modifier = this
    .clip(shape)
    .background(backgroundColor)
    .border(width = borderWidth, color = borderColor, shape = shape)

// Spring Touch Feedback Modifier
fun Modifier.springPress(): Modifier = composed {
    var isPressed by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(
        targetValue = if (isPressed) 0.97f else 1.0f,
        animationSpec = spring(
            dampingRatio = 0.78f,
            stiffness = 380f
        )
    )
    this
        .graphicsLayer {
            scaleX = scale
            scaleY = scale
        }
        .pointerInput(Unit) {
            detectTapGestures(
                onPress = {
                    isPressed = true
                    tryAwaitRelease()
                    isPressed = false
                }
            )
        }
}
```

### 4.2 Architectural State Invariants
1. **Never stall audio playback**: UI rendering runs on the main thread; all crypto decryption, FTS5 queries, and 320k stream resolution execute on background Tokio / Kotlin Coroutine worker threads.
2. **Dynamic ambient color bleed**: Update `AmbientPaletteState` with a 300ms cross-fade whenever `PlayerState.currentTrack` changes to maintain smooth visual flow.
3. **Muted songs exclusion**: Any track present in `muted_tracks` table must be omitted from auto-play queues and algorithmic daily mixes without throwing errors.
4. **Offline indicator reliability**: A track displays the green `Status-Online` dot if and only if the decrypted 320k `.m4a` file exists on the local storage partition.
