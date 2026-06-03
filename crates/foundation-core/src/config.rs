// SPDX-License-Identifier: AGPL-3.0-or-later
//! Discovery and transport configuration.
//!
//! Models `deploy/discovery_defaults.toml` — the primal discovery chain:
//! 1. Environment (`${PRIMAL}_SOCKET` or `${PRIMAL}_PORT`)
//! 2. XDG discovery socket (`capability.resolve`)
//! 3. Config file (UDS paths then TCP bootstrap)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// Loopback address used as last-resort TCP fallback.
///
/// This is only reached when:
/// 1. No environment variable sets a host
/// 2. No `bootstrap_tcp.host` is configured
///
/// In production (VPS), UDS is preferred and TCP is never reached.
/// On desktop/dev, `discovery_defaults.toml` should set the host explicitly.
pub const LOOPBACK_FALLBACK: &str = "127.0.0.1";

/// Top-level discovery configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscoveryConfig {
    /// Operational metadata.
    #[serde(default)]
    pub metadata: Option<DiscoveryMetadata>,
    /// UDS socket paths per primal.
    #[serde(default)]
    pub sockets: HashMap<String, String>,
    /// TCP bootstrap (dev/desktop only).
    #[serde(default)]
    pub bootstrap_tcp: Option<TcpBootstrap>,
}

/// Metadata about the deployment transport mode.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscoveryMetadata {
    /// VPS standard (e.g. `uds_only`).
    #[serde(default)]
    pub vps_standard: Option<String>,
    /// Preferred transport.
    #[serde(default)]
    pub transport_preference: Option<String>,
}

/// TCP bootstrap configuration for dev/desktop environments.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TcpBootstrap {
    /// Host address for TCP connections.
    #[serde(default = "default_host")]
    pub host: String,
    /// Per-primal port assignments.
    #[serde(flatten)]
    pub ports: HashMap<String, toml::Value>,
}

fn default_host() -> String {
    String::from(LOOPBACK_FALLBACK)
}

/// Resolved transport endpoint for a primal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transport {
    /// Unix domain socket path.
    Uds(PathBuf),
    /// TCP socket address.
    Tcp {
        /// Host address.
        host: String,
        /// Port number.
        port: u16,
    },
}

impl DiscoveryConfig {
    /// Load discovery configuration from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] or [`CoreError::TomlParse`] on failure.
    pub fn from_file(path: &Path) -> Result<Self, CoreError> {
        let content = std::fs::read_to_string(path).map_err(|e| CoreError::io(path, e))?;
        toml::from_str(&content).map_err(|e| CoreError::TomlParse {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Resolve the transport endpoint for a primal using the 3-tier discovery chain.
    ///
    /// Priority: environment → config sockets → config TCP bootstrap.
    #[must_use]
    pub fn resolve(&self, primal: &str) -> Option<Transport> {
        // Tier 1: Environment variable
        let socket_env = format!("{}_SOCKET", primal.to_uppercase());
        if let Ok(socket_path) = std::env::var(&socket_env) {
            if !socket_path.is_empty() {
                return Some(Transport::Uds(PathBuf::from(socket_path)));
            }
        }

        let port_env = format!("{}_PORT", primal.to_uppercase());
        if let Ok(port_str) = std::env::var(&port_env) {
            if let Ok(port) = port_str.parse::<u16>() {
                let host_env = format!("{}_HOST", primal.to_uppercase());
                let host = std::env::var(&host_env).unwrap_or_else(|_| {
                    self.bootstrap_tcp
                        .as_ref()
                        .map_or_else(|| String::from(LOOPBACK_FALLBACK), |tcp| tcp.host.clone())
                });
                return Some(Transport::Tcp { host, port });
            }
        }

        // Tier 2: Config socket paths
        if let Some(socket_template) = self.sockets.get(primal) {
            let expanded = expand_xdg(socket_template);
            let path = PathBuf::from(&expanded);
            if path.exists() {
                return Some(Transport::Uds(path));
            }
        }

        // Tier 3: TCP bootstrap (dev/desktop only)
        if let Some(tcp) = &self.bootstrap_tcp {
            if let Some(port_val) = tcp.ports.get(primal) {
                if let Some(port) = port_val.as_integer() {
                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "TOML ports are always u16-range integers"
                    )]
                    let port = port as u16;
                    return Some(Transport::Tcp {
                        host: tcp.host.clone(),
                        port,
                    });
                }
            }
        }

        None
    }

    /// Check if the configuration prefers UDS-only mode (VPS standard).
    #[must_use]
    pub fn is_uds_only(&self) -> bool {
        self.metadata
            .as_ref()
            .and_then(|m| m.vps_standard.as_deref())
            .is_some_and(|s| s == "uds_only")
    }
}

/// Expand `${XDG_RUNTIME_DIR}` in a socket path template.
fn expand_xdg(template: &str) -> String {
    if template.contains("${XDG_RUNTIME_DIR}") {
        let xdg = std::env::var(crate::env_keys::XDG_RUNTIME_DIR)
            .unwrap_or_else(|_| format!("/run/user/{}", nix_uid()));
        template.replace("${XDG_RUNTIME_DIR}", &xdg)
    } else {
        template.to_owned()
    }
}

/// Get the current user's UID without libc (read /proc/self/status).
fn nix_uid() -> u32 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse().ok())
        })
        .unwrap_or(1000)
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    const SAMPLE_CONFIG: &str = r#"
[metadata]
vps_standard = "uds_only"
transport_preference = "uds"

