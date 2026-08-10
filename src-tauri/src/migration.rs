//! One-time migration of ST-IA's application data from the development
//! bundle identifier to the production one (see ADR-006).
//!
//! M0–M4 shipped under `com.romainbourbon.st-ia.dev`. The release candidate
//! uses `com.romainbourbon.stia`, which moves the Application Support
//! directory macOS resolves for us. Without this migration an existing user
//! would be asked to download the 574 MB model again for no reason.
//!
//! Deliberately narrow: the only artefact ever considered is the single
//! qualified model file, by exact name. No generic sweep of Application
//! Support, no foreign file, no directory the app did not create.

use crate::domain::model;
use crate::model::compute_sha256;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// Bundle identifier used up to and including M4.
pub const LEGACY_BUNDLE_ID: &str = "com.romainbourbon.st-ia.dev";

/// What the migration decided to do, based purely on what exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationAction {
    /// Production model already in place — never touch it, never re-verify
    /// the legacy one. Makes a second launch a no-op.
    AlreadyPresent,
    /// Nothing to migrate from.
    NoLegacy,
    /// Legacy file exists but does not match the manifest: leave it alone
    /// rather than promote a bad file into the production location.
    LegacyInvalid,
    /// Legacy file verified against the manifest and safe to move.
    Migrate,
}

/// Pure decision: what to do given the observable state. Split out from the
/// filesystem work so the policy is unit-testable without 574 MB of fixture.
pub fn decide(production_exists: bool, legacy_exists: bool, legacy_valid: bool) -> MigrationAction {
    if production_exists {
        MigrationAction::AlreadyPresent
    } else if !legacy_exists {
        MigrationAction::NoLegacy
    } else if !legacy_valid {
        MigrationAction::LegacyInvalid
    } else {
        MigrationAction::Migrate
    }
}

fn models_dir_for(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("models")
}

/// Legacy Application Support directory, derived from the production one by
/// swapping the last path component. Never hardcodes a home directory.
fn legacy_app_data_dir(production_app_data_dir: &Path) -> Option<PathBuf> {
    production_app_data_dir
        .parent()
        .map(|container| container.join(LEGACY_BUNDLE_ID))
}

pub fn run(app: &AppHandle) {
    let Ok(prod_dir) = app.path().app_data_dir() else {
        eprintln!("[st-ia] migration: no app data dir, skipping");
        return;
    };
    let Some(legacy_dir) = legacy_app_data_dir(&prod_dir) else {
        return;
    };

    let prod_model = models_dir_for(&prod_dir).join(model::MODEL_FILE_NAME);
    let legacy_model = models_dir_for(&legacy_dir).join(model::MODEL_FILE_NAME);

    let production_exists = prod_model.is_file();
    let legacy_exists = legacy_model.is_file();

    // Only pay for hashing when the answer can still depend on it.
    let legacy_valid = if !production_exists && legacy_exists {
        is_valid_model(&legacy_model)
    } else {
        false
    };

    match decide(production_exists, legacy_exists, legacy_valid) {
        MigrationAction::AlreadyPresent => {
            eprintln!("[st-ia] migration: production model already present, nothing to do");
        }
        MigrationAction::NoLegacy => {
            eprintln!("[st-ia] migration: no legacy model found, nothing to do");
        }
        MigrationAction::LegacyInvalid => {
            eprintln!(
                "[st-ia] migration: legacy model at {} does not match the manifest, left untouched",
                legacy_model.display()
            );
        }
        MigrationAction::Migrate => migrate(&prod_dir, &legacy_dir, &legacy_model, &prod_model),
    }
}

fn is_valid_model(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if metadata.len() != model::MODEL_EXPECTED_SIZE {
        eprintln!(
            "[st-ia] migration: legacy model size {} != expected {}",
            metadata.len(),
            model::MODEL_EXPECTED_SIZE
        );
        return false;
    }
    match compute_sha256(path) {
        Ok(hash) => {
            let valid = model::is_valid(metadata.len(), &hash);
            eprintln!("[st-ia] migration: legacy model hash={hash} valid={valid}");
            valid
        }
        Err(e) => {
            eprintln!("[st-ia] migration: could not hash legacy model: {e}");
            false
        }
    }
}

fn migrate(prod_dir: &Path, legacy_dir: &Path, legacy_model: &Path, prod_model: &Path) {
    if !move_model(&models_dir_for(prod_dir), legacy_model, prod_model) {
        return;
    }

    // Post-move sanity check. Full SHA-256 re-verification is intentionally
    // left to the model manager's own startup detection, which is the single
    // authority on whether a model may be declared `ready`.
    match std::fs::metadata(prod_model).map(|m| m.len()) {
        Ok(size) if size == model::MODEL_EXPECTED_SIZE => {
            eprintln!("[st-ia] migration: destination verified ({size} bytes)");
            prune_empty_legacy_dirs(legacy_dir);
        }
        other => eprintln!("[st-ia] migration: unexpected destination state: {other:?}"),
    }
}

