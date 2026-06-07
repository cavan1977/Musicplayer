//! 数据库模块：SQLite 持久化
use chrono::{DateTime, Utc};
use rusqlite::{self, Connection, Result as RusResult, params};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 歌曲记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub id: Option<i64>,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub file_path: String,
    pub duration_secs: f64,
    pub quality: String,
    pub cover_url: Option<String>,
    pub cdn_url: Option<String>,
    pub date_added: DateTime<Utc>,
}

impl Song {
    fn from_row(row: &rusqlite::Row) -> RusResult<Self> {
        let date_str: String = row.get(9)?;
        let date_added = DateTime::parse_from_rfc3339(&date_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        Ok(Song {
            id: Some(row.get(0)?),
            title: row.get(1)?,
            artist: row.get(2)?,
            album: row.get(3)?,
            file_path: row.get(4)?,
            duration_secs: row.get(5)?,
            quality: row.get(6)?,
            cover_url: row.get(7)?,
            cdn_url: row.get(8)?,
            date_added,
        })
    }
}

/// 数据库
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> RusResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> RusResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS songs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT NOT NULL,
                file_path TEXT NOT NULL UNIQUE,
                duration_secs REAL NOT NULL,
                quality TEXT NOT NULL,
                cover_url TEXT,
                cdn_url TEXT,
                date_added TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        // History table for play counts and timestamps
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                song_id INTEGER PRIMARY KEY,
                played_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                play_count INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
            )",
            [],
        )?;
        // Playlists table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS playlists (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        // Playlist songs junction table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS playlist_songs (
                playlist_id INTEGER NOT NULL,
                song_id INTEGER NOT NULL,
                position INTEGER NOT NULL,
                added_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (playlist_id, song_id),
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
            )",
            [],
        )?;
        Ok(())
    }

    pub fn add_song(&self, song: &Song) -> RusResult<i64> {
        self.conn.execute(
            "INSERT INTO songs (title, artist, album, file_path, duration_secs, quality, cover_url, cdn_url, date_added)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                song.title,
                song.artist,
                song.album,
                song.file_path,
                song.duration_secs,
                song.quality,
                song.cover_url,
                song.cdn_url,
                song.date_added.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Update cover_url for an existing song by file_path.
    pub fn update_song_cover(&self, file_path: &str, cover_url: Option<&str>, duration_secs: Option<f64>) -> RusResult<()> {
        self.conn.execute(
            "UPDATE songs SET cover_url = ?1, duration_secs = COALESCE(?2, duration_secs) WHERE file_path = ?3",
            params![cover_url, duration_secs, file_path],
        )?;
        Ok(())
    }

    pub fn get_song_by_path(&self, path: &str) -> RusResult<Option<Song>> {
        let mut stmt = self.conn.prepare("SELECT * FROM songs WHERE file_path = ?1")?;
        let mut rows = stmt.query([path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Song::from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    pub fn list_songs(&self) -> RusResult<Vec<Song>> {
        let mut stmt = self.conn.prepare("SELECT * FROM songs ORDER BY date_added DESC")?;
        let mut rows = stmt.query([])?;
        let mut songs = Vec::new();
        while let Some(row) = rows.next()? {
            songs.push(Song::from_row(&row)?);
        }
        Ok(songs)
    }

    // History
    pub fn record_play(&self, song_id: i64) -> RusResult<()> {
        self.conn.execute(
            "INSERT INTO history (song_id, played_at, play_count) VALUES (?1, CURRENT_TIMESTAMP, 1)
             ON CONFLICT(song_id) DO UPDATE SET play_count = play_count + 1, played_at = CURRENT_TIMESTAMP",
            [song_id],
        )?;
        Ok(())
    }

    pub fn get_recent_history(&self, limit: i32) -> RusResult<Vec<(Song, i32)>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.*, h.play_count FROM songs s
             JOIN history h ON s.id = h.song_id
             ORDER BY h.played_at DESC
             LIMIT ?1"
        )?;
        let mut rows = stmt.query([limit])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            let song = Song {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                file_path: row.get(4)?,
                duration_secs: row.get(5)?,
                quality: row.get(6)?,
                cover_url: row.get(7)?,
                cdn_url: row.get(8)?,
                date_added: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            };
            let play_count: i32 = row.get(10)?;
            result.push((song, play_count));
        }
        Ok(result)
    }

    // Playlist operations
    pub fn create_playlist(&self, name: &str) -> RusResult<i64> {
        self.conn.execute(
            "INSERT INTO playlists (name) VALUES (?1)",
            [name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_playlists(&self) -> RusResult<Vec<(i64, String)>> {
        let mut stmt = self.conn.prepare("SELECT id, name FROM playlists ORDER BY name")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_playlist_by_name(&self, name: &str) -> RusResult<Option<(i64, String)>> {
        let mut stmt = self.conn.prepare("SELECT id, name FROM playlists WHERE name = ?1")?;
        let mut rows = stmt.query([name])?;
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?)))
        } else {
            Ok(None)
        }
    }

    /// Get or create a playlist by name. Returns the playlist ID.
    pub fn get_or_create_playlist(&self, name: &str) -> RusResult<i64> {
        match self.get_playlist_by_name(name)? {
            Some((id, _)) => Ok(id),
            None => self.create_playlist(name),
        }
    }

    pub fn add_song_to_playlist(&self, playlist_id: i64, song_id: i64) -> RusResult<()> {
        let pos: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(position), 0) FROM playlist_songs WHERE playlist_id = ?1",
            [playlist_id],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "INSERT OR IGNORE INTO playlist_songs (playlist_id, song_id, position) VALUES (?1, ?2, ?3)",
            params![playlist_id, song_id, pos + 1],
        )?;
        Ok(())
    }

    pub fn get_playlist_songs(&self, playlist_id: i64) -> RusResult<Vec<Song>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.* FROM songs s
             JOIN playlist_songs ps ON s.id = ps.song_id
             WHERE ps.playlist_id = ?1
             ORDER BY ps.position"
        )?;
        let mut rows = stmt.query([playlist_id])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            let song = Song {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                file_path: row.get(4)?,
                duration_secs: row.get(5)?,
                quality: row.get(6)?,
                cover_url: row.get(7)?,
                cdn_url: row.get(8)?,
                date_added: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            };
            result.push(song);
        }
        Ok(result)
    }
}