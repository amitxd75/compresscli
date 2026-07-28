//! Cache management for CompressCLI
//!
//! Provides signature-based caching to skip re-compressing files when the input file
//! (mtime + file size) and compression options have not changed.

use crate::core::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub input_size: u64,
    pub input_mtime_secs: u64,
    pub output_path: PathBuf,
    pub options_hash: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CacheData {
    pub entries: HashMap<String, CacheEntry>,
}

pub struct CacheManager {
    cache_path: PathBuf,
    data: CacheData,
    enabled: bool,
}

impl CacheManager {
    /// Creates or loads the CacheManager instance.
    /// Returns an error if the cache directory cannot be created.
    pub fn new(enabled: bool) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("compresscli");

        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        let cache_path = cache_dir.join("cache.json");
        let data = if enabled && cache_path.exists() {
            fs::read_to_string(&cache_path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            CacheData::default()
        };

        Ok(Self {
            cache_path,
            data,
            enabled,
        })
    }

    /// Generates a key for a file path, using canonical path resolution where possible.
    fn make_key(path: &Path) -> String {
        fs::canonicalize(path)
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .to_string()
    }

    /// Checks if a file compression task can be skipped due to a valid cache hit
    pub fn is_cached(&self, input_path: &Path, output_path: &Path, options_hash: &str) -> bool {
        if !self.enabled {
            return false;
        }

        if !output_path.exists() {
            return false;
        }

        let metadata = match fs::metadata(input_path) {
            Ok(m) => m,
            Err(_) => return false,
        };

        let key = Self::make_key(input_path);
        let canonical_out =
            fs::canonicalize(output_path).unwrap_or_else(|_| output_path.to_path_buf());

        if let Some(entry) = self.data.entries.get(&key) {
            let mtime_secs = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let entry_out_canonical =
                fs::canonicalize(&entry.output_path).unwrap_or_else(|_| entry.output_path.clone());

            return entry.input_size == metadata.len()
                && entry.input_mtime_secs == mtime_secs
                && entry_out_canonical == canonical_out
                && entry.options_hash == options_hash;
        }

        false
    }

    /// Records a successful compression task into the cache
    pub fn record(
        &mut self,
        input_path: &Path,
        output_path: &Path,
        options_hash: &str,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let metadata = fs::metadata(input_path)?;
        let mtime_secs = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let entry = CacheEntry {
            input_size: metadata.len(),
            input_mtime_secs: mtime_secs,
            output_path: output_path.to_path_buf(),
            options_hash: options_hash.to_string(),
        };

        let key = Self::make_key(input_path);
        self.data.entries.insert(key, entry);

        // Save updated cache to file
        if let Ok(content) = serde_json::to_string_pretty(&self.data) {
            let _ = fs::write(&self.cache_path, content);
        }

        Ok(())
    }

    /// Helper function to check cache hit and print notification by creating a temporary CacheManager instance.
    /// Note: Reads and parses the cache file on invocation. For batch processing, prefer using a shared `CacheManager` instance.
    pub fn check_cache_hit(input_path: &Path, output_path: &Path, options_hash: &str) -> bool {
        if let Ok(cache) = Self::new(true)
            && cache.is_cached(input_path, output_path, options_hash)
        {
            crate::ui::progress::print_badge(
                "CACHE",
                &format!("Compression skipped (cached): {}", output_path.display()),
            );
            return true;
        }
        false
    }

    /// Helper function to record cache entry silently by creating a temporary CacheManager instance.
    /// Note: Reads, mutates, and writes the cache file on invocation. For batch processing, prefer using a shared `CacheManager` instance.
    pub fn record_cache(input_path: &Path, output_path: &Path, options_hash: &str) {
        if let Ok(mut cache) = Self::new(true) {
            let _ = cache.record(input_path, output_path, options_hash);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cache_disabled() {
        let cache = CacheManager::new(false).unwrap();
        let file = NamedTempFile::new().unwrap();
        assert!(!cache.is_cached(file.path(), file.path(), "hash"));
    }

    #[test]
    fn test_cache_hit_and_miss() {
        let mut cache = CacheManager::new(true).unwrap();
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        let hash = "test_hash_123";
        assert!(!cache.is_cached(input_file.path(), output_file.path(), hash));

        cache
            .record(input_file.path(), output_file.path(), hash)
            .unwrap();

        assert!(cache.is_cached(input_file.path(), output_file.path(), hash));
        assert!(!cache.is_cached(input_file.path(), output_file.path(), "different_hash"));
    }
}