/// Moves the model into the production location. Returns whether the file is
/// now at `prod_model`.
fn move_model(prod_models_dir: &Path, legacy_model: &Path, prod_model: &Path) -> bool {
    if let Err(e) = std::fs::create_dir_all(prod_models_dir) {
        eprintln!("[st-ia] migration: could not create {prod_models_dir:?}: {e}");
        return false;
    }

    // Both locations live under Application Support, i.e. the same volume, so
    // this is an atomic in-place move: the file is either at the old path or
    // the new one, never lost and never half-written. No copy, no 1.1 GB peak,
    // no separate delete step that could run before the destination exists.
    match std::fs::rename(legacy_model, prod_model) {
        Ok(()) => {
            eprintln!(
                "[st-ia] migration: moved model to {} (atomic rename)",
                prod_model.display()
            );
            true
        }
        Err(rename_err) => {
            eprintln!("[st-ia] migration: rename failed ({rename_err}), falling back to copy");
            copy_fallback(prod_models_dir, legacy_model, prod_model)
        }
    }
}

/// Copy-based fallback for the (not expected on macOS) case where the two
/// directories are on different volumes: write to the manager's temp name,
/// verify the copy, then promote atomically, and only then drop the source.
fn copy_fallback(prod_models_dir: &Path, legacy_model: &Path, prod_model: &Path) -> bool {
    let temp = prod_models_dir.join(model::temp_file_name());
    let _ = std::fs::remove_file(&temp);

    if let Err(e) = std::fs::copy(legacy_model, &temp) {
        eprintln!("[st-ia] migration: copy failed: {e}");
        let _ = std::fs::remove_file(&temp);
        return false;
    }
    if !is_valid_model(&temp) {
        eprintln!("[st-ia] migration: copied file failed verification, discarding");
        let _ = std::fs::remove_file(&temp);
        return false;
    }
    if let Err(e) = std::fs::rename(&temp, prod_model) {
        eprintln!("[st-ia] migration: promotion failed: {e}");
        let _ = std::fs::remove_file(&temp);
        return false;
    }
    // Destination is verified and in place; only now is the source redundant.
    let _ = std::fs::remove_file(legacy_model);
    true
}

/// Removes the now-empty legacy directories. `remove_dir` is non-recursive
/// and fails on a non-empty directory, so anything else the user happened to
/// have there is preserved by construction.
fn prune_empty_legacy_dirs(legacy_dir: &Path) {
    let models = models_dir_for(legacy_dir);
    if std::fs::remove_dir(&models).is_ok() {
        eprintln!("[st-ia] migration: removed empty {}", models.display());
        if std::fs::remove_dir(legacy_dir).is_ok() {
            eprintln!("[st-ia] migration: removed empty {}", legacy_dir.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_model_wins_and_short_circuits() {
        // Even with a valid legacy file present, an existing production model
        // is never overwritten — this is what makes relaunching a no-op.
        assert_eq!(decide(true, true, true), MigrationAction::AlreadyPresent);
        assert_eq!(decide(true, false, false), MigrationAction::AlreadyPresent);
        assert_eq!(decide(true, true, false), MigrationAction::AlreadyPresent);
    }

    #[test]
    fn nothing_to_do_without_a_legacy_model() {
        assert_eq!(decide(false, false, false), MigrationAction::NoLegacy);
    }

    #[test]
    fn corrupted_legacy_model_is_never_promoted() {
        assert_eq!(decide(false, true, false), MigrationAction::LegacyInvalid);
    }

    #[test]
    fn valid_legacy_model_is_migrated() {
        assert_eq!(decide(false, true, true), MigrationAction::Migrate);
    }

    #[test]
    fn legacy_dir_is_a_sibling_of_the_production_dir() {
        let prod = Path::new("/Users/x/Library/Application Support/com.romainbourbon.stia");
        assert_eq!(
            legacy_app_data_dir(prod).unwrap(),
            Path::new("/Users/x/Library/Application Support/com.romainbourbon.st-ia.dev")
        );
    }

    #[test]
    fn migration_moves_only_the_model_and_keeps_foreign_files() {
        let root = tempfile::tempdir().unwrap();
        let prod = root.path().join("com.romainbourbon.stia");
        let legacy = root.path().join(LEGACY_BUNDLE_ID);
        let legacy_models = legacy.join("models");
        std::fs::create_dir_all(&legacy_models).unwrap();

        let legacy_model = legacy_models.join(model::MODEL_FILE_NAME);
        std::fs::write(&legacy_model, b"model bytes").unwrap();
        let foreign = legacy_models.join("user-notes.txt");
        std::fs::write(&foreign, b"keep me").unwrap();

        let prod_model = prod.join("models").join(model::MODEL_FILE_NAME);
        assert!(move_model(&prod.join("models"), &legacy_model, &prod_model));
        prune_empty_legacy_dirs(&legacy);

        assert!(prod_model.is_file(), "model should be at the new location");
        assert!(!legacy_model.exists(), "source should have been moved");
        assert!(foreign.is_file(), "unrelated file must survive");
        assert!(
            legacy_models.is_dir(),
            "legacy dir must be kept while it still holds other files"
        );
    }

    #[test]
    fn empty_legacy_dirs_are_pruned_after_migration() {
        let root = tempfile::tempdir().unwrap();
        let prod = root.path().join("com.romainbourbon.stia");
        let legacy = root.path().join(LEGACY_BUNDLE_ID);
        std::fs::create_dir_all(legacy.join("models")).unwrap();
        let legacy_model = legacy.join("models").join(model::MODEL_FILE_NAME);
        std::fs::write(&legacy_model, b"model bytes").unwrap();

        let prod_model = prod.join("models").join(model::MODEL_FILE_NAME);
        assert!(move_model(&prod.join("models"), &legacy_model, &prod_model));
        prune_empty_legacy_dirs(&legacy);

        assert!(prod_model.is_file());
        assert!(!legacy.exists(), "empty legacy tree should be pruned");
    }
}