[sockets]
discovery = "${XDG_RUNTIME_DIR}/ecoPrimals/discovery.sock"
beardog = "${XDG_RUNTIME_DIR}/ecoPrimals/beardog.sock"
toadstool = "${XDG_RUNTIME_DIR}/ecoPrimals/toadstool.sock"

[bootstrap_tcp]
host = "127.0.0.1"
beardog = 9100
toadstool = 9400
"#;

    #[test]
    fn parse_discovery_config() {
        let config: DiscoveryConfig = toml::from_str(SAMPLE_CONFIG).unwrap();
        assert!(config.is_uds_only());
        assert_eq!(config.sockets.len(), 3);
    }

    #[test]
    fn resolve_from_env() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // SAFETY: test-only env manipulation, serial test execution
        unsafe { std::env::set_var("BEARDOG_SOCKET", "/tmp/test-beardog.sock") };
        let config: DiscoveryConfig = toml::from_str(SAMPLE_CONFIG).unwrap();
        let transport = config.resolve("beardog");
        assert_eq!(
            transport,
            Some(Transport::Uds(PathBuf::from("/tmp/test-beardog.sock")))
        );
        unsafe { std::env::remove_var("BEARDOG_SOCKET") };
    }

    #[test]
    fn resolve_tcp_fallback() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // SAFETY: test-only env manipulation
        unsafe { std::env::remove_var("TOADSTOOL_SOCKET") };
        unsafe { std::env::remove_var("TOADSTOOL_PORT") };
        let config: DiscoveryConfig = toml::from_str(SAMPLE_CONFIG).unwrap();
        let transport = config.resolve("toadstool");
        assert_eq!(
            transport,
            Some(Transport::Tcp {
                host: String::from("127.0.0.1"),
                port: 9400,
            })
        );
    }

    #[test]
    fn is_uds_only_false_without_metadata() {
        let config: DiscoveryConfig = toml::from_str(
            r#"
[sockets]
discovery = "/tmp/discovery.sock"
"#,
        )
        .unwrap();
        assert!(!config.is_uds_only());
    }

    #[test]
    fn is_uds_only_false_with_other_standard() {
        let config: DiscoveryConfig = toml::from_str(
            r#"
[metadata]
vps_standard = "tcp_allowed"
"#,
        )
        .unwrap();
        assert!(!config.is_uds_only());
    }

    #[test]
    fn expand_xdg_uses_env_var() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // SAFETY: test-only env manipulation
        let dir = tempfile::tempdir().unwrap();
        let runtime = dir.path().to_str().unwrap();
        unsafe { std::env::set_var("XDG_RUNTIME_DIR", runtime) };
        unsafe { std::env::remove_var("DISCOVERY_SOCKET") };
        unsafe { std::env::remove_var("DISCOVERY_PORT") };

        let sock_dir = dir.path().join("ecoPrimals");
        std::fs::create_dir_all(&sock_dir).unwrap();
        let sock_path = sock_dir.join("discovery.sock");
        std::fs::write(&sock_path, b"").unwrap();

        // Verify expand_xdg works with the env var set
        let expanded = expand_xdg("${XDG_RUNTIME_DIR}/ecoPrimals/discovery.sock");
        assert_eq!(expanded, sock_path.to_str().unwrap());

        // Verify the file exists (necessary for resolve to use UDS tier)
        assert!(sock_path.exists());

        let config: DiscoveryConfig = toml::from_str(
            r#"
[sockets]
discovery = "${XDG_RUNTIME_DIR}/ecoPrimals/discovery.sock"
"#,
        )
        .unwrap();
        let transport = config.resolve("discovery");
        assert_eq!(transport, Some(Transport::Uds(sock_path)));

        unsafe { std::env::remove_var("XDG_RUNTIME_DIR") };
    }

    #[test]
    fn expand_xdg_falls_back_to_run_user_uid() {
        let _guard = ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let saved_xdg = std::env::var("XDG_RUNTIME_DIR").ok();
        // SAFETY: test-only env manipulation
        unsafe { std::env::remove_var("XDG_RUNTIME_DIR") };

        let uid = nix_uid();
        let expanded = expand_xdg("${XDG_RUNTIME_DIR}/ecoPrimals/fallbacktest.sock");
        assert_eq!(
            expanded,
            format!("/run/user/{uid}/ecoPrimals/fallbacktest.sock")
        );

        if let Some(xdg) = saved_xdg {
            unsafe { std::env::set_var("XDG_RUNTIME_DIR", xdg) };
        }
    }

    #[test]
    fn nix_uid_reads_proc_status() {
        let uid = nix_uid();
        assert!(uid > 0);
    }

    #[test]
    fn expand_xdg_passthrough_without_placeholder() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("plain.sock");
        std::fs::write(&sock_path, b"").unwrap();

        let config: DiscoveryConfig = toml::from_str(&format!(
            r#"
[sockets]
plain = "{}"
"#,
            sock_path.display()
        ))
        .unwrap();
        assert_eq!(config.resolve("plain"), Some(Transport::Uds(sock_path)));
    }
}
