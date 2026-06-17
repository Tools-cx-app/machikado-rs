//! File utilities: folder traversal and sorting.

use std::path::Path;
use walkdir::WalkDir;

use crate::sign::FileEntry;

/// Recursively load all files from a folder, sorted by relative path (using `/` separator).
///
/// `ignore_prefixes`: relative path prefixes to skip (e.g. `["subdir_to_skip"]`).
/// `ignore_names`: exact relative paths to skip (e.g. `["machikado", "mazoku"]`).
///
/// Sort order matches ZygiskNext's `TreeSet` behavior (lexicographic string sort).
///
/// # Example
///
/// ```ignore
/// // Build time: skip the signature files themselves
/// let entries = machikado_rs::load_folder_files(module_dir, &[], &["machikado", "mazoku"])?;
/// ```
pub fn load_folder_files(
    folder: &Path,
    ignore_prefixes: &[&str],
    ignore_names: &[&str],
) -> std::io::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();

    for entry in WalkDir::new(folder)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let relative_path = entry
            .path()
            .strip_prefix(folder)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");

        // Check prefix-based ignore
        if ignore_prefixes.iter().any(|p| relative_path.starts_with(p)) {
            continue;
        }
        // Check exact-name ignore
        if ignore_names.iter().any(|n| relative_path == *n) {
            continue;
        }

        let content = std::fs::read(entry.path())?;
        entries.push(FileEntry {
            relative_path,
            content,
        });
    }

    // Sort by `/`-separated path, matching ZygiskNext TreeSet behavior
    entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use std::sync::atomic::{AtomicUsize, Ordering};

    static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir() -> std::path::PathBuf {
        let n = DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("machikado_test_{}_{}", std::process::id(), n));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_file(dir: &Path, rel: &str, content: &[u8]) {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }

    #[test]
    fn test_load_folder_ignore_prefix() {
        let dir = temp_dir();
        let _guard = Cleanup(Some(dir.clone()));

        write_file(&dir, "keep.txt", b"keep");
        write_file(&dir, "skip/config.txt", b"skip");
        write_file(&dir, "skip/nested/data.bin", b"data");

        // Exclude the "skip/" prefix
        let entries = load_folder_files(&dir, &["skip"], &[]).unwrap();
        let paths: Vec<&str> = entries.iter().map(|e| e.relative_path.as_str()).collect();
        assert_eq!(paths, vec!["keep.txt"]);
    }

    #[test]
    fn test_load_folder_ignore_name() {
        let dir = temp_dir();
        let _guard = Cleanup(Some(dir.clone()));

        write_file(&dir, "a.txt", b"a");
        write_file(&dir, "b.txt", b"b");
        write_file(&dir, "sub/c.txt", b"c");

        // Exclude a specific file by exact name
        let entries = load_folder_files(&dir, &[], &["b.txt"]).unwrap();
        let paths: Vec<&str> = entries.iter().map(|e| e.relative_path.as_str()).collect();
        assert_eq!(paths, vec!["a.txt", "sub/c.txt"]);
    }

    #[test]
    fn test_load_folder_ignore_combined() {
        let dir = temp_dir();
        let _guard = Cleanup(Some(dir.clone()));

        write_file(&dir, "keep.txt", b"k");
        write_file(&dir, "skip_prefix/data.txt", b"d");
        write_file(&dir, "skip_exact.txt", b"e");

        // Both prefix and exact-name ignores work together
        let entries = load_folder_files(&dir, &["skip_prefix"], &["skip_exact.txt"]).unwrap();
        let paths: Vec<&str> = entries.iter().map(|e| e.relative_path.as_str()).collect();
        assert_eq!(paths, vec!["keep.txt"]);
    }

    #[test]
    fn test_load_folder_sorts() {
        let dir = temp_dir();
        let _guard = Cleanup(Some(dir.clone()));

        write_file(&dir, "c.txt", b"c");
        write_file(&dir, "a.txt", b"a");
        write_file(&dir, "b/1.txt", b"b1");
        write_file(&dir, "b/0.txt", b"b0");

        let entries = load_folder_files(&dir, &[], &[]).unwrap();
        let paths: Vec<&str> = entries.iter().map(|e| e.relative_path.as_str()).collect();
        assert_eq!(paths, vec!["a.txt", "b/0.txt", "b/1.txt", "c.txt"]);
    }

    /// RAII guard to clean up temp dir on drop
    struct Cleanup(Option<std::path::PathBuf>);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            if let Some(_dir) = self.0.take() {
                // let _ = fs::remove_dir_all(&dir);
            }
        }
    }
}
