# Softify Next-Gen — UI/UX Design Specification & Architecture

> **Design Theme**: Minimalist, Warm & Cozy, Frosted Studio Glassmorphism  
> **Target Framework**: Kotlin Compose Multiplatform (Android & Desktop)  
> **Native Bridges**: AndroidX `Media3` (ExoPlayer) + Linux MPRIS D-Bus Engine  
> **Core Typography**: **Outfit** (Headings & Accents) + **Plus Jakarta Sans** (Body & Interface) + **JetBrains Mono** (Timecodes & Metadata)

---

## 1. Visual Identity & Design Tokens

### 1.1 Philosophy: "The Cozy Living Room Studio"
Softify Next-Gen discards noisy advertisements, harsh neon clashes, and cluttered social feeds. It treats the music player as a serene, warm, distraction-free sanctuary. Every screen balances:
- **Calm Minimalism**: Generous whitespace, razor-sharp hierarchy, and no superfluous visual noise.
- **Warm & Cozy Ambiance**: Deep velvet obsidian base, warm amber and terracotta undertones, and gentle glow inspired by late-night listening sessions.
- **Precision Glassmorphism**: Translucent floating surfaces, multi-layer depth, 1px frosted specular borders, and dynamic ambient lighting derived from the active album cover art.

---

### 1.2 Color System & Design Tokens

```
┌────────────────────────────────────────────────────────────────────────┐
│                        COZY GLASS COLOR PALETTE                        │
├──────────────────────┬─────────────┬───────────────────────────────────┤
│ Token                │ Hex / RGBA  │ Description & Usage               │
├──────────────────────┼─────────────┼───────────────────────────────────┤
│ --bg-base            │ #0A0A12     │ Deep velvet obsidian canvas       │
│ --bg-surface-glass   │ #161524BF   │ 75% translucent surface glass     │
│ --bg-card-glass      │ #1E1C3080   │ 50% frosted card container        │
│ --bg-overlay-modal   │ #0D0C17E6   │ 90% blur backdrop for modals      │
│ --accent-amber       │ #F59E0B     │ Warm cozy amber glow              │
│ --accent-studio-green│ #10B981     │ 320 Kbps studio master badge      │
│ --accent-terracotta  │ #E07A5F     │ Warm highlights & like indicators │
│ --text-primary       │ #FAF7F2     │ Cozy warm cream (contrast > 12:1) │
│ --text-secondary     │ #A8A29E     │ Muted warm stone gray             │
│ --text-tertiary      │ #78716C     │ Subtle captions & timecodes       │
│ --glass-border       │ #FFFFFF1A   │ 1px subtle specular rim highlight │
│ --glass-border-focus │ #F59E0B4D   │ Active focus glass outline        │
└──────────────────────┴─────────────┴───────────────────────────────────┘
```

#### Ambient Backdrop Lighting Formula
When a track is playing, the background renders a fluid multi-color gradient bleed derived from the dominant and secondary colors of the album art:
- Layer 0: Solid `--bg-base` (`#0A0A12`)
- Layer 1: Two radial gradient light orbs (radius `320dp`, blur `90dp`, opacity `18%`, animated with a slow 12-second sinusoidal drift)
- Layer 2: Full-screen frosted glass overlay (`backdrop-filter: blur(40dp)`)

---

### 1.3 Typography System

| Role | Font Family | Weight | Size (dp/sp) | Tracking | Line Height |
|---|---|---|---|---|---|
| **Display Hero** | **Outfit** | SemiBold (600) | 36sp | -0.5px | 44sp |
| **Title Large** | **Outfit** | Medium (500) | 24sp | -0.2px | 32sp |
| **Title Medium** | **Outfit** | SemiBold (600) | 18sp | 0px | 24sp |
| **Body Large** | **Plus Jakarta Sans** | Regular (400) | 16sp | +0.1px | 24sp |
| **Body Medium** | **Plus Jakarta Sans** | Regular (400) / Medium (500) | 14sp | +0.15px | 20sp |
| **Caption / Label** | **Plus Jakarta Sans** | Medium (500) | 12sp | +0.3px | 16sp |
| **Timecode / Meta** | **JetBrains Mono** | Regular (400) | 11sp | 0px | 14sp |

