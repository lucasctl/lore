use anyhow::Context;
use git2::Repository;
use std::path::{Path, PathBuf};

pub struct LorePaths {
    pub repo: Repository,
    pub lore_dir: PathBuf,
    pub db_path: PathBuf,
}

pub fn discover(start: impl AsRef<Path>) -> anyhow::Result<LorePaths> {
    let repo = Repository::discover(start)?;

    if repo.is_bare() {
        anyhow::bail!("Bare repositories are not supported");
    }

    let repo_path = repo
        .path()
        .parent()
        .context("Failed to get repository path")?
        .to_path_buf();

    Ok(LorePaths {
        repo,
        lore_dir: repo_path.join(".lore"),
        db_path: repo_path.join(".lore").join("lore.db"),
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
            paths.repo.path().canonicalize().unwrap(),
            repo_path.join(".git").canonicalize().unwrap()
        );
        assert_eq!(
            paths.lore_dir,
            paths.repo.path().parent().unwrap().join(".lore")
        );
        assert_eq!(
            paths.db_path,
            paths
                .repo
                .path()
                .parent()
                .unwrap()
                .join(".lore")
                .join("lore.db")
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
