// Transparent HTTP response cache backed by SQLite.
// Stores raw response bodies keyed on the SHA-256 of the request URL.
// Each entry has a TTL (seconds) after which it is treated as expired.
//
// The cache is a single file at:
//    Linux/macOS: ~/.cache/xgt/cache.db
//    Windows: %LOCALAPPDATA%\xgt\cache.db

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// TTL constants in seconds

/// diff entries are keyed on accession + two fixed release IDs.
/// The result never changes once both releases exit.
pub const TTL_DIFF: u64 = 365 * 3600; // 1 year

/// Genome cards and metadata change only when GTDB releases a new version,
/// which can happen a few times a year.
pub const TTL_GENOME: u64 = 30 * 24 * 3600; // 30 days

/// Taxon history only grows, entries are never removed, only added.
pub const TTL_HISTORY: u64 = 90 * 24 * 3600; // 90 days

/// Taxon name and lineage records are moderately stable.
pub const TTL_TAXON: u64 = 30 * 24 * 3600; // 30 days

/// Search results change as new genomes are added or removed.
pub const TTL_SEARCH: u64 = 7 * 24 * 3600; // 7 days

pub fn cache_path() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".cache"));
    base.join("xgt").join("cache.db")
}

pub struct Cache {
    conn: Connection,
}

impl Cache {
    /// Open (or create) the cache database.
    pub fn open() -> Result<Self> {
        let path = cache_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create cache directory")?;
        }

        let conn = Connection::open(&path)
            .with_context(|| format!("Failed to open cache at {}", path.display()))?;

        // WAL mode: allows concurrent reads alongside writes
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS cache (
                key TEXT PRIMARY KEY,
                body BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                ttl INTEGER NOT NULL
            );",
        )?;

        Ok(Self { conn })
    }

    /// Compute the cache key for a URL.
    /// Uses SHA-256 so the key is always a fixed-length hex string
    /// regardless of URL length or special characters.
    pub fn key(url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());

        let digest = hasher.finalize();
        hex::encode(digest)
    }

    /// Return the cached response body if present and not expired.
    pub fn get(&self, url: &str) -> Option<String> {
        let key = Self::key(url);
        let now = now_secs();

        self.conn
            .query_row(
                "SELECT body FROM cache
             WHERE key = ?1 AND (created_at + ttl) >?2",
                params![key, now],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }

    /// Store a response body in the cache with the given TTL.
    pub fn set(&self, url: &str, body: &str, ttl: u64) -> Result<()> {
        let key = Self::key(url);
        let now = now_secs();

        self.conn.execute(
            "INSERT OR REPLACE INTO cache (key, body, created_at, ttl)
             VALUES (?1, ?2, ?3, ?4)",
            params![key, body.as_bytes(), now, ttl as i64],
        )?;

        Ok(())
    }

    /// Delete all expired entries. Call periodically to reclaim space.
    pub fn evict_expired(&self) -> Result<usize> {
        let now = now_secs();
        let n = self.conn.execute(
            "DELETE FROM cache WHERE (created_at + ttl) <= ?1",
            params![now],
        )?;

        Ok(n)
    }

    /// Delete all cache entries.
    pub fn clear(&self) -> Result<usize> {
        let n = self.conn.execute("DELETE FROM cache", [])?;

        Ok(n)
    }

    /// Return cache statistics: entry count and approximate size on disk.
    pub fn info(&self) -> Result<CacheInfo> {
        let entry_count: u64 = self.conn.query_row("SELECT COUNT(*) FROM cache", [], |r| {
            Ok(r.get::<_, i64>(0)? as u64)
        })?;

        let expired_count: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM cache WHERE (created_at + ttl) <= ?1",
            params![now_secs()],
            |r| Ok(r.get::<_, i64>(0)? as u64),
        )?;

        let size_bytes = std::fs::metadata(cache_path())
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(CacheInfo {
            entry_count,
            expired_count,
            size_bytes,
        })
    }
}

pub struct CacheInfo {
    pub entry_count: u64,
    pub expired_count: u64,
    pub size_bytes: u64,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
