-- db/src/schema.sql
CREATE TABLE IF NOT EXISTS artists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    bio TEXT,
    cover_url TEXT
);

CREATE TABLE IF NOT EXISTS albums (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    artist_id INTEGER NOT NULL REFERENCES artists(id),
    release_year INTEGER,
    cover_url TEXT,
    UNIQUE(title, artist_id)
);

CREATE TABLE IF NOT EXISTS songs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    album_id INTEGER REFERENCES albums(id),
    duration_secs INTEGER,
    audio_format TEXT CHECK(audio_format IN ('FLAC', 'WAV', 'DTS')),
    bitrate INTEGER,
    sample_rate INTEGER,
    channels INTEGER,
    file_url TEXT NOT NULL UNIQUE,
    local_path TEXT,
    quality_tag TEXT
);

CREATE TABLE IF NOT EXISTS playlists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS playlist_songs (
    playlist_id INTEGER NOT NULL REFERENCES playlists(id),
    song_id INTEGER NOT NULL REFERENCES songs(id),
    position INTEGER NOT NULL,
    PRIMARY KEY(playlist_id, song_id)
);

CREATE TABLE IF NOT EXISTS play_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    song_id INTEGER NOT NULL REFERENCES songs(id),
    played_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(song_id, played_at)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_songs_title ON songs(title);
CREATE INDEX IF NOT EXISTS idx_songs_album_id ON songs(album_id);
CREATE INDEX IF NOT EXISTS idx_albums_artist_id ON albums(artist_id);