---

### 1.4 Glass Surface Components

Every glass component follows a 3-layer compositing rule:
1. **Backdrop Blur**: `RenderEffect.createBlurEffect(16f, 16f)` on Android 12+ / Compose Desktop Skia blur shader.
2. **Surface Fill**: Translucent dark acrylic (`rgba(22, 21, 36, 0.70)`).
3. **Specular Rim Stroke**: `1.dp` border with a linear gradient from top-left (`rgba(255, 255, 255, 0.15)`) to bottom-right (`rgba(255, 255, 255, 0.03)`).

---

## 2. Screen-by-Screen In-Depth UI Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│                        APPLICATION NAVIGATION MAP                      │
├────────────────────────────────────────────────────────────────────────┤
│  [ Home ] ─── [ Search ] ─── [ Library ] ─── [ Insights/Wrapped ]      │
│      │             │              │                  │                 │
│      └─────────────┴──────┬───────┴──────────────────┘                 │
│                           │                                            │
│                 [ Mini-Player Dock ]                                   │
│                           │ (Tap to Expand)                            │
│                 [ Full Now Playing Player ]                            │
│                 ├── Synced Lyrics Sheet (with Romanization)            │
│                 ├── 10-Band Studio Equalizer                           │
│                 ├── Sleep Timer Modal                                  │
│                 └── Audio Queue Sheet                                  │
└────────────────────────────────────────────────────────────────────────┘
```

---

### Screen 1: Home Dashboard ("The Living Room")

The entry point greets the user with warm, personalized listening hubs and instant 1-tap playback.

#### 1. Top Glass Header
- **Greeting**: Time-aware greeting (`"Good morning, Sarthak"` / `"Quiet evening"`) in `Outfit Medium 22sp`.
- **Status Indicator**: Right-aligned pill badge: `● 320k Studio Ready` in `Studio Green` with a subtle pulsing dot.
- **Settings / GitHub Update Pill**: Gear icon and subtle update indicator with version badge.

#### 2. Quick-Listen 6-Grid (Top Favorites)
- 2 rows × 3 columns of compact frosted glass cards (`height: 56dp`).
- Left: `48dp` rounded square artwork (`radius: 8dp`).
- Center: Track/Playlist title in `Plus Jakarta Sans SemiBold 13sp` (single-line ellipsis).
- Right: Subtle hover/pressed glowing Play button (`accent-amber`).
- **Interaction**: Tap anywhere on the card immediately begins playback without navigating away.

#### 3. Daily Mixes Carousel (Mixes 1 to 6)
- Horizontal snapping carousel of cards (`width: 160dp`, `height: 220dp`, `corner: 20dp`).
- Card layout:
  - Top: Album art collage or stylized minimalist generative pattern.
  - Middle: `"Daily Mix 1"` in `Outfit SemiBold 16sp`.
  - Bottom: Artist tags (e.g. `"The Weeknd, Daft Punk, Gesaffelstein"`) in `text-secondary 12sp`.
  - Ambient glass glow behind each card matching the mix's primary genre palette.

#### 4. Mood Stations Shelf (Horizontal Pills)
- Horizontal chip row: `[ ☕ Cozy Acoustic ] [ 🌙 Midnight Drift ] [ ⚡ High Voltage ] [ 🧘 Deep Focus ] [ 🌧️ Melancholy ]`.
- Selecting a mood immediately cross-fades the recommendations and generates a 25-track smart mood mix on the fly.

#### 5. "Jump Back In" Session Shelf
- Horizontal list of the user's recently paused albums and playlists with a miniature horizontal progress bar beneath the artwork showing remaining duration.

---

### Screen 2: Search & Discovery (Multi-Source Hub)

#### 1. Floating Glass Search Bar (Sticky Top)
- Glass capsule (`height: 52dp`, `corner: 26dp`, border `1px solid rgba(255,255,255,0.12)`).
- Leading: Search magnifying glass icon (`text-secondary`).
- Placeholder: `"Search songs, artists, or paste YouTube/Spotify link..."`
- Trailing:
  - Clear icon (when text is present).
  - Quick `"Paste Link"` button that auto-detects clipboard URLs.

#### 2. Universal Link Drop & Direct Player
- If the user enters or pastes:
  - **YouTube Link** (`https://youtu.be/...` or `youtube.com/watch?v=...`):
    - Instantly renders a **"Direct YouTube Stream"** preview card showing video title, channel name, thumbnail, and a prominent `[ Play Audio Only (320k/Opus) ]` button.
  - **Spotify Playlist Link** (`https://open.spotify.com/playlist/...`):
    - Instantly renders an **"Import Spotify Playlist"** card with track count preview and `[ Import to Local Library ]` action.

