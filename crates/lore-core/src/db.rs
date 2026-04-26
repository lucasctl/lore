use rusqlite::Connection;
use std::path::Path;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS commits (
    hash      TEXT PRIMARY KEY,
    author    TEXT NOT NULL,
    timestamp INTEGER NOT NULL  -- unix epoch
);

CREATE TABLE IF NOT EXISTS file_changes (
    id          INTEGER PRIMARY KEY,
    commit_hash TEXT NOT NULL REFERENCES commits(hash),
    path        TEXT NOT NULL,
    insertions  INTEGER NOT NULL DEFAULT 0,
    deletions   INTEGER NOT NULL DEFAULT 0
);
-- speeds up GROUP BY path queries used by `lore hot`
CREATE INDEX IF NOT EXISTS idx_file_changes_path ON file_changes(path);
";

#[derive(Debug)]
pub struct Commit {
    pub hash: String,
    pub author: String,
    pub timestamp: i64,
}

impl Commit {
    pub fn new(hash: &str, author: &str, timestamp: i64) -> Self {
        Commit {
            hash: hash.to_string(),
            author: author.to_string(),
            timestamp,
        }
    }
}

#[derive(Debug)]
pub struct FileChange {
    pub commit_hash: String,
    pub path: String,
    pub insertions: i64,
    pub deletions: i64,
}

impl FileChange {
    pub fn new(commit_hash: &str, path: &str, insertions: i64, deletions: i64) -> Self {
        FileChange {
            commit_hash: commit_hash.to_string(),
            path: path.to_string(),
            insertions,
            deletions,
        }
    }
}

pub fn open(path: &Path) -> anyhow::Result<Connection> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    let conn = Connection::open(path)?;
    conn.execute_batch(SCHEMA)?;
    Ok(conn)
}

pub fn reset(path: &Path) -> anyhow::Result<()> {
    if std::fs::metadata(path).is_ok() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_db_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open(&db_path).unwrap();
        // Check that the tables were created
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        let table_names: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|res| res.unwrap())
            .collect();
        assert!(table_names.contains(&"commits".to_string()));
        assert!(table_names.contains(&"file_changes".to_string()));
    }
}
