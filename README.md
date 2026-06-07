# HiFi Crystal Music Player

A Windows desktop HiFi music player built with Rust + Dioxus, featuring lossless audio support, spectrum analysis, lyrics display, and multi-theme UI.

## Features

- **Lossless Audio Playback** — Supports FLAC, WAV, MP3, OGG, M4A, AAC, ALAC, AIFF, DSD and more
- **Dual Audio Backends** — Rodio (default) + WASAPI exclusive mode for different audio devices
- **Spectrum Analysis** — 64-band real-time FFT spectrum visualization
- **Lyrics Display** — Supports LRC external lyrics and embedded lyrics auto-loading
- **Three Themes** — Aqua Glass (crystal), Sonic Flux (cyberpunk), Vintage Pro (retro HiFi)
- **Music Library Management** — SQLite persistence, batch folder import, playlists, play history
- **Drag & Drop Import** — Drag audio files directly into the window
- **Play Modes** — Sequential, repeat one, repeat all, shuffle
- **Audio Filters** — FIR filter configuration
- **Output Device Switching** — Select audio output device on the fly

## Project Architecture

```
musicplayer/
├── Cargo.toml              # Workspace root config
├── audio/                  # Audio engine module
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Module entry, exports Player / PlaybackMode
│       ├── player.rs       # Unified Player interface (thread-safe handle)
│       ├── backend.rs      # Backend abstraction (Rodio / WASAPI dispatch)
│       ├── playback.rs     # Rodio playback core: decode, queue, progress
│       ├── wasapi_backend.rs  # WASAPI exclusive mode backend
│       ├── wasapi_control.rs  # WASAPI device control
│       ├── decoder.rs      # Generic decoder interface
│       ├── symphonia_decoder.rs  # Symphonia decoder implementation
│       ├── dsd_decoder.rs  # DSD format decoder
│       ├── metadata.rs     # Audio metadata reader (audiotags + lofty)
│       ├── spectrum.rs     # FFT spectrum analyzer
│       ├── filter.rs       # FIR filter configuration
│       ├── volume.rs       # Volume control
│       ├── pipeline.rs     # Audio processing pipeline
│       ├── device.rs       # Audio device enumeration
│       └── error.rs        # Error type definitions
├── db/                     # Database module
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # SQLite operations (song CRUD, playlists, history)
│       └── schema.sql      # Database schema reference
└── ui-app/                 # Desktop UI application
    ├── Cargo.toml
    └── src/
        ├── main.rs         # App entry: window config, state management, event loop
        ├── theme.rs        # Theme system: StyleSheet + ThemeManager + 3 themes
        ├── lyrics_parser.rs # LRC lyrics parser
        ├── import.rs       # Music folder import (native dialog + batch scan)
        ├── aqua/           # Aqua Glass theme UI
        ├── sonic/          # Sonic Flux theme UI
        ├── vintage/        # Vintage Pro theme UI
        ├── components/     # Shared UI components
        │   ├── album_cover.rs  # Album cover
        │   ├── now_playing.rs  # Now playing
        │   ├── player_bar.rs   # Player control bar
        │   ├── song_list.rs    # Song list
        │   ├── title_bar.rs    # Title bar
        │   └── vu_meter.rs     # VU meter / spectrum display
        └── styles/         # Theme style definitions
            ├── aqua_glass.rs   # Aqua Glass styles
            ├── sonic_flux.rs   # Sonic Flux styles
            └── vintage_pro.rs  # Vintage Pro styles
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| UI Framework | Dioxus 0.7 (Desktop) |
| Audio Decoding | Symphonia + Rodio |
| Metadata | audiotags + lofty |
| Database | rusqlite (SQLite) |
| Spectrum Analysis | rustfft |
| Exclusive Audio | WASAPI (windows-rs) |
| Async Runtime | Tokio |

## Build & Run

### Prerequisites

- Rust 1.85+ (stable)
- Windows 10/11
- C++ Build Tools (required for rusqlite compilation)

### Build

```bash
# Debug mode
cargo build

# Release mode
cargo build --release
```

### Run

```bash
cargo run --bin ui-app
```

### Launch with file

```bash
cargo run --bin ui-app -- "C:\path\to\song.flac"
```

## Usage

1. **Import Music** — Click the import button, select a music folder, and it will be scanned automatically
2. **Playback Controls** — Play/pause, previous/next, seek, volume adjustment
3. **Switch Themes** — Toggle between Aqua Glass / Sonic Flux / Vintage Pro in the title bar
4. **Lyrics** — Place `.lrc` files in the same directory as audio files for auto-loading
5. **Drag & Drop** — Drag audio files directly into the window to import and play

## License

MIT
