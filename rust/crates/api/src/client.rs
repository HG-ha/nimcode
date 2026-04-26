use crate::error::ApiError;
use crate::prompt_cache::{PromptCache, PromptCacheRecord, PromptCacheStats};
use crate::providers::openai_compat::{self, OpenAiCompatClient, OpenAiCompatConfig};
use crate::providers::{self, ProviderKind};
use crate::types::{MessageRequest, MessageResponse, StreamEvent};

/// Top-level provider client. After the NIM-only refactor every model is
/// dispatched through `OpenAiCompatClient` configured for NVIDIA NIM. The
/// enum is preserved (instead of being collapsed to a struct) so that
/// downstream code can keep matching on `ProviderClient::Nim(_)` without
/// rippling type-level changes.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum ProviderClient {
    Nim(OpenAiCompatClient),
}

impl ProviderClient {
    pub fn from_model(model: &str) -> Result<Self, ApiError> {
        Self::from_model_with_anthropic_auth(model, None)
    }

    /// Construct a NIM client. The legacy `anthropic_auth` parameter is
    /// preserved in the signature only to keep CLI call-sites compiling
    /// during the migration; it is unused on the NIM path.
    pub fn from_model_with_anthropic_auth(
        model: &str,
        _anthropic_auth: Option<AuthSource>,
    ) -> Result<Self, ApiError> {
        let _resolved_model = providers::resolve_model_alias(model);
        Ok(Self::Nim(OpenAiCompatClient::from_env(
            OpenAiCompatConfig::nim(),
        )?))
    }

    #[must_use]
    pub const fn provider_kind(&self) -> ProviderKind {
        match self {
            Self::Nim(_) => ProviderKind::Nim,
        }
    }

    /// Prompt-caching is an Anthropic-only feature. Retained as a no-op so
    /// CLI call sites keep compiling without conditional logic.
    #[must_use]
    pub fn with_prompt_cache(self, _prompt_cache: PromptCache) -> Self {
        self
    }

    #[must_use]
    pub const fn prompt_cache_stats(&self) -> Option<PromptCacheStats> {
        None
    }

    #[must_use]
    pub const fn take_last_prompt_cache_record(&self) -> Option<PromptCacheRecord> {
        None
    }

    pub async fn send_message(
        &self,
        request: &MessageRequest,
    ) -> Result<MessageResponse, ApiError> {
        match self {
            Self::Nim(client) => client.send_message(request).await,
        }
    }

    pub async fn stream_message(
        &self,
        request: &MessageRequest,
    ) -> Result<MessageStream, ApiError> {
        match self {
            Self::Nim(client) => client
                .stream_message(request)
                .await
                .map(MessageStream::OpenAiCompat),
        }
    }

    /// Return the live list of model ids exposed by the configured backend.
    /// Hits `<base>/models` and pulls `data[].id`. Used by the CLI to drive
    /// the `/model` autocompletion / picker without hard-coding a static
    /// catalog.
    pub async fn list_models(&self) -> Result<Vec<String>, ApiError> {
        match self {
            Self::Nim(client) => client.list_models().await,
        }
    }
}

/// Construct a NIM client from `NVIDIA_NIM_API_KEY` / `NVIDIA_NIM_BASE_URL`
/// purely for the purpose of fetching `/v1/models`. Returns an error when
/// the API key is missing so the CLI can surface a clear "set NVIDIA_NIM_API_KEY"
/// hint instead of a confusing `404`.
pub async fn fetch_nim_model_ids() -> Result<Vec<String>, ApiError> {
    let client = OpenAiCompatClient::from_env(OpenAiCompatConfig::nim())?;
    client.list_models().await
}

#[derive(Debug)]
pub enum MessageStream {
    OpenAiCompat(openai_compat::MessageStream),
}

