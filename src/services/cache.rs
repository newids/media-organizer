use sqlx::{sqlite::{SqlitePool, SqliteConnectOptions, SqlitePoolOptions}, Row, Sqlite, Pool};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration};
use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

/// Cache service configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Database file path
    pub database_path: PathBuf,
    /// Maximum number of connections in pool
    pub max_connections: u32,
    /// Minimum number of connections in pool
    pub min_connections: u32,
    /// Connection idle timeout
    pub idle_timeout: Duration,
    /// Connection acquisition timeout
    pub acquire_timeout: Duration,
    /// Enable WAL mode for better concurrency
    pub enable_wal_mode: bool,
    /// Enable foreign key constraints
    pub enable_foreign_keys: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("cache.db"),
            max_connections: 10,
            min_connections: 1,
            idle_timeout: Duration::from_secs(600), // 10 minutes
            acquire_timeout: Duration::from_secs(30),
            enable_wal_mode: true,
            enable_foreign_keys: true,
        }
    }
}

impl CacheConfig {
    /// Create a new cache configuration with the specified database path
    pub fn new(database_path: impl AsRef<Path>) -> Self {
        Self {
            database_path: database_path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }
    
    /// Set maximum number of connections in pool
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }
    
    /// Set minimum number of connections in pool
    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = min;
        self
    }
    
    /// Set connection idle timeout
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }
    
    /// Set connection acquisition timeout
    pub fn acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = timeout;
        self
    }
    
    /// Enable or disable WAL mode
    pub fn wal_mode(mut self, enable: bool) -> Self {
        self.enable_wal_mode = enable;
        self
    }
    
    /// Enable or disable foreign key constraints
    pub fn foreign_keys(mut self, enable: bool) -> Self {
        self.enable_foreign_keys = enable;
        self
    }
}

/// Cache service errors
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cache entry not found: {0}")]
    NotFound(String),
    #[error("Invalid cache data: {0}")]
    InvalidData(String),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// File metadata cache entry
#[derive(Debug, Clone)]
pub struct CachedFileMetadata {
    pub id: String,
    pub path: PathBuf,
    pub size: u64,
    pub modified_time: SystemTime,
    pub hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CachedFileMetadata {
    /// Create a new metadata cache entry
    pub fn new(path: PathBuf, size: u64, modified_time: SystemTime, hash: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            path,
            size,
            modified_time,
            hash,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Update the metadata and refresh the updated_at timestamp
    pub fn update(&mut self, size: u64, modified_time: SystemTime, hash: Option<String>) {
        self.size = size;
        self.modified_time = modified_time;
        self.hash = hash;
        self.updated_at = Utc::now();
    }
}

/// Thumbnail cache entry
#[derive(Debug, Clone)]
pub struct CachedThumbnail {
    pub id: String,
    pub file_path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub created_at: DateTime<Utc>,
}

impl CachedThumbnail {
    /// Create a new thumbnail cache entry
    pub fn new(file_path: PathBuf, thumbnail_path: PathBuf) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            file_path,
            thumbnail_path,
            created_at: Utc::now(),
        }
    }
}

/// Cache service implementation with SQLite backend
#[derive(Debug, Clone)]
pub struct CacheService {
    pool: Pool<Sqlite>,
    config: CacheConfig,
}

impl CacheService {
    /// Create a new cache service with the specified database path using default configuration
    pub async fn new(database_path: impl AsRef<Path>) -> Result<Self, CacheError> {
        let config = CacheConfig::new(database_path);
        Self::new_with_config(config).await
    }
    
    /// Create a new cache service with custom configuration
    pub async fn new_with_config(config: CacheConfig) -> Result<Self, CacheError> {
        // Validate configuration
        if config.max_connections == 0 {
            return Err(CacheError::Configuration("max_connections must be greater than 0".to_string()));
        }
        if config.min_connections > config.max_connections {
            return Err(CacheError::Configuration("min_connections cannot be greater than max_connections".to_string()));
        }
        
        // Ensure the parent directory exists
        if let Some(parent) = config.database_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        // Create connection options with optimizations
        let mut connect_options = SqliteConnectOptions::new()
            .filename(&config.database_path)
            .create_if_missing(true);
        
        // Configure SQLite options based on configuration
        if config.enable_wal_mode {
            connect_options = connect_options.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        }
        
        if config.enable_foreign_keys {
            connect_options = connect_options.foreign_keys(true);
        }
        
        // Add performance optimizations
        connect_options = connect_options
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal) // Better performance than FULL
            .pragma("cache_size", "-64000") // 64MB cache
            .pragma("temp_store", "memory") // Store temp tables in memory
            .pragma("mmap_size", "268435456"); // 256MB memory map
        
        // Create connection pool with configuration
        let pool_options = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .idle_timeout(Some(config.idle_timeout))
            .acquire_timeout(config.acquire_timeout);
        
        let pool = pool_options.connect_with(connect_options).await?;
        
        let mut service = Self {
            pool,
            config,
        };
        
        // Run migrations on startup
        service.run_migrations().await?;
        
        tracing::info!(
            "Cache service initialized with {} max connections at {:?}",
            service.config.max_connections,
            service.config.database_path
        );
        