#### 3. Live FTS5 Search Results
- **Category Filter Tabs**: `All`, `Tracks`, `Artists`, `Albums`, `Playlists`.
- **Top Result Card (Glass Hero)**:
  - Large `72dp` artwork with soft back-glow.
  - Title in `Outfit SemiBold 18sp` + Artist in `Plus Jakarta Sans 14sp`.
  - `"Studio Master 320k"` badge.
  - Instant Play floating action button.
- **Track List Rows**:
  - `48dp` artwork thumbnail.
  - Title, Artist, Album, and duration formatted in `JetBrains Mono 12sp`.
  - Overflow menu `(⋮)`: *Like, Add to Playlist, Download 320k Offline, Never Play Again (Mute)*.

---

### Screen 3: Now Playing Screen (Full Studio Player & Lyrics)

The centerpiece of Softify. Designed with breathtaking depth, fluid motion, and zero clutter.

```
┌────────────────────────────────────────────────────────┐
│ [▼ Minimize]          NOW PLAYING          [⋯ Options] │
├────────────────────────────────────────────────────────┤
│                                                        │
│                    ┌─────────────┐                     │
│                    │             │                     │
│                    │  ALBUM ART  │ (Ambient Color      │
│                    │  [320x320]  │  Bleed Behind)      │
│                    │             │                     │
│                    └─────────────┘                     │
│                                                        │
│  Blinding Lights                          [ ♥ Liked ]  │
│  The Weeknd • After Hours                              │
│  [ STUDIO MASTER 320 KBPS AAC ]                        │
│                                                        │
│  ───────●────────────────────────────────────────────  │
│  01:14                                          03:20  │
│                                                        │
│       [ 🔀 ]    [⏮]    [  ▶  ]    [⏭]    [ 🔁 ]        │
│                                                        │
├────────────────────────────────────────────────────────┤
│  [📜 Synced Lyrics]    [🎚️ Equalizer]    [🌙 Sleep 30m] │
└────────────────────────────────────────────────────────┘
```

#### 1. Artwork & Ambient Stage
- Centerpiece: `320dp × 320dp` artwork with `24dp` rounded corners and subtle specular rim.
- Elevation: Dynamic shadow matching dominant art color with `40dp` spread at 25% opacity.
- Fluid vinyl rotation animation (optional toggle) or gentle pulsing breathing animation.

#### 2. Track Meta & Quality Indicator
- Title: `Outfit SemiBold 24sp`, auto-scrolling marquee if text overflows.
- Artist & Album: `Plus Jakarta Sans 16sp` with interactive artist link.
- **Studio Fidelity Pill**:
  - `[ 320 KBPS AAC • STUDIO MASTER ]` rendered in frosted glass with emerald green typography (`#10B981`) and a discrete waveform icon.
  - Tap opens the **Audio Stream Inspector** showing actual CDN URL, container (`m4a`), codec (`aac`), sample rate (`44.1 kHz`), and provider (`jiosaavn_studio_320k`).

