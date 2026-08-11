//! Startup recovery: removes artefacts left behind by a previous run that
//! was killed or crashed mid-job.
//!
//! Deliberately narrow. Only two locations are ever touched, both owned by
//! ST-IA itself:
//!
//! * `{temp}/ST-IA/<pid>-<nanos>/` — job workspaces;
//! * `{Application Support}/<bundle-id>/models/*.download` — interrupted
//!   model downloads.
//!
//! The system temp directory at large, the user's files, the user's output
//! folders and a valid model file are never candidates.

use crate::domain::model;
use crate::domain::transcription::is_removable_job_dir_name;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// ST-IA's own subdirectory of the system temp dir.
///
/// Job workspaces live directly inside it — but they are *not* the only thing
/// there. WKWebView namespaces its own scratch space by process name and
/// creates a `WebKit/` directory in this exact path, observed on a packaged
/// release build. That makes the `<pid>-<nanos>` name check in
/// `clean_stale_job_dirs` load-bearing rather than merely defensive: without
/// it, startup cleanup would delete the WebView's working directory.
pub fn temp_root() -> PathBuf {
    std::env::temp_dir().join("ST-IA")
}

pub fn run(app: &AppHandle) {
    let removed_jobs = clean_stale_job_dirs(&temp_root());
    let removed_downloads = clean_stale_downloads(app);
    eprintln!(
        "[st-ia] startup cleanup: {removed_jobs} stale job dir(s), \
         {removed_downloads} interrupted download(s) removed"
    );
}

/// Removes job workspaces inside ST-IA's temp root.
///
/// Guard rails: we only read entries of `root` itself (no recursive walk of
/// the system temp dir), we require the entry to be a directory, and we
/// require its name to match the `<pid>-<nanos>` shape this app generates.
/// Anything else found in that directory is left untouched.
fn clean_stale_job_dirs(root: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(root) else {
        return 0;
    };

    let mut removed = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        // `symlink_metadata` rather than `is_dir()`: `is_dir()` follows the
        // link, so a symlink named like a job directory would be reported as
        // a directory and handed to `remove_dir_all` while pointing
        // somewhere we never meant to touch. We only ever delete a real
        // directory we created, so a symlink is always the wrong thing here
        // regardless of where it points — no need to resolve it to decide.
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            eprintln!(
                "[st-ia] startup cleanup: skipping symlink {}",
                path.display()
            );
            continue;
        }
        if !meta.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !is_removable_job_dir_name(name) {
            eprintln!("[st-ia] startup cleanup: skipping unrecognized entry {name}");
            continue;
        }
        if std::fs::remove_dir_all(&path).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// Strategy A (see ADR-005): an interrupted download is discarded at the
/// next launch rather than resumed — M3 deliberately has no HTTP range
/// resume. Only the exact `<model>.download` temp name is removed; the
/// verified model file itself is never a candidate.
fn clean_stale_downloads(app: &AppHandle) -> usize {
    let Ok(app_data_dir) = app.path().app_data_dir() else {
        eprintln!("[st-ia] startup cleanup: no app data dir, skipping download check");
        return 0;
    };
    let temp_download = app_data_dir.join("models").join(model::temp_file_name());
    eprintln!(
        "[st-ia] startup cleanup: checking {}",
        temp_download.display()
    );
    if !temp_download.is_file() {
        return 0;
    }
    match std::fs::remove_file(&temp_download) {
        Ok(()) => 1,
        Err(e) => {
            eprintln!("[st-ia] startup cleanup: could not remove interrupted download: {e}");
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_job_shaped_dirs_only() {
        let root = tempfile::tempdir().unwrap();
        let job = root.path().join("1234-56789");
        let foreign = root.path().join("not-a-job");
        let nested = job.join("audio.wav");
        std::fs::create_dir_all(&job).unwrap();
        std::fs::create_dir_all(&foreign).unwrap();
        std::fs::write(&nested, b"x").unwrap();

        let removed = clean_stale_job_dirs(root.path());

        assert_eq!(removed, 1);
        assert!(!job.exists(), "job workspace should be removed");
        assert!(foreign.exists(), "unrecognized entry must be left alone");
    }

    #[test]
    fn ignores_files_sitting_in_the_temp_root() {
        let root = tempfile::tempdir().unwrap();
        // Even a job-shaped *file* is not a workspace and must survive.
        let file = root.path().join("1234-56789");
        std::fs::write(&file, b"x").unwrap();

        assert_eq!(clean_stale_job_dirs(root.path()), 0);
        assert!(file.exists());
    }

    #[test]
    fn never_follows_a_job_shaped_symlink() {
        // Adversarial: a symlink whose *name* passes the job-name check but
        // which points outside the temp root. `is_dir()` would have reported
        // it as a directory; the target and the link must both survive.
        let root = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let precious = elsewhere.path().join("precious.txt");
        std::fs::write(&precious, b"user data").unwrap();

        let link = root.path().join("1234-56789");
        std::os::unix::fs::symlink(elsewhere.path(), &link).unwrap();

        let removed = clean_stale_job_dirs(root.path());

        assert_eq!(removed, 0, "a symlink is never a job workspace");
        assert!(precious.is_file(), "symlink target must be untouched");
        assert!(
            std::fs::symlink_metadata(&link).is_ok(),
            "the link itself must be left alone too"
        );
    }

    #[test]
    fn traversal_shaped_names_are_not_job_dirs() {
        // Belt and braces: these can only appear if something other than this
        // app created them, and none of them may ever be treated as ours.
        for hostile in [
            "..",
            ".",
            "../..",
            "..-..",
            "1234-56789/../..",
            "-",
            "1234-",
        ] {
            assert!(
                !is_removable_job_dir_name(hostile),
                "{hostile:?} must not be removable"
            );
        }
    }

    #[test]
    fn missing_root_is_not_an_error() {
        let root = tempfile::tempdir().unwrap();
        let absent = root.path().join("never-created");
        assert_eq!(clean_stale_job_dirs(&absent), 0);
    }
}