        Ok(service)
    }
    
    /// Get the cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
    
    /// Run database migrations to set up schema
    async fn run_migrations(&mut self) -> Result<(), CacheError> {
        self.create_schema().await?;
        self.create_indexes().await?;
        Ok(())
    }
    
    /// Create the database schema
    async fn create_schema(&self) -> Result<(), CacheError> {
        // Create file_metadata table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS file_metadata (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                size INTEGER NOT NULL,
                modified_time INTEGER NOT NULL,
                hash TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // Create thumbnail_cache table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS thumbnail_cache (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL UNIQUE,
                thumbnail_path TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // Create schema_version table for migration tracking
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // Insert initial schema version if not exists
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO schema_version (version, applied_at)
            VALUES (1, ?);
            "#
        )
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        
        tracing::info!("Database schema created successfully");
        Ok(())
    }
    
    /// Create performance indexes
    async fn create_indexes(&self) -> Result<(), CacheError> {
        // Index on path for fast lookups
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_file_metadata_path ON file_metadata(path);"
        )
        .execute(&self.pool)
        .await?;
        
        // Index on modified_time for cache invalidation
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_file_metadata_modified ON file_metadata(modified_time);"
        )
        .execute(&self.pool)
        .await?;
        
        // Index on thumbnail file_path for fast lookups
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_thumbnail_file_path ON thumbnail_cache(file_path);"
        )
        .execute(&self.pool)
        .await?;
        
        tracing::info!("Database indexes created successfully");
        Ok(())
    }
    
    /// Get the current schema version
    pub async fn get_schema_version(&self) -> Result<i64, CacheError> {
        let row = sqlx::query("SELECT MAX(version) as version FROM schema_version")
            .fetch_one(&self.pool)
            .await?;
        
        let version: Option<i64> = row.try_get("version")?;
        Ok(version.unwrap_or(0))
    }
    
    /// Store file metadata in cache
    pub async fn store_metadata(&self, metadata: &CachedFileMetadata) -> Result<(), CacheError> {
        let modified_timestamp = system_time_to_timestamp(metadata.modified_time);
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO file_metadata (
                id, path, size, modified_time, hash, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&metadata.id)
        .bind(metadata.path.to_string_lossy().as_ref())
        .bind(metadata.size as i64)
        .bind(modified_timestamp)
        .bind(&metadata.hash)
        .bind(metadata.created_at.to_rfc3339())
        .bind(metadata.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        
        tracing::debug!("Stored metadata for path: {:?}", metadata.path);
        Ok(())
    }
    
    /// Retrieve file metadata from cache by path
    pub async fn get_metadata(&self, path: &Path) -> Result<Option<CachedFileMetadata>, CacheError> {
        let path_str = path.to_string_lossy();
        
        let row = sqlx::query(
            r#"
            SELECT id, path, size, modified_time, hash, created_at, updated_at
            FROM file_metadata
            WHERE path = ?
            "#
        )
        .bind(path_str.as_ref())
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let modified_timestamp: i64 = row.try_get("modified_time")?;
            let created_at_str: String = row.try_get("created_at")?;
            let updated_at_str: String = row.try_get("updated_at")?;
            
            Ok(Some(CachedFileMetadata {
                id: row.try_get("id")?,
                path: PathBuf::from(row.try_get::<String, _>("path")?),
                size: row.try_get::<i64, _>("size")? as u64,
                modified_time: timestamp_to_system_time(modified_timestamp),
                hash: row.try_get("hash")?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| CacheError::InvalidData(format!("Invalid created_at timestamp: {}", e)))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|e| CacheError::InvalidData(format!("Invalid updated_at timestamp: {}", e)))?
                    .with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Check if metadata exists in cache and is fresh (not older than file modification time)
    pub async fn is_metadata_fresh(&self, path: &Path, file_modified: SystemTime) -> Result<bool, CacheError> {
        if let Some(cached) = self.get_metadata(path).await? {
            // Check if cached metadata is newer than or equal to file modification time
            Ok(cached.modified_time >= file_modified)
        } else {
            Ok(false)
        }
    }
    
    /// Check if metadata exists in cache for the given path
    pub async fn metadata_exists(&self, path: &Path) -> Result<bool, CacheError> {
        let path_str = path.to_string_lossy();
        
        let row = sqlx::query("SELECT 1 FROM file_metadata WHERE path = ?")
            .bind(path_str.as_ref())
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(row.is_some())
    }
    
    /// Store multiple file metadata entries in a single transaction for better performance
    pub async fn batch_store_metadata(&self, metadata_list: &[CachedFileMetadata]) -> Result<(), CacheError> {
        if metadata_list.is_empty() {
            return Ok(());
        }
        
        let mut tx = self.pool.begin().await?;
        
        for metadata in metadata_list {
            let modified_timestamp = system_time_to_timestamp(metadata.modified_time);
            
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO file_metadata (
                    id, path, size, modified_time, hash, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&metadata.id)
            .bind(metadata.path.to_string_lossy().as_ref())
            .bind(metadata.size as i64)
            .bind(modified_timestamp)
            .bind(&metadata.hash)
            .bind(metadata.created_at.to_rfc3339())
            .bind(metadata.updated_at.to_rfc3339())
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        tracing::debug!("Batch stored {} metadata entries", metadata_list.len());
        Ok(())
    }
    
    /// Remove metadata from cache by path
    pub async fn remove_metadata(&self, path: &Path) -> Result<bool, CacheError> {
        let path_str = path.to_string_lossy();
        
        let result = sqlx::query("DELETE FROM file_metadata WHERE path = ?")
            .bind(path_str.as_ref())
            .execute(&self.pool)
            .await?;
        
        let removed = result.rows_affected() > 0;
        if removed {
            tracing::debug!("Removed metadata for path: {:?}", path);
        }
        Ok(removed)
    }
    
    /// Get all metadata entries that match a path prefix (useful for directory operations)
    pub async fn get_metadata_by_prefix(&self, path_prefix: &Path) -> Result<Vec<CachedFileMetadata>, CacheError> {
        let prefix_str = format!("{}%", path_prefix.to_string_lossy());
        
        let rows = sqlx::query(
            r#"
            SELECT id, path, size, modified_time, hash, created_at, updated_at
            FROM file_metadata
            WHERE path LIKE ?
            ORDER BY path
            "#
        )
        .bind(&prefix_str)
        .fetch_all(&self.pool)
        .await?;
        
        let mut results = Vec::new();
        for row in rows {
            let modified_timestamp: i64 = row.try_get("modified_time")?;
            let created_at_str: String = row.try_get("created_at")?;
            let updated_at_str: String = row.try_get("updated_at")?;
            
            results.push(CachedFileMetadata {
                id: row.try_get("id")?,
                path: PathBuf::from(row.try_get::<String, _>("path")?),
                size: row.try_get::<i64, _>("size")? as u64,
                modified_time: timestamp_to_system_time(modified_timestamp),
                hash: row.try_get("hash")?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| CacheError::InvalidData(format!("Invalid created_at timestamp: {}", e)))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|e| CacheError::InvalidData(format!("Invalid updated_at timestamp: {}", e)))?
                    .with_timezone(&Utc),
            });
        }
        
        Ok(results)
    }
    
    /// Remove all metadata entries that match a path prefix (useful for directory deletions)
    pub async fn remove_metadata_by_prefix(&self, path_prefix: &Path) -> Result<usize, CacheError> {
        let prefix_str = format!("{}%", path_prefix.to_string_lossy());
        
        let result = sqlx::query("DELETE FROM file_metadata WHERE path LIKE ?")
            .bind(&prefix_str)
            .execute(&self.pool)
            .await?;
        
        let removed_count = result.rows_affected() as usize;
        if removed_count > 0 {
            tracing::debug!("Removed {} metadata entries with prefix: {:?}", removed_count, path_prefix);
        }
        Ok(removed_count)
    }
    
    /// Store thumbnail path in cache
    pub async fn store_thumbnail_path(&self, thumbnail: &CachedThumbnail) -> Result<(), CacheError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO thumbnail_cache (
                id, file_path, thumbnail_path, created_at
            ) VALUES (?, ?, ?, ?)
            "#
        )
        .bind(&thumbnail.id)
        .bind(thumbnail.file_path.to_string_lossy().as_ref())
        .bind(thumbnail.thumbnail_path.to_string_lossy().as_ref())
        .bind(thumbnail.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        
        tracing::debug!("Stored thumbnail path for file: {:?} -> {:?}", 
                       thumbnail.file_path, thumbnail.thumbnail_path);
        Ok(())
    }
    
    /// Retrieve thumbnail path from cache by file path
    pub async fn get_thumbnail_path(&self, file_path: &Path) -> Result<Option<CachedThumbnail>, CacheError> {
        let file_path_str = file_path.to_string_lossy();
        
        let row = sqlx::query(
            r#"
            SELECT id, file_path, thumbnail_path, created_at
            FROM thumbnail_cache
            WHERE file_path = ?
            "#
        )
        .bind(file_path_str.as_ref())
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let created_at_str: String = row.try_get("created_at")?;
            
            Ok(Some(CachedThumbnail {
                id: row.try_get("id")?,
                file_path: PathBuf::from(row.try_get::<String, _>("file_path")?),
                thumbnail_path: PathBuf::from(row.try_get::<String, _>("thumbnail_path")?),
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| CacheError::InvalidData(format!("Invalid created_at timestamp: {}", e)))?
                    .with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Check if thumbnail exists in cache and the thumbnail file still exists on filesystem
    pub async fn is_thumbnail_valid(&self, file_path: &Path) -> Result<bool, CacheError> {
        if let Some(cached_thumbnail) = self.get_thumbnail_path(file_path).await? {
            // Check if the cached thumbnail file still exists on filesystem
            Ok(cached_thumbnail.thumbnail_path.exists())
        } else {
            Ok(false)
        }
    }
    
    /// Check if thumbnail cache entry exists for the given file path
    pub async fn thumbnail_exists(&self, file_path: &Path) -> Result<bool, CacheError> {
        let file_path_str = file_path.to_string_lossy();
        
        let row = sqlx::query("SELECT 1 FROM thumbnail_cache WHERE file_path = ?")
            .bind(file_path_str.as_ref())
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(row.is_some())
    }
    
    /// Store multiple thumbnail entries in a single transaction for better performance
    pub async fn batch_store_thumbnail_paths(&self, thumbnails: &[CachedThumbnail]) -> Result<(), CacheError> {
        if thumbnails.is_empty() {
            return Ok(());
        }
        
        let mut tx = self.pool.begin().await?;
        
        for thumbnail in thumbnails {
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO thumbnail_cache (
                    id, file_path, thumbnail_path, created_at
                ) VALUES (?, ?, ?, ?)
                "#
            )
            .bind(&thumbnail.id)
            .bind(thumbnail.file_path.to_string_lossy().as_ref())
            .bind(thumbnail.thumbnail_path.to_string_lossy().as_ref())
            .bind(thumbnail.created_at.to_rfc3339())
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        tracing::debug!("Batch stored {} thumbnail entries", thumbnails.len());
        Ok(())
    }
    
    /// Remove thumbnail from cache by file path
    pub async fn remove_thumbnail(&self, file_path: &Path) -> Result<bool, CacheError> {
        let file_path_str = file_path.to_string_lossy();
        
        let result = sqlx::query("DELETE FROM thumbnail_cache WHERE file_path = ?")
            .bind(file_path_str.as_ref())
            .execute(&self.pool)
            .await?;
        
        let removed = result.rows_affected() > 0;
        if removed {
            tracing::debug!("Removed thumbnail cache entry for file: {:?}", file_path);
        }
        Ok(removed)
    }
    
    /// Get all thumbnail entries that match a file path prefix (useful for directory operations)
    pub async fn get_thumbnails_by_prefix(&self, path_prefix: &Path) -> Result<Vec<CachedThumbnail>, CacheError> {
        let prefix_str = format!("{}%", path_prefix.to_string_lossy());
        
        let rows = sqlx::query(
            r#"
            SELECT id, file_path, thumbnail_path, created_at
            FROM thumbnail_cache
            WHERE file_path LIKE ?
            ORDER BY file_path
            "#
        )
        .bind(&prefix_str)
        .fetch_all(&self.pool)
        .await?;
        
        let mut results = Vec::new();
        for row in rows {
            let created_at_str: String = row.try_get("created_at")?;
            
            results.push(CachedThumbnail {
                id: row.try_get("id")?,
                file_path: PathBuf::from(row.try_get::<String, _>("file_path")?),
                thumbnail_path: PathBuf::from(row.try_get::<String, _>("thumbnail_path")?),
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| CacheError::InvalidData(format!("Invalid created_at timestamp: {}", e)))?
                    .with_timezone(&Utc),
            });
        }
        
        Ok(results)
    }
    
    /// Remove all thumbnail entries that match a file path prefix (useful for directory deletions)
    pub async fn remove_thumbnails_by_prefix(&self, path_prefix: &Path) -> Result<usize, CacheError> {
        let prefix_str = format!("{}%", path_prefix.to_string_lossy());
        
        let result = sqlx::query("DELETE FROM thumbnail_cache WHERE file_path LIKE ?")
            .bind(&prefix_str)
            .execute(&self.pool)
            .await?;
        
        let removed_count = result.rows_affected() as usize;
        if removed_count > 0 {
            tracing::debug!("Removed {} thumbnail cache entries with prefix: {:?}", removed_count, path_prefix);
        }
        Ok(removed_count)
    }
    
    /// Clean up thumbnail cache entries where the thumbnail files no longer exist on filesystem
    pub async fn cleanup_invalid_thumbnails(&self) -> Result<usize, CacheError> {
        // Get all thumbnail entries
        let rows = sqlx::query("SELECT id, file_path, thumbnail_path FROM thumbnail_cache")
            .fetch_all(&self.pool)
            .await?;
        
        let mut invalid_ids = Vec::new();
        
        for row in rows {
            let thumbnail_path: String = row.try_get("thumbnail_path")?;
            let path = PathBuf::from(thumbnail_path);
            
            // Check if thumbnail file still exists
            if !path.exists() {
                let id: String = row.try_get("id")?;
                invalid_ids.push(id);
            }
        }
        
        if invalid_ids.is_empty() {
            return Ok(0);
        }
        
        // Remove invalid entries in batches to avoid SQL query length limits
        let mut removed_count = 0;
        for chunk in invalid_ids.chunks(100) {
            let placeholders = vec!["?"; chunk.len()].join(",");
            let sql = format!("DELETE FROM thumbnail_cache WHERE id IN ({})", placeholders);
            
            let mut query = sqlx::query(&sql);
            for id in chunk {
                query = query.bind(id);
            }
            
            let result = query.execute(&self.pool).await?;
            removed_count += result.rows_affected() as usize;
        }
        
        tracing::info!("Cleaned up {} invalid thumbnail cache entries", removed_count);
        Ok(removed_count)
    }
    
    /// Remove stale cache entries based on file modification times
    pub async fn remove_stale_entries(&self, max_age_seconds: i64) -> Result<CacheCleanupResult, CacheError> {
        let cutoff_time = Utc::now() - chrono::Duration::seconds(max_age_seconds);
        let cutoff_timestamp = cutoff_time.timestamp();
        
        // Clean up stale metadata entries
        let metadata_result = sqlx::query(
            "DELETE FROM file_metadata WHERE modified_time < ?"
        )
        .bind(cutoff_timestamp)
        .execute(&self.pool)
        .await?;
        
        let stale_metadata_count = metadata_result.rows_affected() as usize;
        
        // Clean up orphaned thumbnails (no corresponding metadata)
        let orphaned_thumbnails_count = self.cleanup_orphaned_thumbnails().await?;
        
        // Clean up invalid thumbnails (thumbnail files don't exist)
        let invalid_thumbnails_count = self.cleanup_invalid_thumbnails().await?;
        
        let result = CacheCleanupResult {
            stale_metadata_removed: stale_metadata_count,
            orphaned_thumbnails_removed: orphaned_thumbnails_count,
            invalid_thumbnails_removed: invalid_thumbnails_count,
            total_entries_removed: stale_metadata_count + orphaned_thumbnails_count + invalid_thumbnails_count,
        };
        
        tracing::info!(
            "Cache cleanup completed: {} stale metadata, {} orphaned thumbnails, {} invalid thumbnails", 
            stale_metadata_count, orphaned_thumbnails_count, invalid_thumbnails_count
        );
        
        Ok(result)
    }
    
    /// Clean up orphaned thumbnail entries that have no corresponding file metadata
    pub async fn cleanup_orphaned_thumbnails(&self) -> Result<usize, CacheError> {
        let result = sqlx::query(
            r#"
            DELETE FROM thumbnail_cache 
            WHERE file_path NOT IN (
                SELECT path FROM file_metadata
            )
            "#
        )
        .execute(&self.pool)
        .await?;
        
        let removed_count = result.rows_affected() as usize;
        if removed_count > 0 {
            tracing::debug!("Removed {} orphaned thumbnail entries", removed_count);
        }
        
        Ok(removed_count)
    }
    
    /// Vacuum the database to reclaim space and optimize performance
    pub async fn vacuum_database(&self) -> Result<(), CacheError> {
        tracing::info!("Starting database vacuum operation");
        
        // SQLite VACUUM command to reclaim space and defragment
        sqlx::query("VACUUM")
            .execute(&self.pool)
            .await?;
        
        // Analyze tables for query optimization
        sqlx::query("ANALYZE")
            .execute(&self.pool)
            .await?;
        
        tracing::info!("Database vacuum and analysis completed");
        Ok(())
    }
    
    /// Get comprehensive cache metrics and statistics
    pub async fn get_cache_metrics(&self) -> Result<CacheMetrics, CacheError> {
        // Get metadata cache stats
        let metadata_row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_entries,
                SUM(size) as total_size,
                AVG(size) as avg_size,
                MIN(created_at) as oldest_entry,
                MAX(created_at) as newest_entry
            FROM file_metadata
            "#
        )
        .fetch_one(&self.pool)
        .await?;
        
        // Get thumbnail cache stats
        let thumbnail_row = sqlx::query("SELECT COUNT(*) as total_entries FROM thumbnail_cache")
            .fetch_one(&self.pool)
            .await?;
        
        // Get database size information
        let db_size_row = sqlx::query("SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()")
            .fetch_one(&self.pool)
            .await?;
        
        let metadata_stats = MetadataCacheStats {
            total_entries: metadata_row.try_get::<i64, _>("total_entries")? as usize,
            total_size_bytes: metadata_row.try_get::<Option<i64>, _>("total_size")?.unwrap_or(0) as u64,
            average_file_size: metadata_row.try_get::<Option<f64>, _>("avg_size")?.unwrap_or(0.0) as u64,
            oldest_entry: metadata_row.try_get::<Option<String>, _>("oldest_entry")?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            newest_entry: metadata_row.try_get::<Option<String>, _>("newest_entry")?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
        };
        
        let thumbnail_stats = ThumbnailCacheStats {
            total_entries: thumbnail_row.try_get::<i64, _>("total_entries")? as usize,
        };
        
        let database_stats = DatabaseStats {
            size_bytes: db_size_row.try_get::<i64, _>("size")? as u64,
            database_path: self.config.database_path.clone(),
        };
        
        Ok(CacheMetrics {
            metadata_cache: metadata_stats,
            thumbnail_cache: thumbnail_stats,
            database: database_stats,
            last_updated: Utc::now(),
        })
    }
    
    /// Implement LRU eviction policy to maintain cache size limits
    pub async fn enforce_cache_size_limit(&self, max_entries: usize) -> Result<usize, CacheError> {
        // Get current cache size
        let current_count_row = sqlx::query("SELECT COUNT(*) as count FROM file_metadata")
            .fetch_one(&self.pool)
            .await?;
        let current_count: i64 = current_count_row.try_get("count")?;
        
        if (current_count as usize) <= max_entries {
            return Ok(0); // No eviction needed
        }
        
        let entries_to_remove = (current_count as usize) - max_entries;
        
        // Remove oldest entries (LRU - based on created_at timestamp)
        let result = sqlx::query(
            r#"
            DELETE FROM file_metadata 
            WHERE id IN (
                SELECT id FROM file_metadata 
                ORDER BY created_at ASC 
                LIMIT ?
            )
            "#
        )
        .bind(entries_to_remove as i64)
        .execute(&self.pool)
        .await?;
        
        let removed_count = result.rows_affected() as usize;
        
        // Clean up any orphaned thumbnails after metadata removal
        let orphaned_count = self.cleanup_orphaned_thumbnails().await?;
        
        tracing::info!(
            "LRU eviction completed: removed {} metadata entries and {} orphaned thumbnails", 
            removed_count, orphaned_count
        );
        
        Ok(removed_count)
    }
    
    /// Perform comprehensive cache maintenance
    pub async fn perform_maintenance(&self, config: &CacheMaintenanceConfig) -> Result<CacheMaintenanceResult, CacheError> {
        tracing::info!("Starting comprehensive cache maintenance");
        let start_time = std::time::Instant::now();
        
        // Step 1: Remove stale entries
        let cleanup_result = self.remove_stale_entries(config.max_age_seconds).await?;
        
        // Step 2: Enforce size limits
        let evicted_count = if let Some(max_entries) = config.max_entries {
            self.enforce_cache_size_limit(max_entries).await?
        } else {
            0
        };
        
        // Step 3: Vacuum database if requested
        if config.vacuum_database {
            self.vacuum_database().await?;
        }
        
        // Step 4: Get final metrics
        let final_metrics = self.get_cache_metrics().await?;
        
        let maintenance_duration = start_time.elapsed();
        
        let result = CacheMaintenanceResult {
            cleanup_result,
            evicted_entries: evicted_count,
            database_vacuumed: config.vacuum_database,
            final_metrics,
            maintenance_duration,
        };
        
        tracing::info!(
            "Cache maintenance completed in {:.2}s: {} total entries removed", 
            maintenance_duration.as_secs_f64(),
            result.cleanup_result.total_entries_removed + evicted_count
        );
        
        Ok(result)
    }
    
    /// Start a background task for scheduled cache maintenance
    pub fn start_scheduled_maintenance(
        &self, 
        interval_seconds: u64,
        config: CacheMaintenanceConfig,
    ) -> tokio::task::JoinHandle<()> {
        let cache_service = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_seconds));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            loop {
                interval.tick().await;
                
                tracing::debug!("Running scheduled cache maintenance");
                
                match cache_service.perform_maintenance(&config).await {
                    Ok(result) => {
                        tracing::info!(
                            "Scheduled maintenance completed: {} entries removed in {:.2}s",
                            result.cleanup_result.total_entries_removed + result.evicted_entries,
                            result.maintenance_duration.as_secs_f64()
                        );
                    }
                    Err(e) => {
                        tracing::error!("Scheduled cache maintenance failed: {}", e);
                    }
                }
            }
        })
    }
    
    /// Stop scheduled maintenance task
    pub fn stop_scheduled_maintenance(handle: tokio::task::JoinHandle<()>) {
        handle.abort();
        tracing::info!("Scheduled cache maintenance stopped");
    }
    
    /// Close the database connection pool
    pub async fn close(&self) {
        self.pool.close().await;
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<CacheStats, CacheError> {
        let metadata_count_row = sqlx::query("SELECT COUNT(*) as count FROM file_metadata")
            .fetch_one(&self.pool)
            .await?;
        let metadata_count: i64 = metadata_count_row.try_get("count")?;
        
        let thumbnail_count_row = sqlx::query("SELECT COUNT(*) as count FROM thumbnail_cache")
            .fetch_one(&self.pool)
            .await?;
        let thumbnail_count: i64 = thumbnail_count_row.try_get("count")?;
        
        Ok(CacheStats {
            metadata_entries: metadata_count as usize,
            thumbnail_entries: thumbnail_count as usize,
            database_path: self.config.database_path.clone(),
        })
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub metadata_entries: usize,
    pub thumbnail_entries: usize,
    pub database_path: PathBuf,
}

/// Cache cleanup results
#[derive(Debug, Clone)]
pub struct CacheCleanupResult {
    pub stale_metadata_removed: usize,
    pub orphaned_thumbnails_removed: usize,
    pub invalid_thumbnails_removed: usize,
    pub total_entries_removed: usize,
}

/// Metadata cache statistics
#[derive(Debug, Clone)]
pub struct MetadataCacheStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub average_file_size: u64,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
}