#### 3. Waveform Scrubber & Controls
- **Scrub Bar**: Smooth pill-shaped slider with buffer progress track in translucent white, played progress in warm amber, and glowing thumb knob.
- Time indicators: Left elapsed time, right total duration in `JetBrains Mono 12sp`.
- **Playback Control Deck**:
  - Shuffle (`🔀`), Previous (`⏮`), Play/Pause (`▶` / `⏸`), Next (`⏭`), Repeat (`🔁`).
  - Play/Pause Button: Large `68dp` floating glass circle with radial amber glow and smooth 150ms spring scale animation on press.

#### 4. Quick Action Dock
- **Like Button (Heart)**: Toggles filled terracotta heart with particle pop animation.
- **Mute Track ("Never Play Again")**: Discrete slash-circle icon. Prompts: *"Mute this track from all future mixes and autoplay?"*
- **Lyrics Drawer Toggle**: Slides up the full synchronized lyrics sheet.
- **Equalizer Button**: Opens the 10-band audio studio.
- **Sleep Timer Button**: Displays active countdown (e.g. `"24m remaining"`).

---

### Screen 4: Synchronized Lyrics & Romanization Sheet

A cozy, full-height karaoke-style typography canvas.

#### 1. Dual-Mode Lyrics Header
- Left: Song Title & Artist.
- Right: **Romanization Mode Switcher**:
  - `[ Original ]` (e.g., Devanagari, Hangul, Kanji)
  - `[ Romanized (Romaji / Hinglish) ]` (English alphabet phonetic translation)
- Close / Minimize drawer handle at top.

#### 2. Real-Time Lyrics Viewport
- Auto-scrolls synchronously with audio playback to keep the active line dead center.
- **Active Line Styling**:
  - Font: `Outfit SemiBold 26sp`.
  - Color: `#FAF7F2` (100% white-cream) with a soft text glow (`text-shadow: 0 0 16px rgba(245, 158, 11, 0.4)`).
  - Background: Subtle frosted spotlight capsule behind the active line.
- **Inactive Lines**:
  - Font: `Plus Jakarta Sans Regular 18sp`.
  - Color: `rgba(255, 255, 255, 0.35)`.
  - Blur: Soft `1.5px` depth-of-field blur on lines further away from center.
- **Interactive Scrub-to-Line**:
  - Tapping any lyric line instantly seeks audio playback to that exact timestamp tag (`[mm:ss.xx]`).

---

### Screen 5: Library & Playlists ("The Vault")

#### 1. Hero Liked Songs Card
- Full-width glass banner with an angled gradient heart backdrop.
- `"Liked Songs"` in `Outfit SemiBold 22sp`.
- Total tracks count and total playtime in minutes.
- Quick `"Shuffle All Liked Songs"` button.

#### 2. Spotify 1-Click Importer Sheet
- Prominent banner: `"Import any Spotify Playlist in seconds with Zero Rate Limits"`.
- Action opens glass modal:
  - Input: Paste Spotify URL (`https://open.spotify.com/playlist/...`).
  - Real-time progress bar: `"Imported 142 of 210 tracks..."`.
  - Complete: Automatically saves playlist into local SQLite and begins instant 320k stream resolution in the background.

#### 3. Custom Playlists & Offline Downloads
- Tabs: `Playlists`, `Artists`, `Albums`, `Downloaded (Offline 320k)`.
- Create Playlist button: Pops minimalist glass dialog requesting Title and Description.
- Downloaded Shelf: Filter exclusively for offline-ready cached `.m4a` files for flight/no-signal mode.

---

### Screen 6: 10-Band Studio Equalizer (DSP Suite)

Allows audiophiles to tune the output profile for headphones, car audio, or studio monitors.

```
┌────────────────────────────────────────────────────────┐
│ [← Back]           10-BAND STUDIO EQUALIZER            │
├────────────────────────────────────────────────────────┤
│  PRESET: [ Acoustic Cozy ▼ ]     [✓ EQ Enabled]        │
│                                                        │
│  +12dB ──                                              │
│          ▲       ▲                                     │
│   0dB ───┼───▲───┼───▲───▲───────▲───▲───────▲───────  │
│          ▼   ▼   ▼   ▼   ▼   ▲   ▼   ▼   ▲   ▼         │
│  -12dB ──────────────────────▼───────────▼───────────  │
│        31Hz 63Hz 125 250 500 1k  2k  4k  8k  16kHz     │
│                                                        │
├────────────────────────────────────────────────────────┤
│  BASS BOOST: ─────●────── [ +4 dB ]                    │
│  VIRTUALIZER: ──●──────── [ Off ]                      │
└────────────────────────────────────────────────────────┘
```

