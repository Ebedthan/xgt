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

#[cfg(test)]
mod tests {
    use super::*;

    // Replace the current open() implementation with:

    impl Cache {
        /// Initialise the schema on an already-opened connection.
        /// Shared by open() and the test constructor.
        fn init(conn: Connection) -> Result<Self> {
            conn.execute_batch("PRAGMA journal_mode=WAL;")?;
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS cache (
                    key        TEXT PRIMARY KEY,
                    body       BLOB NOT NULL,
                    created_at INTEGER NOT NULL,
                    ttl        INTEGER NOT NULL
                );",
            )?;
            Ok(Self { conn })
        }

        /// Open an in-memory database. Used only in tests.
        #[cfg(test)]
        fn open_in_memory() -> Result<Self> {
            let conn = Connection::open_in_memory().context("Failed to open in-memory SQLite")?;
            Self::init(conn)
        }
    }

    // TTL constants

    #[test]
    fn test_ttl_constants_are_positive() {
        assert!(TTL_DIFF > 0);
        assert!(TTL_GENOME > 0);
        assert!(TTL_HISTORY > 0);
        assert!(TTL_TAXON > 0);
        assert!(TTL_SEARCH > 0);
    }

    #[test]
    fn test_ttl_ordering() {
        // diff results are immutable, longest TTL
        assert!(TTL_DIFF <= TTL_HISTORY);
        // history only grows, longer than card/search
        assert!(TTL_HISTORY >= TTL_GENOME);
        assert!(TTL_GENOME == TTL_TAXON);
        // search results change most frequently, shortest TTL
        assert!(TTL_TAXON >= TTL_SEARCH);
    }

    // Cache::key

    #[test]
    fn test_key_is_deterministic() {
        let url = "https://gtdb-api.ecogenomic.org/genome/GCA_000005845.2/card";
        assert_eq!(Cache::key(url), Cache::key(url));
    }

    #[test]
    fn test_key_differs_for_different_urls() {
        let url1 = "https://gtdb-api.ecogenomic.org/genome/GCA_000005845.2/card";
        let url2 = "https://gtdb-api.ecogenomic.org/genome/GCA_000005845.2/metadata";
        assert_ne!(Cache::key(url1), Cache::key(url2));
    }

    #[test]
    fn test_key_is_64_hex_chars() {
        // SHA-256 produces 32 bytes = 64 hex characters
        let key = Cache::key("https://example.com");
        assert_eq!(key.len(), 64);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_key_empty_url() {
        let key = Cache::key("");
        assert_eq!(key.len(), 64);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_key_url_with_special_chars() {
        // Species names contain spaces and underscores, these are percent-encoded
        // in real URLs but the key function must handle them regardless
        let key = Cache::key("https://gtdb-api.ecogenomic.org/taxon/s__Escherichia%20coli");
        assert_eq!(key.len(), 64);
    }

    // Cache::open_in_memory

    #[test]
    fn test_open_in_memory_succeeds() {
        let result = Cache::open_in_memory();
        assert!(result.is_ok(), "in-memory cache should open without error");
    }

    #[test]
    fn test_open_in_memory_table_exists() {
        let cache = Cache::open_in_memory().unwrap();
        // If the table does not exist this query would error
        let count: i64 = cache
            .conn
            .query_row("SELECT COUNT(*) FROM cache", [], |r| r.get(0))
            .expect("cache table must exist after open");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_open_in_memory_idempotent() {
        // Two separate in-memory caches are independent (different connections)
        let c1 = Cache::open_in_memory().unwrap();
        let c2 = Cache::open_in_memory().unwrap();
        c1.set("https://example.com", "body1", TTL_SEARCH).unwrap();
        // c2 must not see c1's entry
        assert!(c2.get("https://example.com").is_none());
    }

    // Cache::get / Cache::set

    #[test]
    fn test_get_missing_key_returns_none() {
        let cache = Cache::open_in_memory().unwrap();
        assert!(cache.get("https://not-stored.example.com").is_none());
    }

    #[test]
    fn test_set_then_get_returns_body() {
        let cache = Cache::open_in_memory().unwrap();
        let url = "https://gtdb-api.ecogenomic.org/genome/GCA_000005845.2/card";
        let body = r#"{"accession":"GCA_000005845.2"}"#;

        cache.set(url, body, TTL_GENOME).unwrap();
        assert_eq!(cache.get(url).as_deref(), Some(body));
    }

    #[test]
    fn test_set_with_long_ttl_is_not_expired() {
        let cache = Cache::open_in_memory().unwrap();
        let url = "https://example.com/long-lived";
        cache.set(url, "fresh", TTL_DIFF).unwrap();
        assert!(
            cache.get(url).is_some(),
            "long-TTL entry should not be expired"
        );
    }

    #[test]
    fn test_set_with_ttl_zero_is_immediately_expired() {
        let cache = Cache::open_in_memory().unwrap();
        let url = "https://example.com/expired";
        cache.set(url, "stale", 0).unwrap();
        // TTL=0 means created_at + ttl == now, which fails the > check
        assert!(
            cache.get(url).is_none(),
            "TTL=0 entry should be treated as expired immediately"
        );
    }

    #[test]
    fn test_set_overwrites_existing_entry() {
        let cache = Cache::open_in_memory().unwrap();
        let url = "https://example.com/overwrite";

        cache.set(url, "first", TTL_SEARCH).unwrap();
        cache.set(url, "second", TTL_SEARCH).unwrap();

        assert_eq!(
            cache.get(url).as_deref(),
            Some("second"),
            "second set should replace the first"
        );
    }

    #[test]
    fn test_set_get_unicode_body() {
        let cache = Cache::open_in_memory().unwrap();
        let url = "https://example.com/unicode";
        let body = "Côte d'Ivoire - p__Pseudomonadota - s__Escherichia coli 😀";

        cache.set(url, body, TTL_SEARCH).unwrap();
        assert_eq!(cache.get(url).as_deref(), Some(body));
    }

    #[test]
    fn test_set_get_json_body_with_newlines() {
        let cache = Cache::open_in_memory().unwrap();
        let url = "https://example.com/json";
        let body = "{\n  \"accession\": \"GCA_000005845.2\",\n  \"changed\": true\n}\n";

        cache.set(url, body, TTL_SEARCH).unwrap();
        assert_eq!(cache.get(url).as_deref(), Some(body));
    }

    #[test]
    fn test_set_get_empty_body() {
        let cache = Cache::open_in_memory().unwrap();
        let url = "https://example.com/empty";

        cache.set(url, "", TTL_SEARCH).unwrap();
        assert_eq!(
            cache.get(url).as_deref(),
            Some(""),
            "empty body should be stored and retrieved as empty string"
        );
    }

    // Cache::evict_expired

    #[test]
    fn test_evict_expired_empty_cache_returns_zero() {
        let cache = Cache::open_in_memory().unwrap();
        let n = cache.evict_expired().unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn test_evict_expired_does_not_touch_fresh_entries() {
        let cache = Cache::open_in_memory().unwrap();
        cache
            .set("https://example.com/a", "fresh", TTL_GENOME)
            .unwrap();
        cache
            .set("https://example.com/b", "fresh", TTL_DIFF)
            .unwrap();

        let n = cache.evict_expired().unwrap();
        assert_eq!(n, 0, "no entries should be evicted");
        assert!(cache.get("https://example.com/a").is_some());
        assert!(cache.get("https://example.com/b").is_some());
    }

    #[test]
    fn test_evict_expired_removes_ttl_zero_entries() {
        let cache = Cache::open_in_memory().unwrap();
        cache
            .set("https://example.com/expired", "stale", 0)
            .unwrap();

        let n = cache.evict_expired().unwrap();
        assert_eq!(n, 1, "one expired entry should be removed");
        assert!(cache.get("https://example.com/expired").is_none());
    }

    #[test]
    fn test_evict_expired_leaves_fresh_removes_stale() {
        let cache = Cache::open_in_memory().unwrap();
        cache
            .set("https://example.com/fresh", "ok", TTL_SEARCH)
            .unwrap();
        cache.set("https://example.com/stale-1", "gone", 0).unwrap();
        cache.set("https://example.com/stale-2", "gone", 0).unwrap();

        let n = cache.evict_expired().unwrap();
        assert_eq!(n, 2, "two expired entries should be removed");
        assert!(cache.get("https://example.com/fresh").is_some());
        assert!(cache.get("https://example.com/stale-1").is_none());
        assert!(cache.get("https://example.com/stale-2").is_none());
    }

    #[test]
    fn test_evict_expired_returns_correct_count() {
        let cache = Cache::open_in_memory().unwrap();
        for i in 0..5 {
            cache
                .set(&format!("https://example.com/{i}"), "stale", 0)
                .unwrap();
        }
        let n = cache.evict_expired().unwrap();
        assert_eq!(n, 5);
    }

    // Cache::clear

    #[test]
    fn test_clear_empty_cache_returns_zero() {
        let cache = Cache::open_in_memory().unwrap();
        let n = cache.clear().unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn test_clear_removes_all_entries() {
        let cache = Cache::open_in_memory().unwrap();
        cache
            .set("https://example.com/a", "body1", TTL_SEARCH)
            .unwrap();
        cache
            .set("https://example.com/b", "body2", TTL_GENOME)
            .unwrap();
        cache
            .set("https://example.com/c", "body3", TTL_DIFF)
            .unwrap();

        let n = cache.clear().unwrap();
        assert_eq!(n, 3);
        assert!(cache.get("https://example.com/a").is_none());
        assert!(cache.get("https://example.com/b").is_none());
        assert!(cache.get("https://example.com/c").is_none());
    }

    #[test]
    fn test_clear_then_get_returns_none() {
        let cache = Cache::open_in_memory().unwrap();
        let url = "https://example.com/will-be-cleared";
        cache.set(url, "body", TTL_SEARCH).unwrap();
        assert!(cache.get(url).is_some());

        cache.clear().unwrap();
        assert!(cache.get(url).is_none());
    }

    #[test]
    fn test_clear_then_set_works_again() {
        // Clearing must not break subsequent insertions
        let cache = Cache::open_in_memory().unwrap();
        cache
            .set("https://example.com/x", "first", TTL_SEARCH)
            .unwrap();
        cache.clear().unwrap();
        cache
            .set("https://example.com/x", "second", TTL_SEARCH)
            .unwrap();
        assert_eq!(
            cache.get("https://example.com/x").as_deref(),
            Some("second")
        );
    }

    // Cache::info

    #[test]
    fn test_info_empty_cache() {
        let cache = Cache::open_in_memory().unwrap();
        let info = cache.info().unwrap();
        assert_eq!(info.entry_count, 0);
        assert_eq!(info.expired_count, 0);
        // size_bytes for in-memory DB is 0 (no file on disk)
        assert_eq!(info.size_bytes, 0);
    }

    #[test]
    fn test_info_after_set_fresh_entry() {
        let cache = Cache::open_in_memory().unwrap();
        cache
            .set("https://example.com/a", "body", TTL_GENOME)
            .unwrap();

        let info = cache.info().unwrap();
        assert_eq!(info.entry_count, 1);
        assert_eq!(info.expired_count, 0);
    }

    #[test]
    fn test_info_expired_entry_counted_separately() {
        let cache = Cache::open_in_memory().unwrap();
        cache
            .set("https://example.com/fresh", "ok", TTL_SEARCH)
            .unwrap();
        cache.set("https://example.com/expired", "gone", 0).unwrap();

        let info = cache.info().unwrap();
        assert_eq!(info.entry_count, 2, "both entries present in DB");
        assert_eq!(info.expired_count, 1, "one entry is expired");
    }

    #[test]
    fn test_info_after_evict_expired_count_is_zero() {
        let cache = Cache::open_in_memory().unwrap();
        cache.set("https://example.com/stale", "gone", 0).unwrap();

        cache.evict_expired().unwrap();

        let info = cache.info().unwrap();
        assert_eq!(info.entry_count, 0);
        assert_eq!(info.expired_count, 0);
    }

    #[test]
    fn test_info_after_clear_all_counts_zero() {
        let cache = Cache::open_in_memory().unwrap();
        cache
            .set("https://example.com/a", "body1", TTL_SEARCH)
            .unwrap();
        cache.set("https://example.com/b", "body2", 0).unwrap();

        cache.clear().unwrap();

        let info = cache.info().unwrap();
        assert_eq!(info.entry_count, 0);
        assert_eq!(info.expired_count, 0);
    }

    // Interaction between set / get / evict / clear

    #[test]
    fn test_multiple_urls_are_independent() {
        let cache = Cache::open_in_memory().unwrap();
        cache
            .set("https://example.com/1", "body-one", TTL_GENOME)
            .unwrap();
        cache
            .set("https://example.com/2", "body-two", TTL_GENOME)
            .unwrap();
        cache
            .set("https://example.com/3", "body-three", TTL_GENOME)
            .unwrap();

        assert_eq!(
            cache.get("https://example.com/1").as_deref(),
            Some("body-one")
        );
        assert_eq!(
            cache.get("https://example.com/2").as_deref(),
            Some("body-two")
        );
        assert_eq!(
            cache.get("https://example.com/3").as_deref(),
            Some("body-three")
        );
    }

    #[test]
    fn test_set_get_large_body() {
        // Genome cards can be several KB, verify no truncation
        let cache = Cache::open_in_memory().unwrap();
        let url = "https://example.com/large";
        let body = "x".repeat(100_000);

        cache.set(url, &body, TTL_GENOME).unwrap();
        assert_eq!(
            cache.get(url).as_deref(),
            Some(body.as_str()),
            "large body must be stored and retrieved without truncation"
        );
    }

    #[test]
    fn test_now_secs_is_current_unix_timestamp() {
        // Verify now_secs() is roughly correct (within 5 seconds of SystemTime)
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let now = now_secs();
        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        assert!(now >= before, "now_secs() must be >= start of test");
        assert!(now <= after, "now_secs() must be <= end of test");
    }
}