/// Thumbnail cache statistics
#[derive(Debug, Clone)]
pub struct ThumbnailCacheStats {
    pub total_entries: usize,
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub size_bytes: u64,
    pub database_path: PathBuf,
}

/// Comprehensive cache metrics
#[derive(Debug, Clone)]
pub struct CacheMetrics {
    pub metadata_cache: MetadataCacheStats,
    pub thumbnail_cache: ThumbnailCacheStats,
    pub database: DatabaseStats,
    pub last_updated: DateTime<Utc>,
}

/// Configuration for cache maintenance operations
#[derive(Debug, Clone)]
pub struct CacheMaintenanceConfig {
    /// Maximum age of cache entries in seconds before they're considered stale
    pub max_age_seconds: i64,
    /// Maximum number of cache entries (for LRU eviction)
    pub max_entries: Option<usize>,
    /// Whether to run database vacuum operation
    pub vacuum_database: bool,
}

impl Default for CacheMaintenanceConfig {
    fn default() -> Self {
        Self {
            max_age_seconds: 7 * 24 * 3600, // 7 days
            max_entries: Some(10000), // 10k entries
            vacuum_database: true,
        }
    }
}

/// Results of cache maintenance operation
#[derive(Debug, Clone)]
pub struct CacheMaintenanceResult {
    pub cleanup_result: CacheCleanupResult,
    pub evicted_entries: usize,
    pub database_vacuumed: bool,
    pub final_metrics: CacheMetrics,
    pub maintenance_duration: std::time::Duration,
}