#### 1. Interactive Frequency Sliders
- 10 vertical sliders: `31Hz`, `63Hz`, `125Hz`, `250Hz`, `500Hz`, `1kHz`, `2kHz`, `4kHz`, `8kHz`, `16kHz`.
- Range: `-12.0 dB` to `+12.0 dB` with central 0dB snap notch.
- Fluid Bézier response curve connecting slider thumbs in real time.

#### 2. Presets Carousel
- Quick preset chips: `Flat`, `Acoustic Cozy`, `Deep Bass`, `Vocal Warmth`, `Late Night`, `Electronic`, `Rock`, `Club`.
- Custom preset saving: Save custom slider configurations to SQLite.

#### 3. DSP Enhancements
- Pre-amp Gain slider (`-6dB` to `+6dB`).
- Bass Boost rotary dial.
- Surround Virtualizer slider.

---

### Screen 7: Softify Wrapped (Listening Insights)

A nostalgic, cinematic celebration of the user's personal listening history.

#### Story Slides Carousel
1. **Slide 1 — Total Time**:
   - Giant stylized counter in `Outfit SemiBold 48sp`: `"14,280 Minutes of Pure Music"`.
   - Comparison: *"That's equal to 238 hours of studio-grade sound"*.
2. **Slide 2 — Top Tracks**:
   - Number 1 track spotlight with artwork, play count, and first date discovered.
   - Ranked list: Tracks #1 through #5 with mini audio snippets.
3. **Slide 3 — Top Artists**:
   - Top 5 artist portraits in floating hexagonal frosted glass cards.
   - Percentage breakdown of top listening share.
4. **Slide 4 — Listening Clock**:
   - Circular 24-hour heat ring showing when the user streams music most (e.g., peak at 11:30 PM).
   - Favorite listening day: `"Saturday Night"`.
5. **Slide 5 — The Audio Persona**:
   - Evaluates listening clusters to award a custom badge:
     - *"The Night Owl"* (70%+ listening after 10 PM)
     - *"The Sonic Explorer"* (High ratio of unique artists)
     - *"The Acoustic Dreamer"* (Dominant cozy/chill tags)
   - Share button: Exports a clean glassmorphic `.png` card ready for sharing.

---

### Screen 8: Sleep Timer & Night Mode Modal

Designed specifically to avoid waking or blinding the user in a dark room.

- **Presentation**: Half-height frosted glass bottom sheet (`backdrop blur: 24dp`).
- **Options**:
  - `[ 15 Minutes ]`
  - `[ 30 Minutes ]`
  - `[ 45 Minutes ]`
  - `[ 1 Hour ]`
  - `[ End of This Song ]`
- **Graceful Fade-Out Curve**:
  - Checkbox enabled by default: *"Graceful 30-second volume fade-out before stopping"*.
  - Prevents the abrupt cutoff that jolts listeners awake.
- **Active State Display**:
  - Subdued countdown timer in the bottom player bar (`🌙 18:42`).
  - 1-tap `[ +5 min ]` extension button.

---

## 3. Desktop Adaptive Layout (Compose for Desktop / Linux)

On large screens (Linux / macOS / Windows), the app transitions from a mobile bottom-nav layout to a spacious studio dashboard:

