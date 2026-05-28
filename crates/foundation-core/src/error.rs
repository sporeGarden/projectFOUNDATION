// SPDX-License-Identifier: AGPL-3.0-or-later
//! Error types for foundation-core operations.

use std::path::PathBuf;

/// Errors arising from core type operations (parsing, validation, I/O).
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// TOML manifest failed to parse.
    #[error("failed to parse TOML manifest at {path}: {source}")]
    TomlParse {
        /// Path to the file that failed.
        path: PathBuf,
        /// Underlying parse error.
        source: toml::de::Error,
    },

    /// JSON serialization/deserialization failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// I/O error reading a manifest or data file.
    #[error("I/O error at {path}: {source}")]
    Io {
        /// Path involved in the I/O operation.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// A required field was missing or invalid in a manifest.
    #[error("validation error in {manifest}: {message}")]
    Validation {
        /// Which manifest file.
        manifest: PathBuf,
        /// Description of the issue.
        message: String,
    },

    /// Thread ID not found in the index.
    #[error("thread {id} not found in thread index")]
    ThreadNotFound {
        /// The missing thread identifier.
        id: u32,
    },

    /// BLAKE3 hash mismatch during integrity check.
    #[error("BLAKE3 mismatch for {path}: expected {expected}, got {actual}")]
    Blake3Mismatch {
        /// File that failed the check.
        path: PathBuf,
        /// Expected hex digest.
        expected: String,
        /// Computed hex digest.
        actual: String,
    },
}

impl CoreError {
    /// Wrap an I/O error with a path for context.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn io_wrapper_includes_path() {
        let err = CoreError::io(
            "/data/manifest.toml",
            IoError::new(ErrorKind::NotFound, "missing"),
        );
        let msg = err.to_string();
        assert!(msg.contains("/data/manifest.toml"));
        assert!(msg.contains("I/O error"));
    }

    #[test]
    fn toml_parse_error_display() {
        let parse_err = toml::from_str::<toml::Table>("not = [valid").unwrap_err();
        let err = CoreError::TomlParse {
            path: PathBuf::from("bad.toml"),
            source: parse_err,
        };
        let msg = err.to_string();
        assert!(msg.contains("bad.toml"));
        assert!(msg.contains("failed to parse TOML"));
    }

    #[test]
    fn json_error_from_serde() {
        let json_err: CoreError = serde_json::from_str::<serde_json::Value>("not json")
            .unwrap_err()
            .into();
        assert!(json_err.to_string().starts_with("JSON error:"));
    }

    #[test]
    fn validation_error_display() {
        let err = CoreError::Validation {
            manifest: PathBuf::from("thread01.toml"),
            message: String::from("missing field `id`"),
        };
        let msg = err.to_string();
        assert!(msg.contains("thread01.toml"));
        assert!(msg.contains("missing field"));
    }

    #[test]
    fn thread_not_found_display() {
        let err = CoreError::ThreadNotFound { id: 42 };
        assert_eq!(err.to_string(), "thread 42 not found in thread index");
    }

    #[test]
    fn blake3_mismatch_display() {
        let err = CoreError::Blake3Mismatch {
            path: PathBuf::from("/data/file.dat"),
            expected: String::from("aaa"),
            actual: String::from("bbb"),
        };
        let msg = err.to_string();
        assert!(msg.contains("/data/file.dat"));
        assert!(msg.contains("aaa"));
        assert!(msg.contains("bbb"));
    }
}
