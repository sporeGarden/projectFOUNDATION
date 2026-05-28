// SPDX-License-Identifier: AGPL-3.0-or-later
//! BLAKE3 content addressing — streaming hash for zero-copy large files.

use std::io::Read;
use std::path::Path;

use foundation_core::CoreError;

/// Compute the BLAKE3 hash of a file using streaming reads (zero full-file allocation).
///
/// Uses a 64 KiB buffer for constant-memory hashing regardless of file size.
///
/// # Errors
///
/// Returns [`CoreError::Io`] if the file cannot be opened or read.
pub fn blake3_file(path: &Path) -> Result<String, CoreError> {
    let mut file = std::fs::File::open(path).map_err(|e| CoreError::io(path, e))?;

    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; 65_536];

    loop {
        let bytes_read = file.read(&mut buffer).map_err(|e| CoreError::io(path, e))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Verify a file against an expected BLAKE3 hash.
///
/// # Errors
///
/// Returns [`CoreError::Blake3Mismatch`] if the hash does not match,
/// or [`CoreError::Io`] if the file cannot be read.
pub fn verify_blake3(path: &Path, expected: &str) -> Result<(), CoreError> {
    if expected.is_empty() {
        return Ok(());
    }

    let actual = blake3_file(path)?;
    if actual != expected {
        return Err(CoreError::Blake3Mismatch {
            path: path.to_path_buf(),
            expected: expected.to_owned(),
            actual,
        });
    }
    Ok(())
}

/// Hash arbitrary bytes in memory (for small payloads like JSON-RPC responses).
#[must_use]
pub fn blake3_bytes(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn hash_file_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.dat");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"hello foundation").unwrap();
        drop(f);

        let hash1 = blake3_file(&path).unwrap();
        let hash2 = blake3_file(&path).unwrap();
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn verify_matching_hash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("verify.dat");
        std::fs::write(&path, b"test content").unwrap();

        let hash = blake3_file(&path).unwrap();
        assert!(verify_blake3(&path, &hash).is_ok());
    }

    #[test]
    fn verify_mismatched_hash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.dat");
        std::fs::write(&path, b"actual content").unwrap();

        let result = verify_blake3(
            &path,
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
        assert!(result.is_err());
    }

    #[test]
    fn empty_expected_skips_verification() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("skip.dat");
        std::fs::write(&path, b"anything").unwrap();

        assert!(verify_blake3(&path, "").is_ok());
    }

    #[test]
    fn hash_bytes() {
        let h = blake3_bytes(b"test");
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn blake3_not_sha256_regression() {
        let data = b"foundation lineage correctness";
        let hash = blake3_bytes(data);
        let sha256 = "not_blake3";
        assert_ne!(hash, sha256);
        assert_eq!(
            hash,
            blake3::hash(data).to_hex().to_string(),
            "blake3_bytes must produce genuine BLAKE3 output"
        );
        assert_ne!(hash.len(), 0);
        let known_blake3 = blake3::hash(b"foundation lineage correctness")
            .to_hex()
            .to_string();
        assert_eq!(hash, known_blake3);
    }
}