/// Convert SystemTime to Unix timestamp for database storage
fn system_time_to_timestamp(time: SystemTime) -> i64 {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Convert Unix timestamp to SystemTime
fn timestamp_to_system_time(timestamp: i64) -> SystemTime {
    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    async fn create_test_cache() -> (CacheService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_cache.db");
        
        // Ensure the directory exists before creating database
        std::fs::create_dir_all(temp_dir.path()).unwrap_or_default();
        
        
        let cache = CacheService::new(&db_path).await.unwrap();
        (cache, temp_dir)
    }
    
    #[tokio::test]
    async fn test_cache_service_creation() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        // Test that we can get cache stats
        let stats = cache.get_cache_stats().await.unwrap();
        assert_eq!(stats.metadata_entries, 0);
        assert_eq!(stats.thumbnail_entries, 0);
        
        // Test schema version
        let version = cache.get_schema_version().await.unwrap();
        assert_eq!(version, 1);
    }
    
    #[tokio::test]
    async fn test_schema_creation() {
        let (_cache, _temp_dir) = create_test_cache().await;
        
        // If we get here without panicking, schema creation worked
        // More detailed tests will be added in subsequent subtasks
    }
    
    #[tokio::test]
    async fn test_cache_config() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("config_test.db");
        
        // Test custom configuration
        let config = CacheConfig::new(&db_path)
            .max_connections(5)
            .min_connections(1)
            .idle_timeout(Duration::from_secs(300))
            .wal_mode(true)
            .foreign_keys(true);
        
        let cache = CacheService::new_with_config(config.clone()).await.unwrap();
        
        // Verify configuration is stored correctly
        assert_eq!(cache.config().max_connections, 5);
        assert_eq!(cache.config().min_connections, 1);
        assert_eq!(cache.config().idle_timeout, Duration::from_secs(300));
        assert_eq!(cache.config().enable_wal_mode, true);
        assert_eq!(cache.config().enable_foreign_keys, true);
        assert_eq!(cache.config().database_path, db_path);
        
        // Test that cache stats work with config
        let stats = cache.get_cache_stats().await.unwrap();
        assert_eq!(stats.database_path, db_path);
    }
    
    #[tokio::test]
    async fn test_cache_config_validation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("validation_test.db");
        
        // Test invalid max_connections = 0
        let config = CacheConfig::new(&db_path).max_connections(0);
        let result = CacheService::new_with_config(config).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CacheError::Configuration(_)));
        
        // Test min_connections > max_connections
        let config = CacheConfig::new(&db_path)
            .max_connections(2)
            .min_connections(5);
        let result = CacheService::new_with_config(config).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CacheError::Configuration(_)));
    }
    
    #[tokio::test]
    async fn test_metadata_store_and_retrieve() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        // Create test metadata
        let test_path = PathBuf::from("/test/file.txt");
        let metadata = CachedFileMetadata::new(
            test_path.clone(),
            1024,
            SystemTime::now(),
            Some("abc123".to_string())
        );
        
        // Store metadata
        cache.store_metadata(&metadata).await.unwrap();
        
        // Retrieve metadata
        let retrieved = cache.get_metadata(&test_path).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.path, test_path);
        assert_eq!(retrieved.size, 1024);
        assert_eq!(retrieved.hash, Some("abc123".to_string()));
    }
    
    #[tokio::test]
    async fn test_metadata_exists() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let test_path = PathBuf::from("/test/exists.txt");
        
        // Should not exist initially
        assert!(!cache.metadata_exists(&test_path).await.unwrap());
        
        // Store metadata
        let metadata = CachedFileMetadata::new(
            test_path.clone(),
            512,
            SystemTime::now(),
            None
        );
        cache.store_metadata(&metadata).await.unwrap();
        
        // Should exist now
        assert!(cache.metadata_exists(&test_path).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_metadata_freshness() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let test_path = PathBuf::from("/test/fresh.txt");
        let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        let new_time = SystemTime::now();
        
        // Store metadata with old modification time
        let metadata = CachedFileMetadata::new(
            test_path.clone(),
            256,
            old_time,
            None
        );
        cache.store_metadata(&metadata).await.unwrap();
        
        // Check freshness - should be stale compared to new time
        assert!(!cache.is_metadata_fresh(&test_path, new_time).await.unwrap());
        
        // Check freshness - should be fresh compared to old time
        assert!(cache.is_metadata_fresh(&test_path, old_time).await.unwrap());
        
        // Non-existent file should not be fresh
        let nonexistent_path = PathBuf::from("/test/nonexistent.txt");
        assert!(!cache.is_metadata_fresh(&nonexistent_path, new_time).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_batch_store_metadata() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        // Create multiple metadata entries
        let metadata_list = vec![
            CachedFileMetadata::new(
                PathBuf::from("/test/batch1.txt"),
                100,
                SystemTime::now(),
                Some("hash1".to_string())
            ),
            CachedFileMetadata::new(
                PathBuf::from("/test/batch2.txt"),
                200,
                SystemTime::now(),
                Some("hash2".to_string())
            ),
            CachedFileMetadata::new(
                PathBuf::from("/test/batch3.txt"),
                300,
                SystemTime::now(),
                None
            ),
        ];
        
        // Batch store
        cache.batch_store_metadata(&metadata_list).await.unwrap();
        
        // Verify all entries were stored
        for metadata in &metadata_list {
            let retrieved = cache.get_metadata(&metadata.path).await.unwrap();
            assert!(retrieved.is_some());
            let retrieved = retrieved.unwrap();
            assert_eq!(retrieved.path, metadata.path);
            assert_eq!(retrieved.size, metadata.size);
            assert_eq!(retrieved.hash, metadata.hash);
        }
        
        // Test empty batch (should not error)
        cache.batch_store_metadata(&[]).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_remove_metadata() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        let test_path = PathBuf::from("/test/remove.txt");
        let metadata = CachedFileMetadata::new(
            test_path.clone(),
            128,
            SystemTime::now(),
            None
        );
        
        // Store metadata
        cache.store_metadata(&metadata).await.unwrap();
        assert!(cache.metadata_exists(&test_path).await.unwrap());
        
        // Remove metadata
        let removed = cache.remove_metadata(&test_path).await.unwrap();
        assert!(removed);
        assert!(!cache.metadata_exists(&test_path).await.unwrap());
        
        // Try to remove again (should return false)
        let removed = cache.remove_metadata(&test_path).await.unwrap();
        assert!(!removed);
    }
    
    #[tokio::test]
    async fn test_metadata_prefix_operations() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        // Create metadata entries in different directories
        let entries = vec![
            ("/test/dir1/file1.txt", 100),
            ("/test/dir1/file2.txt", 200),
            ("/test/dir1/subdir/file3.txt", 300),
            ("/test/dir2/file4.txt", 400),
            ("/other/file5.txt", 500),
        ];
        
        for (path_str, size) in &entries {
            let metadata = CachedFileMetadata::new(
                PathBuf::from(path_str),
                *size,
                SystemTime::now(),
                None
            );
            cache.store_metadata(&metadata).await.unwrap();
        }
        
        // Get metadata by prefix
        let prefix = PathBuf::from("/test/dir1");
        let results = cache.get_metadata_by_prefix(&prefix).await.unwrap();
        
        // Should find 3 entries under /test/dir1
        assert_eq!(results.len(), 3);
        
        // Verify the paths start with the prefix
        for result in &results {
            assert!(result.path.to_string_lossy().starts_with("/test/dir1"));
        }
        
        // Remove metadata by prefix
        let removed_count = cache.remove_metadata_by_prefix(&prefix).await.unwrap();
        assert_eq!(removed_count, 3);
        
        // Verify they're gone
        let results = cache.get_metadata_by_prefix(&prefix).await.unwrap();
        assert_eq!(results.len(), 0);
        
        // Other entries should still exist
        assert!(cache.metadata_exists(&PathBuf::from("/test/dir2/file4.txt")).await.unwrap());
        assert!(cache.metadata_exists(&PathBuf::from("/other/file5.txt")).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_cached_file_metadata_methods() {
        let test_path = PathBuf::from("/test/methods.txt");
        let mut metadata = CachedFileMetadata::new(
            test_path.clone(),
            1024,
            SystemTime::now(),
            Some("original_hash".to_string())
        );
        
        let original_updated_at = metadata.updated_at;
        
        // Wait a small amount to ensure timestamp difference
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Update metadata
        metadata.update(2048, SystemTime::now(), Some("new_hash".to_string()));
        
        // Verify updates
        assert_eq!(metadata.size, 2048);
        assert_eq!(metadata.hash, Some("new_hash".to_string()));
        assert!(metadata.updated_at > original_updated_at);
    }
    
    #[tokio::test]
    async fn test_thumbnail_store_and_retrieve() {
        let (cache, temp_dir) = create_test_cache().await;
        
        // Create test thumbnail entry
        let file_path = PathBuf::from("/test/image.jpg");
        let thumbnail_path = temp_dir.path().join("thumb.jpg");
        
        // Create the thumbnail file
        std::fs::write(&thumbnail_path, b"fake thumbnail data").unwrap();
        
        let thumbnail = CachedThumbnail::new(file_path.clone(), thumbnail_path.clone());
        
        // Store thumbnail
        cache.store_thumbnail_path(&thumbnail).await.unwrap();
        
        // Retrieve thumbnail
        let retrieved = cache.get_thumbnail_path(&file_path).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.file_path, file_path);
        assert_eq!(retrieved.thumbnail_path, thumbnail_path);
    }
    
    #[tokio::test]
    async fn test_thumbnail_exists() {
        let (cache, temp_dir) = create_test_cache().await;
        
        let file_path = PathBuf::from("/test/exists.jpg");
        let thumbnail_path = temp_dir.path().join("exists_thumb.jpg");
        
        // Should not exist initially
        assert!(!cache.thumbnail_exists(&file_path).await.unwrap());
        
        // Create and store thumbnail
        std::fs::write(&thumbnail_path, b"thumbnail").unwrap();
        let thumbnail = CachedThumbnail::new(file_path.clone(), thumbnail_path);
        cache.store_thumbnail_path(&thumbnail).await.unwrap();
        
        // Should exist now
        assert!(cache.thumbnail_exists(&file_path).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_thumbnail_validation() {
        let (cache, temp_dir) = create_test_cache().await;
        
        let file_path = PathBuf::from("/test/validation.png");
        let thumbnail_path = temp_dir.path().join("validation_thumb.png");
        
        // Create thumbnail file
        std::fs::write(&thumbnail_path, b"thumbnail data").unwrap();
        let thumbnail = CachedThumbnail::new(file_path.clone(), thumbnail_path.clone());
        cache.store_thumbnail_path(&thumbnail).await.unwrap();
        
        // Should be valid initially
        assert!(cache.is_thumbnail_valid(&file_path).await.unwrap());
        
        // Remove the thumbnail file
        std::fs::remove_file(&thumbnail_path).unwrap();
        
        // Should now be invalid
        assert!(!cache.is_thumbnail_valid(&file_path).await.unwrap());
        
        // Non-existent entry should be invalid
        let nonexistent_path = PathBuf::from("/test/nonexistent.jpg");
        assert!(!cache.is_thumbnail_valid(&nonexistent_path).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_batch_store_thumbnails() {
        let (cache, temp_dir) = create_test_cache().await;
        
        // Create multiple thumbnail entries
        let thumbnails = vec![
            {
                let thumb_path = temp_dir.path().join("batch1_thumb.jpg");
                std::fs::write(&thumb_path, b"thumbnail1").unwrap();
                CachedThumbnail::new(
                    PathBuf::from("/test/batch1.jpg"),
                    thumb_path
                )
            },
            {
                let thumb_path = temp_dir.path().join("batch2_thumb.jpg");
                std::fs::write(&thumb_path, b"thumbnail2").unwrap();
                CachedThumbnail::new(
                    PathBuf::from("/test/batch2.jpg"),
                    thumb_path
                )
            },
            {
                let thumb_path = temp_dir.path().join("batch3_thumb.jpg");
                std::fs::write(&thumb_path, b"thumbnail3").unwrap();
                CachedThumbnail::new(
                    PathBuf::from("/test/batch3.jpg"),
                    thumb_path
                )
            },
        ];
        
        // Batch store
        cache.batch_store_thumbnail_paths(&thumbnails).await.unwrap();
        
        // Verify all entries were stored
        for thumbnail in &thumbnails {
            let retrieved = cache.get_thumbnail_path(&thumbnail.file_path).await.unwrap();
            assert!(retrieved.is_some());
            let retrieved = retrieved.unwrap();
            assert_eq!(retrieved.file_path, thumbnail.file_path);
            assert_eq!(retrieved.thumbnail_path, thumbnail.thumbnail_path);
        }
        
        // Test empty batch (should not error)
        cache.batch_store_thumbnail_paths(&[]).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_remove_thumbnail() {
        let (cache, temp_dir) = create_test_cache().await;
        
        let file_path = PathBuf::from("/test/remove.jpg");
        let thumbnail_path = temp_dir.path().join("remove_thumb.jpg");
        
        // Create and store thumbnail
        std::fs::write(&thumbnail_path, b"thumbnail").unwrap();
        let thumbnail = CachedThumbnail::new(file_path.clone(), thumbnail_path);
        cache.store_thumbnail_path(&thumbnail).await.unwrap();
        assert!(cache.thumbnail_exists(&file_path).await.unwrap());
        
        // Remove thumbnail
        let removed = cache.remove_thumbnail(&file_path).await.unwrap();
        assert!(removed);
        assert!(!cache.thumbnail_exists(&file_path).await.unwrap());
        
        // Try to remove again (should return false)
        let removed = cache.remove_thumbnail(&file_path).await.unwrap();
        assert!(!removed);
    }
    
    #[tokio::test]
    async fn test_thumbnail_prefix_operations() {
        let (cache, temp_dir) = create_test_cache().await;
        
        // Create thumbnail entries in different directories
        let entries = vec![
            ("/test/dir1/image1.jpg", "dir1_image1_thumb.jpg"),
            ("/test/dir1/image2.jpg", "dir1_image2_thumb.jpg"),
            ("/test/dir1/subdir/image3.jpg", "dir1_subdir_image3_thumb.jpg"),
            ("/test/dir2/image4.jpg", "dir2_image4_thumb.jpg"),
            ("/other/image5.jpg", "other_image5_thumb.jpg"),
        ];
        
        for (file_path_str, thumb_name) in &entries {
            let thumb_path = temp_dir.path().join(thumb_name);
            std::fs::write(&thumb_path, b"thumbnail data").unwrap();
            
            let thumbnail = CachedThumbnail::new(
                PathBuf::from(file_path_str),
                thumb_path
            );
            cache.store_thumbnail_path(&thumbnail).await.unwrap();
        }
        
        // Get thumbnails by prefix
        let prefix = PathBuf::from("/test/dir1");
        let results = cache.get_thumbnails_by_prefix(&prefix).await.unwrap();
        
        // Should find 3 entries under /test/dir1
        assert_eq!(results.len(), 3);
        
        // Verify the file paths start with the prefix
        for result in &results {
            assert!(result.file_path.to_string_lossy().starts_with("/test/dir1"));
        }
        
        // Remove thumbnails by prefix
        let removed_count = cache.remove_thumbnails_by_prefix(&prefix).await.unwrap();
        assert_eq!(removed_count, 3);
        
        // Verify they're gone
        let results = cache.get_thumbnails_by_prefix(&prefix).await.unwrap();
        assert_eq!(results.len(), 0);
        
        // Other entries should still exist
        assert!(cache.thumbnail_exists(&PathBuf::from("/test/dir2/image4.jpg")).await.unwrap());
        assert!(cache.thumbnail_exists(&PathBuf::from("/other/image5.jpg")).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_cleanup_invalid_thumbnails() {
        let (cache, temp_dir) = create_test_cache().await;
        
        // Create thumbnails - some valid, some that will become invalid
        let valid_thumb_path = temp_dir.path().join("valid_thumb.jpg");
        let invalid_thumb_path = temp_dir.path().join("invalid_thumb.jpg");
        
        // Create both thumbnail files initially
        std::fs::write(&valid_thumb_path, b"valid thumbnail").unwrap();
        std::fs::write(&invalid_thumb_path, b"invalid thumbnail").unwrap();
        
        let valid_thumbnail = CachedThumbnail::new(
            PathBuf::from("/test/valid.jpg"),
            valid_thumb_path
        );
        let invalid_thumbnail = CachedThumbnail::new(
            PathBuf::from("/test/invalid.jpg"),
            invalid_thumb_path.clone()
        );
        
        // Store both thumbnails
        cache.store_thumbnail_path(&valid_thumbnail).await.unwrap();
        cache.store_thumbnail_path(&invalid_thumbnail).await.unwrap();
        
        // Verify both exist in cache
        assert!(cache.thumbnail_exists(&PathBuf::from("/test/valid.jpg")).await.unwrap());
        assert!(cache.thumbnail_exists(&PathBuf::from("/test/invalid.jpg")).await.unwrap());
        
        // Remove the invalid thumbnail file from filesystem
        std::fs::remove_file(&invalid_thumb_path).unwrap();
        
        // Run cleanup
        let cleaned_count = cache.cleanup_invalid_thumbnails().await.unwrap();
        assert_eq!(cleaned_count, 1);
        
        // Verify only the valid thumbnail remains
        assert!(cache.thumbnail_exists(&PathBuf::from("/test/valid.jpg")).await.unwrap());
        assert!(!cache.thumbnail_exists(&PathBuf::from("/test/invalid.jpg")).await.unwrap());
        
        // Run cleanup again (should find nothing to clean)
        let cleaned_count = cache.cleanup_invalid_thumbnails().await.unwrap();
        assert_eq!(cleaned_count, 0);
    }
    
    #[tokio::test]
    async fn test_cached_thumbnail_methods() {
        let file_path = PathBuf::from("/test/thumbnail_methods.jpg");
        let thumbnail_path = PathBuf::from("/test/thumbnails/thumb.jpg");
        
        let thumbnail = CachedThumbnail::new(file_path.clone(), thumbnail_path.clone());
        
        // Verify construction
        assert_eq!(thumbnail.file_path, file_path);
        assert_eq!(thumbnail.thumbnail_path, thumbnail_path);
        assert!(!thumbnail.id.is_empty());
        
        // Verify created_at is recent
        let now = Utc::now();
        let time_diff = now.signed_duration_since(thumbnail.created_at);
        assert!(time_diff.num_seconds() < 1); // Should be very recent
    }
    
    #[tokio::test]
    async fn test_remove_stale_entries() {
        let (cache, temp_dir) = create_test_cache().await;
        
        // Create some metadata entries with different ages
        let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1000); // Very old
        let new_time = SystemTime::now();
        
        let old_metadata = CachedFileMetadata::new(
            PathBuf::from("/test/old_file.txt"),
            100,
            old_time,
            None
        );
        
        let new_metadata = CachedFileMetadata::new(
            PathBuf::from("/test/new_file.txt"),
            200,
            new_time,
            None
        );
        
        // Store both entries
        cache.store_metadata(&old_metadata).await.unwrap();
        cache.store_metadata(&new_metadata).await.unwrap();
        
        // Create thumbnail for old file and verify it gets cleaned up
        let old_thumb_path = temp_dir.path().join("old_thumb.jpg");
        std::fs::write(&old_thumb_path, b"old thumbnail").unwrap();
        let old_thumbnail = CachedThumbnail::new(
            PathBuf::from("/test/old_file.txt"),
            old_thumb_path
        );
        cache.store_thumbnail_path(&old_thumbnail).await.unwrap();
        
        // Remove entries older than 30 days (should remove the old one)
        let result = cache.remove_stale_entries(30 * 24 * 3600).await.unwrap();
        
        // Should have removed the old metadata entry
        assert_eq!(result.stale_metadata_removed, 1);
        
        // Verify old entry is gone and new entry remains
        assert!(!cache.metadata_exists(&PathBuf::from("/test/old_file.txt")).await.unwrap());
        assert!(cache.metadata_exists(&PathBuf::from("/test/new_file.txt")).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_cleanup_orphaned_thumbnails() {
        let (cache, temp_dir) = create_test_cache().await;
        
        // Create metadata entry
        let metadata = CachedFileMetadata::new(
            PathBuf::from("/test/with_metadata.jpg"),
            1024,
            SystemTime::now(),
            None
        );
        cache.store_metadata(&metadata).await.unwrap();
        
        // Create thumbnail with corresponding metadata
        let valid_thumb_path = temp_dir.path().join("valid_thumb.jpg");
        std::fs::write(&valid_thumb_path, b"valid thumbnail").unwrap();
        let valid_thumbnail = CachedThumbnail::new(
            PathBuf::from("/test/with_metadata.jpg"),
            valid_thumb_path
        );
        cache.store_thumbnail_path(&valid_thumbnail).await.unwrap();
        
        // Create orphaned thumbnail (no corresponding metadata)
        let orphaned_thumb_path = temp_dir.path().join("orphaned_thumb.jpg");
        std::fs::write(&orphaned_thumb_path, b"orphaned thumbnail").unwrap();
        let orphaned_thumbnail = CachedThumbnail::new(
            PathBuf::from("/test/orphaned.jpg"),
            orphaned_thumb_path
        );
        cache.store_thumbnail_path(&orphaned_thumbnail).await.unwrap();
        
        // Verify both thumbnails exist
        assert!(cache.thumbnail_exists(&PathBuf::from("/test/with_metadata.jpg")).await.unwrap());
        assert!(cache.thumbnail_exists(&PathBuf::from("/test/orphaned.jpg")).await.unwrap());
        
        // Clean up orphaned thumbnails
        let removed_count = cache.cleanup_orphaned_thumbnails().await.unwrap();
        assert_eq!(removed_count, 1);
        
        // Verify only valid thumbnail remains
        assert!(cache.thumbnail_exists(&PathBuf::from("/test/with_metadata.jpg")).await.unwrap());
        assert!(!cache.thumbnail_exists(&PathBuf::from("/test/orphaned.jpg")).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_vacuum_database() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        // Add some data to the database
        let metadata = CachedFileMetadata::new(
            PathBuf::from("/test/vacuum_test.txt"),
            1024,
            SystemTime::now(),
            None
        );
        cache.store_metadata(&metadata).await.unwrap();
        
        // Vacuum should complete without error
        cache.vacuum_database().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_cache_metrics() {
        let (cache, temp_dir) = create_test_cache().await;
        
        // Add some test data
        let metadata = CachedFileMetadata::new(
            PathBuf::from("/test/metrics_test.txt"),
            2048,
            SystemTime::now(),
            Some("hash123".to_string())
        );
        cache.store_metadata(&metadata).await.unwrap();
        
        let thumb_path = temp_dir.path().join("metrics_thumb.jpg");
        std::fs::write(&thumb_path, b"thumbnail data").unwrap();
        let thumbnail = CachedThumbnail::new(
            PathBuf::from("/test/metrics_test.txt"),
            thumb_path
        );
        cache.store_thumbnail_path(&thumbnail).await.unwrap();
        
        // Get metrics
        let metrics = cache.get_cache_metrics().await.unwrap();
        
        // Verify metrics
        assert_eq!(metrics.metadata_cache.total_entries, 1);
        assert_eq!(metrics.metadata_cache.total_size_bytes, 2048);
        assert_eq!(metrics.thumbnail_cache.total_entries, 1);
        assert!(metrics.database.size_bytes > 0);
        assert!(metrics.last_updated <= Utc::now());
    }
    
    #[tokio::test]
    async fn test_lru_eviction() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        // Create 5 metadata entries
        for i in 0..5 {
            let metadata = CachedFileMetadata::new(
                PathBuf::from(format!("/test/lru_test_{}.txt", i)),
                1024,
                SystemTime::now(),
                None
            );
            cache.store_metadata(&metadata).await.unwrap();
            
            // Small delay to ensure different created_at timestamps
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        // Verify all 5 entries exist
        let initial_metrics = cache.get_cache_metrics().await.unwrap();
        assert_eq!(initial_metrics.metadata_cache.total_entries, 5);
        
        // Enforce limit of 3 entries (should remove 2 oldest)
        let evicted_count = cache.enforce_cache_size_limit(3).await.unwrap();
        assert_eq!(evicted_count, 2);
        
        // Verify only 3 entries remain
        let final_metrics = cache.get_cache_metrics().await.unwrap();
        assert_eq!(final_metrics.metadata_cache.total_entries, 3);
        
        // Verify oldest entries were removed
        assert!(!cache.metadata_exists(&PathBuf::from("/test/lru_test_0.txt")).await.unwrap());
        assert!(!cache.metadata_exists(&PathBuf::from("/test/lru_test_1.txt")).await.unwrap());
        
        // Verify newest entries remain
        assert!(cache.metadata_exists(&PathBuf::from("/test/lru_test_2.txt")).await.unwrap());
        assert!(cache.metadata_exists(&PathBuf::from("/test/lru_test_3.txt")).await.unwrap());
        assert!(cache.metadata_exists(&PathBuf::from("/test/lru_test_4.txt")).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_comprehensive_maintenance() {
        let (cache, temp_dir) = create_test_cache().await;
        
        // Create mix of old and new entries
        let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        let new_time = SystemTime::now();
        
        // Old metadata (will be stale)
        let old_metadata = CachedFileMetadata::new(
            PathBuf::from("/test/old_maintenance.txt"),
            1024,
            old_time,
            None
        );
        cache.store_metadata(&old_metadata).await.unwrap();
        
        // New metadata entries (for LRU testing)
        for i in 0..5 {
            let metadata = CachedFileMetadata::new(
                PathBuf::from(format!("/test/new_maintenance_{}.txt", i)),
                1024,
                new_time,
                None
            );
            cache.store_metadata(&metadata).await.unwrap();
        }
        
        // Create orphaned thumbnail
        let orphaned_thumb_path = temp_dir.path().join("orphaned_maintenance_thumb.jpg");
        std::fs::write(&orphaned_thumb_path, b"orphaned").unwrap();
        let orphaned_thumbnail = CachedThumbnail::new(
            PathBuf::from("/test/orphaned_maintenance.jpg"),
            orphaned_thumb_path
        );
        cache.store_thumbnail_path(&orphaned_thumbnail).await.unwrap();
        
        // Configure maintenance to remove stale entries and limit to 3 entries
        let config = CacheMaintenanceConfig {
            max_age_seconds: 30 * 24 * 3600, // 30 days
            max_entries: Some(3),
            vacuum_database: true,
        };
        
        // Run comprehensive maintenance
        let result = cache.perform_maintenance(&config).await.unwrap();
        
        // Verify results
        assert_eq!(result.cleanup_result.stale_metadata_removed, 1); // old entry
        assert_eq!(result.cleanup_result.orphaned_thumbnails_removed, 1); // orphaned thumbnail
        assert_eq!(result.evicted_entries, 2); // LRU eviction to get to 3 entries
        assert!(result.database_vacuumed);
        assert_eq!(result.final_metrics.metadata_cache.total_entries, 3);
        assert!(result.maintenance_duration.as_millis() > 0);
    }
    
    #[tokio::test]
    async fn test_cache_maintenance_config() {
        let default_config = CacheMaintenanceConfig::default();
        
        assert_eq!(default_config.max_age_seconds, 7 * 24 * 3600); // 7 days
        assert_eq!(default_config.max_entries, Some(10000));
        assert!(default_config.vacuum_database);
    }
    
    #[tokio::test]
    async fn test_scheduled_maintenance() {
        let (cache, _temp_dir) = create_test_cache().await;
        
        // Start scheduled maintenance with very short interval for testing
        let config = CacheMaintenanceConfig {
            max_age_seconds: 1, // 1 second (very short for testing)
            max_entries: Some(1000),
            vacuum_database: false, // Skip vacuum for speed
        };
        
        let handle = cache.start_scheduled_maintenance(1, config); // Every 1 second
        
        // Let it run for a short time
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Stop the scheduled maintenance
        CacheService::stop_scheduled_maintenance(handle);
        
        // Test completed successfully if no panics occurred
    }
}