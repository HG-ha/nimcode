//! Integration tests for `ProviderClient` after the NIM-only refactor.
//!
//! Pre-refactor this file covered the multi-provider dispatch surface
//! (Anthropic OAuth, xAI grok aliases, DashScope qwen aliases). Those code
//! paths have been removed in favor of a single NVIDIA NIM backend, so the
//! tests below verify the new single-provider behavior.

use std::ffi::OsString;
use std::sync::{Mutex, OnceLock};

use api::{read_base_url, ApiError, ProviderClient, ProviderKind};

#[test]
fn provider_client_resolves_glm_alias_to_nim() {
    let _lock = env_lock();
    let _key = EnvVarGuard::set("NVIDIA_NIM_API_KEY", Some("nvapi-test-key"));

    let client = ProviderClient::from_model("opus")
        .expect("`opus` alias should resolve to a NIM client");

    assert_eq!(client.provider_kind(), ProviderKind::Nim);
}

#[test]
fn provider_client_reports_missing_nim_credentials() {
    let _lock = env_lock();
    let _key = EnvVarGuard::set("NVIDIA_NIM_API_KEY", None);

    let error = ProviderClient::from_model("z-ai/glm-5.1")
        .expect_err("calls without NVIDIA_NIM_API_KEY should fail fast");

    match error {
        ApiError::MissingCredentials {
            provider, env_vars, ..
        } => {
            assert_eq!(provider, "NVIDIA NIM");
            assert_eq!(env_vars, &["NVIDIA_NIM_API_KEY"]);
        }
        other => panic!("expected missing NIM credentials, got {other:?}"),
    }
}

#[test]
fn read_base_url_prefers_env_override() {
    let _lock = env_lock();
    let _override = EnvVarGuard::set(
        "NVIDIA_NIM_BASE_URL",
        Some("https://example.nim.test/v1"),
    );

    assert_eq!(read_base_url(), "https://example.nim.test/v1");
}

#[test]
fn read_base_url_defaults_to_integrate_api_nvidia_com() {
    let _lock = env_lock();
    let _override = EnvVarGuard::set("NVIDIA_NIM_BASE_URL", None);

    assert_eq!(read_base_url(), "https://integrate.api.nvidia.com/v1");
}

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct EnvVarGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: Option<&str>) -> Self {
        let original = std::env::var_os(key);
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match self.original.take() {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}
