use rusqlite::Connection;
use std::path::Path;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS commits (
    hash         TEXT PRIMARY KEY,
    author_name  TEXT NOT NULL,
    author_email TEXT,
    timestamp    INTEGER NOT NULL,
    summary      TEXT,
    parent_count INTEGER NOT NULL,
    is_merge     INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS file_changes (
    id          INTEGER PRIMARY KEY,
    commit_hash TEXT NOT NULL REFERENCES commits(hash) ON DELETE CASCADE,
    path        TEXT NOT NULL,
    old_path    TEXT,
    status      TEXT NOT NULL,
    insertions  INTEGER NOT NULL DEFAULT 0,
    deletions   INTEGER NOT NULL DEFAULT 0
);
-- speeds up GROUP BY path queries used by `lore hot`
CREATE INDEX IF NOT EXISTS idx_file_changes_path ON file_changes(path);
CREATE INDEX IF NOT EXISTS idx_file_changes_commit_hash ON file_changes(commit_hash);
";

#[derive(Debug)]
pub struct Commit {
    pub hash: String,
    pub author_name: String,
    pub author_email: Option<String>,
    pub timestamp: i64,
    pub summary: Option<String>,
    pub parent_count: i64,
    pub is_merge: bool,
}

impl Commit {
    pub fn new(hash: &str, author_name: &str, timestamp: i64) -> Self {
        Commit {
            hash: hash.to_string(),
            author_name: author_name.to_string(),
            author_email: None,
            timestamp,
            summary: None,
            parent_count: 0,
            is_merge: false,
        }
    }
}

#[derive(Debug)]
pub struct FileChange {
    pub commit_hash: String,
    pub path: String,
    pub old_path: Option<String>,
    pub status: String,
    pub insertions: i64,
    pub deletions: i64,
}

impl FileChange {
    pub fn new(
        commit_hash: &str,
        path: &str,
        status: &str,
        insertions: i64,
        deletions: i64,
    ) -> Self {
        FileChange {
            commit_hash: commit_hash.to_string(),
            path: path.to_string(),
            old_path: None,
            status: status.to_string(),
            insertions,
            deletions,
        }
    }
}

pub fn open(path: &Path) -> anyhow::Result<Connection> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("database path has no parent: {}", path.display()))?;

    std::fs::create_dir_all(parent)?;
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

    #[test]
    fn test_schema_columns() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open(&db_path).unwrap();

        let commit_columns = table_columns(&conn, "commits");
        assert_eq!(
            commit_columns,
            vec![
                "hash",
                "author_name",
                "author_email",
                "timestamp",
                "summary",
                "parent_count",
                "is_merge"
            ]
        );

        let file_change_columns = table_columns(&conn, "file_changes");
        assert_eq!(
            file_change_columns,
            vec![
                "id",
                "commit_hash",
                "path",
                "old_path",
                "status",
                "insertions",
                "deletions"
            ]
        );
    }

    fn table_columns(conn: &Connection, table_name: &str) -> Vec<String> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({table_name})"))
            .unwrap();

        stmt.query_map([], |row| row.get(1))
            .unwrap()
            .map(|res| res.unwrap())
            .collect()
    }
}
