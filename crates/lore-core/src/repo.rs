use anyhow::Context;
use git2::Repository;
use std::path::{Path, PathBuf};

pub const LORE_DIR_NAME: &str = ".lore";
pub const DB_FILE_NAME: &str = "db.sqlite";

pub struct RepoPaths {
    pub root: PathBuf,
    pub lore_dir: PathBuf,
    pub db_path: PathBuf,
}

pub fn discover(start: impl AsRef<Path>) -> anyhow::Result<RepoPaths> {
    let start = start.as_ref();
    let repo = Repository::discover(start)
        .with_context(|| format!("Failed to discover git repository from {}", start.display()))?;

    if repo.is_bare() {
        anyhow::bail!("Bare repositories are not supported");
    }

    let root = repo
        .workdir()
        .context("Failed to get repository working directory")?
        .canonicalize()
        .context("Failed to canonicalize repository root")?;
    let lore_dir = root.join(LORE_DIR_NAME);
    let db_path = lore_dir.join(DB_FILE_NAME);

    Ok(RepoPaths {
        root,
        lore_dir,
        db_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_discover() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path().join("repo");
        std::fs::create_dir_all(&repo_path).unwrap();
        Repository::init(&repo_path).unwrap();

        let paths = discover(&repo_path).unwrap();
        assert_eq!(
            paths.root.canonicalize().unwrap(),
            repo_path.canonicalize().unwrap()
        );
        assert_eq!(paths.lore_dir, paths.root.join(LORE_DIR_NAME));
        assert_eq!(
            paths.db_path,
            paths.root.join(LORE_DIR_NAME).join(DB_FILE_NAME)
        );
    }

    #[test]
    fn test_discover_from_nested_directory() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path().join("repo");
        let nested_path = repo_path.join("src").join("bin");
        std::fs::create_dir_all(&nested_path).unwrap();
        Repository::init(&repo_path).unwrap();

        let paths = discover(&nested_path).unwrap();

        assert_eq!(
            paths.root.canonicalize().unwrap(),
            repo_path.canonicalize().unwrap()
        );
        assert_eq!(paths.lore_dir, paths.root.join(LORE_DIR_NAME));
        assert_eq!(
            paths.db_path,
            paths.root.join(LORE_DIR_NAME).join(DB_FILE_NAME)
        );
    }

    #[test]
    fn test_discover_not_a_repo() {
        let dir = tempdir().unwrap();
        let result = discover(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_bare_repo() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path().join("bare_repo");
        Repository::init_bare(&repo_path).unwrap();

        let result = discover(&repo_path);
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("Bare repositories")
        );
    }
}