impl MessageStream {
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::OpenAiCompat(stream) => stream.request_id(),
        }
    }

    pub async fn next_event(&mut self) -> Result<Option<StreamEvent>, ApiError> {
        match self {
            Self::OpenAiCompat(stream) => stream.next_event().await,
        }
    }
}

/// Returns the configured NIM base URL, honoring `NVIDIA_NIM_BASE_URL` when
/// set and falling back to `integrate.api.nvidia.com/v1` otherwise.
#[must_use]
pub fn read_base_url() -> String {
    openai_compat::read_base_url(OpenAiCompatConfig::nim())
}

/// Backwards-compatible alias preserved for CLI call sites that still call
/// `read_xai_base_url()`. After the NIM-only refactor it returns the NIM
/// base URL too — every model now goes through the same endpoint.
#[must_use]
pub fn read_xai_base_url() -> String {
    read_base_url()
}

// --- Legacy auth shims -------------------------------------------------
//
// The Anthropic provider used to expose a rich auth model (env-based API
// keys, OAuth tokens, refresh flows). NIM only needs a single API key
// (`NVIDIA_NIM_API_KEY`). We keep these types as no-op shims so the CLI's
// existing /login plumbing and the runtime's `OAuthTokenSet` keep linking
// while the broader CLI surface gets simplified. They intentionally have
// no behavior on the NIM path.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthSource;

impl AuthSource {
    #[must_use]
    pub fn from_api_key(_api_key: String) -> Self {
        Self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OAuthTokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at_unix_ms: Option<i64>,
}

#[must_use]
pub const fn oauth_token_is_expired(_token: &OAuthTokenSet) -> bool {
    false
}

/// No-op stub: NIM authenticates with a single env-var API key, so there
/// is no saved OAuth token to resolve. Returns `Ok(None)` so the CLI's
/// startup flow can continue without OAuth.
#[allow(clippy::missing_const_for_fn)]
pub fn resolve_saved_oauth_token() -> Result<Option<OAuthTokenSet>, ApiError> {
    Ok(None)
}

/// Resolve startup auth. With NIM-only we ignore the optional saved-token
/// loader and always return the empty `AuthSource` — `OpenAiCompatClient`
/// reads `NVIDIA_NIM_API_KEY` from the env on its own.
pub fn resolve_startup_auth_source<F>(_load_saved: F) -> Result<AuthSource, ApiError>
where
    F: FnOnce() -> Result<Option<OAuthTokenSet>, ApiError>,
{
    Ok(AuthSource)
}

#[cfg(test)]
mod tests {
    use super::ProviderClient;
    use crate::providers::{detect_provider_kind, resolve_model_alias, ProviderKind};

    #[test]
    fn resolves_glm_aliases() {
        assert_eq!(resolve_model_alias("opus"), "deepseek-ai/deepseek-v4-pro");
        assert_eq!(resolve_model_alias("kimi"), "moonshotai/kimi-k2.5");
    }

    #[test]
    fn every_model_routes_to_nim() {
        assert_eq!(detect_provider_kind("z-ai/glm-5.1"), ProviderKind::Nim);
        assert_eq!(detect_provider_kind("anything"), ProviderKind::Nim);
    }

    #[test]
    fn from_model_requires_nvidia_nim_api_key() {
        // Without NVIDIA_NIM_API_KEY the NIM client construction should fail
        // with a clear missing-credentials error rather than panicking.
        let _previous = std::env::var_os("NVIDIA_NIM_API_KEY");
        std::env::remove_var("NVIDIA_NIM_API_KEY");
        let outcome = ProviderClient::from_model("z-ai/glm-5.1");
        if let Some(value) = _previous {
            std::env::set_var("NVIDIA_NIM_API_KEY", value);
        }
        let err = outcome.expect_err("expected missing-credentials error");
        let rendered = err.to_string();
        assert!(
            rendered.contains("NVIDIA_NIM_API_KEY") || rendered.contains("NVIDIA NIM"),
            "error should reference NIM credentials, got: {rendered}"
        );
    }
}
