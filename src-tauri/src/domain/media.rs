use serde::Serialize;
use std::fs;
use std::path::Path;

const SUPPORTED_EXTENSIONS: &[&str] = &["mp4", "mov", "wav", "mp3", "m4a", "flac"];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Video,
    Audio,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub path: String,
    pub file_name: String,
    pub extension: String,
    pub size_bytes: u64,
    pub kind: MediaKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaErrorCode {
    NotFound,
    Unsupported,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaError {
    pub code: MediaErrorCode,
    pub message: String,
}

impl MediaError {
    fn not_found() -> Self {
        Self {
            code: MediaErrorCode::NotFound,
            message: "Le fichier sélectionné est introuvable.".to_string(),
        }
    }

    fn unsupported() -> Self {
        Self {
            code: MediaErrorCode::Unsupported,
            message: "Ce format de fichier n'est pas pris en charge.".to_string(),
        }
    }

    fn empty() -> Self {
        Self {
            code: MediaErrorCode::Empty,
            message: "Le fichier sélectionné est vide.".to_string(),
        }
    }
}

/// Validates a local media path using filesystem metadata only — never reads
/// file content into memory (media files can be several GB).
pub fn validate_media_path(path_str: &str) -> Result<MediaInfo, MediaError> {
    let path = Path::new(path_str);

    let metadata = fs::metadata(path).map_err(|_| MediaError::not_found())?;

    if !metadata.is_file() {
        return Err(MediaError::not_found());
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .unwrap_or_default();

    if !SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
        return Err(MediaError::unsupported());
    }

    if metadata.len() == 0 {
        return Err(MediaError::empty());
    }

    // Confirm the file is actually readable (permissions) without reading its content.
    fs::File::open(path).map_err(|_| MediaError::not_found())?;

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path_str)
        .to_string();

    let kind = if VIDEO_EXTENSIONS.contains(&extension.as_str()) {
        MediaKind::Video
    } else {
        MediaKind::Audio
    };

    Ok(MediaInfo {
        path: path_str.to_string(),
        file_name,
        extension,
        size_bytes: metadata.len(),
        kind,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    struct TempFile {
        path: std::path::PathBuf,
    }

    impl TempFile {
        fn new(name: &str, contents: &[u8]) -> Self {
            let path = std::env::temp_dir().join(format!(
                "st_ia_test_{}_{}_{name}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            let mut file = fs::File::create(&path).expect("create temp file");
            file.write_all(contents).expect("write temp file");
            Self { path }
        }

        fn path_str(&self) -> String {
            self.path.to_string_lossy().to_string()
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    #[test]
    fn accepts_valid_mp4() {
        let file = TempFile::new("sample.mp4", b"fake mp4 bytes");
        let info = validate_media_path(&file.path_str()).expect("should be valid");
        assert_eq!(info.extension, "mp4");
        assert_eq!(info.kind, MediaKind::Video);
        assert_eq!(info.size_bytes, 14);
    }

    #[test]
    fn accepts_valid_mov_uppercase_extension() {
        let file = TempFile::new("sample.MOV", b"fake mov bytes");
        let info = validate_media_path(&file.path_str()).expect("should be valid");
        assert_eq!(info.extension, "mov");
        assert_eq!(info.kind, MediaKind::Video);
    }

    #[test]
    fn accepts_valid_wav() {
        let file = TempFile::new("sample.wav", b"fake wav bytes");
        let info = validate_media_path(&file.path_str()).expect("should be valid");
        assert_eq!(info.extension, "wav");
        assert_eq!(info.kind, MediaKind::Audio);
    }

    #[test]
    fn rejects_unsupported_extension() {
        let file = TempFile::new("sample.txt", b"not a media file");
        let err = validate_media_path(&file.path_str()).expect_err("should be rejected");
        assert_eq!(err.code, MediaErrorCode::Unsupported);
    }

    #[test]
    fn rejects_empty_file() {
        let file = TempFile::new("sample.mp3", b"");
        let err = validate_media_path(&file.path_str()).expect_err("should be rejected");
        assert_eq!(err.code, MediaErrorCode::Empty);
    }

    #[test]
    fn extension_check_is_case_insensitive() {
        let file = TempFile::new("sample.FlAc", b"fake flac bytes");
        let info = validate_media_path(&file.path_str()).expect("should be valid");
        assert_eq!(info.extension, "flac");
        assert_eq!(info.kind, MediaKind::Audio);
    }

    #[test]
    fn rejects_missing_path() {
        let err = validate_media_path("/nonexistent/path/does-not-exist.mp4")
            .expect_err("should be rejected");
        assert_eq!(err.code, MediaErrorCode::NotFound);
    }

    // The tests below are the ones `start_transcription` leans on: this
    // function is the only bound on a path that arrives from the WebView, so
    // it has to hold for values no file picker would ever produce.

    #[test]
    fn rejects_a_directory_even_with_a_media_extension() {
        let dir = std::env::temp_dir().join(format!("st_ia_test_dir_{}.mp4", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let err = validate_media_path(&dir.to_string_lossy()).expect_err("should be rejected");
        assert_eq!(err.code, MediaErrorCode::NotFound);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_a_url_shaped_input() {
        // Not a filesystem path at all. The ffmpeg sidecar is built with
        // only the `file` protocol so it could not fetch this anyway, but
        // the pipeline must refuse it before that ever matters.
        for hostile in [
            "http://example.com/x.mp4",
            "https://example.com/x.mov",
            "concat:/etc/passwd",
            "pipe:0",
        ] {
            let err = validate_media_path(hostile).expect_err("{hostile} should be rejected");
            assert_eq!(err.code, MediaErrorCode::NotFound);
        }
    }

    #[test]
    fn rejects_traversal_that_does_not_resolve_to_a_media_file() {
        for hostile in [
            "../../../../etc/passwd",
            "/etc/passwd",
            "/etc/../etc/passwd",
            "",
        ] {
            assert!(
                validate_media_path(hostile).is_err(),
                "{hostile:?} must be rejected"
            );
        }
    }

    #[test]
    fn rejects_an_extensionless_file() {
        let file = TempFile::new("no-extension", b"bytes");
        let err = validate_media_path(&file.path_str()).expect_err("should be rejected");
        assert_eq!(err.code, MediaErrorCode::Unsupported);
    }

    #[test]
    fn accepts_hostile_but_legitimate_file_names() {
        // A real user's file can contain spaces, unicode, quotes and things
        // that look like shell metacharacters. Sidecars are spawned with an
        // argv vector and no shell, so these are ordinary names — validation
        // must not reject them.
        for name in [
            "espace et accent é à ü.mp4",
            "quote'and\"double.mov",
            "semi;colon && pipe|.wav",
            "$(whoami)-`id`.mp3",
            "emoji-🎬-clip.m4a",
            "-leading-dash.flac",
        ] {
            let file = TempFile::new(name, b"bytes");
            let info = validate_media_path(&file.path_str())
                .unwrap_or_else(|e| panic!("{name:?} should be accepted, got {e:?}"));
            assert!(info.size_bytes > 0);
        }
    }

    #[test]
    fn a_symlink_to_a_non_media_file_is_still_extension_checked() {
        // A symlink is followed by `metadata`, so it can pass the "is a real
        // file" test while pointing anywhere. The extension whitelist is
        // what still applies, and the sidecar would fail to decode it.
        let target = TempFile::new("secret.txt", b"sensitive");
        let link = std::env::temp_dir().join(format!("st_ia_link_{}.txt", std::process::id()));
        let _ = fs::remove_file(&link);
        std::os::unix::fs::symlink(&target.path, &link).unwrap();

        let err = validate_media_path(&link.to_string_lossy()).expect_err("should be rejected");
        assert_eq!(err.code, MediaErrorCode::Unsupported);
        let _ = fs::remove_file(&link);
    }
}
