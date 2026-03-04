-- Tracks which events have been successfully synced to avoid re-downloading.
CREATE TABLE IF NOT EXISTS sync_log (
    event_id TEXT PRIMARY KEY,
    event_name TEXT NOT NULL,
    synced_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    fights_count INTEGER NOT NULL DEFAULT 0
);

-- Index for quick lookups
CREATE INDEX IF NOT EXISTS idx_sync_log_synced_at ON sync_log(synced_at);