```
┌──────────────┬────────────────────────────────────────────────────────┐
│ SOFTIFY      │  🔍 Search songs, artists, links...        [⚙️] [⬆ v1.0]│
├──────────────┼────────────────────────────────────────────────────────┤
│ 🏠 Home      │  GOOD EVENING, SARTHAK                                 │
│ 🔍 Search    │  ┌──────────────────────────────────────────────────┐  │
│ 📚 Library   │  │  DAILY MIX 1                     DAILY MIX 2     │  │
│ ♥ Liked      │  │  [ The Weeknd, Daft Punk ]       [ Taylor Swift ]│  │
│ 📜 Playlists │  └──────────────────────────────────────────────────┘  │
│              │                                                        │
│ PLAYLISTS    │  NOW PLAYING QUEUE                                     │
│ • Late Night │  1. Blinding Lights • The Weeknd              03:20    │
│ • Deep Focus │  2. Save Your Tears • The Weeknd              03:35    │
│ • Cozy Coffee│  3. Starboy • The Weeknd, Daft Punk           03:50    │
├──────────────┴────────────────────────────────────────────────────────┤
│ [Art] Blinding Lights  [⏮] [ ▶ ] [⏭]   ──●────────────── 01:14/03:20  │
│       The Weeknd • 320k              [🎚️ EQ] [📜] [🌙] [🔊 ──────●]  │
└────────────────────────────────────────────────────────────────────────┘
```

- **Left Navigation Rail (`width: 240dp`)**: Frosted acrylic panel with pinned navigation links and custom playlists list.
- **Center Content Canvas**: Scrollable grid adapting from 2 columns (small window) to 4 columns (ultrawide monitor).
- **Persistent Bottom Player Bar (`height: 80dp`)**: Always-accessible playback controls, scrub bar, volume slider, and MPRIS D-Bus engine integration.
- **Linux MPRIS Integration**: Reflects full metadata, cover art URL, and play/pause/next commands to the desktop system tray and lock screen (GNOME / KDE / Hyprland).

---

## 4. Accessibility & Micro-Interaction Standards

1. **Contrast Ratio Compliance**:
   - All primary text (`#FAF7F2`) on glass surfaces (`#161524`) achieves a contrast ratio of **12.4:1**, well exceeding the WCAG AAA requirement (7:1).
   - Secondary text (`#A8A29E`) achieves **5.2:1**, exceeding WCAG AA (4.5:1).
2. **Touch Targets**:
   - Every interactive icon, button, and slider thumb has a minimum hit target of **48dp × 48dp**, even if the visual icon is 24dp.
3. **Motion & Transitions**:
   - Play/Pause transition: `150ms` spring easing.
   - Screen cross-fades: `250ms` cubic-bezier (`0.2, 0.0, 0.0, 1.0`).
   - Full support for system **prefers-reduced-motion**: Disables ambient gradient orbs and simplifies transitions to instant opacity fades.
4. **Haptic Feedback (Android)**:
   - Subtle tactile tick on slider scrubbing.
   - Satisfying soft confirmation pulse on Like/Mute action.

---

## 5. Technical Implementation Blueprint (Kotlin Compose)

### 5.1 Color Scheme in Compose
```kotlin
object SoftifyTheme {
    val BackgroundBase = Color(0xFF0A0A12)
    val GlassSurface = Color(0xBF161524)
    val GlassCard = Color(0x801E1C30)
    val AccentAmber = Color(0xFFF59E0B)
    val AccentStudioGreen = Color(0xFF10B981)
    val AccentTerracotta = Color(0xFFE07A5F)
    val TextPrimary = Color(0xFFFAF7F2)
    val TextSecondary = Color(0xFFA8A29E)
    val GlassBorder = Color(0x1AFFFFFF)
}
```

### 5.2 Reusable Glass Card Modifier
```kotlin
@Composable
fun Modifier.glassmorphism(
    cornerRadius: Dp = 20.dp,
    borderAlpha: Float = 0.12f,
    backgroundColor: Color = SoftifyTheme.GlassSurface
): Modifier = this
    .clip(RoundedCornerShape(cornerRadius))
    .background(backgroundColor)
    .border(
        width = 1.dp,
        brush = Brush.linearGradient(
            colors = listOf(
                Color.White.copy(alpha = borderAlpha),
                Color.White.copy(alpha = borderAlpha * 0.2f)
            )
        ),
        shape = RoundedCornerShape(cornerRadius)
    )
```

---

*This specification serves as the official, permanent UI/UX architecture document for Softify Next-Gen.*
