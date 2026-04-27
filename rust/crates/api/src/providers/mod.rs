#![allow(clippy::cast_possible_truncation)]
use std::future::Future;
use std::pin::Pin;

use serde::Serialize;

use crate::error::ApiError;
use crate::types::{MessageRequest, MessageResponse};

pub mod openai_compat;

#[allow(dead_code)]
pub type ProviderFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ApiError>> + Send + 'a>>;

#[allow(dead_code)]
pub trait Provider {
    type Stream;

    fn send_message<'a>(
        &'a self,
        request: &'a MessageRequest,
    ) -> ProviderFuture<'a, MessageResponse>;

    fn stream_message<'a>(
        &'a self,
        request: &'a MessageRequest,
    ) -> ProviderFuture<'a, Self::Stream>;
}

/// Provider routing kind. After the NIM-only refactor, every model resolves
/// through NVIDIA NIM (OpenAI-compatible chat completions). The enum is kept
/// (rather than collapsed to a struct) so existing pattern-match call sites
/// keep compiling and so we have a clean place to add future providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Nim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ProviderMetadata {
    pub provider: ProviderKind,
    pub auth_env: &'static str,
    pub base_url_env: &'static str,
    pub default_base_url: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelTokenLimit {
    pub max_output_tokens: u32,
    pub context_window_tokens: u32,
}

#[allow(dead_code)]
const NIM_METADATA: ProviderMetadata = ProviderMetadata {
    provider: ProviderKind::Nim,
    auth_env: "NVIDIA_NIM_API_KEY",
    base_url_env: "NVIDIA_NIM_BASE_URL",
    default_base_url: openai_compat::DEFAULT_NIM_BASE_URL,
};

/// Strip the optional `nvidia_nim/` or `nim/` namespace prefix so the bare
/// `vendor/model` id can be sent directly to the NIM API.
fn strip_namespace(model: &str) -> &str {
    model
        .strip_prefix("nvidia_nim/")
        .or_else(|| model.strip_prefix("nim/"))
        .unwrap_or(model)
}

/// Canonicalize a model string: strip namespace prefix and trim whitespace.
/// Returns the bare `vendor/model` id ready for the NIM API.
#[must_use]
pub fn resolve_model_alias(model: &str) -> String {
    let trimmed = model.trim();
    let stripped = strip_namespace(trimmed);
    stripped.to_string()
}

#[allow(dead_code)]
#[must_use]
pub fn metadata_for_model(_model: &str) -> Option<ProviderMetadata> {
    Some(NIM_METADATA)
}

#[must_use]
pub fn detect_provider_kind(_model: &str) -> ProviderKind {
    ProviderKind::Nim
}

#[must_use]
pub fn max_tokens_for_model(model: &str) -> u32 {
    model_token_limit(model).map_or(8_192, |limit| limit.max_output_tokens)
}

/// Returns the effective max output tokens for a model, preferring a plugin
/// override when present. Falls back to [`max_tokens_for_model`] when the
/// override is `None`.
#[must_use]
pub fn max_tokens_for_model_with_override(model: &str, plugin_override: Option<u32>) -> u32 {
    plugin_override.unwrap_or_else(|| max_tokens_for_model(model))
}

/// Per-model token limits for the NIM models we expose by default. Numbers
/// reflect the public NVIDIA NIM model cards on `build.nvidia.com` /
/// `integrate.api.nvidia.com` at time of writing — adjust if the upstream
/// publishes different caps for your tenant.
#[must_use]
pub fn model_token_limit(model: &str) -> Option<ModelTokenLimit> {
    let canonical = resolve_model_alias(model);
    match canonical.as_str() {
        // Z.AI GLM family — 128k context, generous output budget.
        "z-ai/glm-5.1" | "z-ai/glm5" | "z-ai/glm4.7" => Some(ModelTokenLimit {
            max_output_tokens: 16_384,
            context_window_tokens: 131_072,
        }),
        // Moonshot Kimi K2 family — 256k context.
        "moonshotai/kimi-k2.5"
        | "moonshotai/kimi-k2-instruct"
        | "moonshotai/kimi-k2-instruct-0905"
        | "moonshotai/kimi-k2-thinking" => Some(ModelTokenLimit {
            max_output_tokens: 16_384,
            context_window_tokens: 256_000,
        }),
        // MiniMax M2 family — 256k context.
        "minimaxai/minimax-m2.5" | "minimaxai/minimax-m2.7" => Some(ModelTokenLimit {
            max_output_tokens: 16_384,
            context_window_tokens: 256_000,
        }),
        // DeepSeek V3 / V4 — 131k context.
        "deepseek-ai/deepseek-v3.2"
        | "deepseek-ai/deepseek-v3.1-terminus"
        | "deepseek-ai/deepseek-v4-pro"
        | "deepseek-ai/deepseek-v4-flash" => Some(ModelTokenLimit {
            max_output_tokens: 16_384,
            context_window_tokens: 131_072,
        }),
        // Qwen 3 family — 256k context.
        "qwen/qwen3-coder-480b-a35b-instruct"
        | "qwen/qwen3-next-80b-a3b-instruct"
        | "qwen/qwen3-next-80b-a3b-thinking"
        | "qwen/qwen3.5-122b-a10b"
        | "qwen/qwen3.5-397b-a17b" => Some(ModelTokenLimit {
            max_output_tokens: 16_384,
            context_window_tokens: 262_144,
        }),
        // Conservative fallback for any other NIM model that we have not yet
        // pinned in this table.
        _ => None,
    }
}

pub fn preflight_message_request(request: &MessageRequest) -> Result<(), ApiError> {
    let Some(limit) = model_token_limit(&request.model) else {
        return Ok(());
    };

    let estimated_input_tokens = estimate_message_request_input_tokens(request);
    let estimated_total_tokens = estimated_input_tokens.saturating_add(request.max_tokens);
    if estimated_total_tokens > limit.context_window_tokens {
        return Err(ApiError::ContextWindowExceeded {
            model: resolve_model_alias(&request.model),
            estimated_input_tokens,
            requested_output_tokens: request.max_tokens,
            estimated_total_tokens,
            context_window_tokens: limit.context_window_tokens,
        });
    }

    Ok(())
}

fn estimate_message_request_input_tokens(request: &MessageRequest) -> u32 {
    let mut estimate = estimate_serialized_tokens(&request.messages);
    estimate = estimate.saturating_add(estimate_serialized_tokens(&request.system));
    estimate = estimate.saturating_add(estimate_serialized_tokens(&request.tools));
    estimate = estimate.saturating_add(estimate_serialized_tokens(&request.tool_choice));
    estimate
}

fn estimate_serialized_tokens<T: Serialize>(value: &T) -> u32 {
    serde_json::to_vec(value)
        .ok()
        .map_or(0, |bytes| (bytes.len() / 4 + 1) as u32)
}

/// Parse a `.env` file body into key/value pairs using a minimal `KEY=VALUE`
/// grammar. Lines that are blank, start with `#`, or do not contain `=` are
/// ignored. Surrounding double or single quotes are stripped from the value.
/// An optional leading `export ` prefix on the key is also stripped so files
/// shared with shell `source` workflows still parse cleanly.
pub(crate) fn parse_dotenv(content: &str) -> std::collections::HashMap<String, String> {
    let mut values = std::collections::HashMap::new();
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = line.split_once('=') else {
            continue;
        };
        let trimmed_key = raw_key.trim();
        let key = trimmed_key
            .strip_prefix("export ")
            .map_or(trimmed_key, str::trim)
            .to_string();
        if key.is_empty() {
            continue;
        }
        let trimmed_value = raw_value.trim();
        let unquoted = if (trimmed_value.starts_with('"') && trimmed_value.ends_with('"')
            || trimmed_value.starts_with('\'') && trimmed_value.ends_with('\''))
            && trimmed_value.len() >= 2
        {
            &trimmed_value[1..trimmed_value.len() - 1]
        } else {
            trimmed_value
        };
        values.insert(key, unquoted.to_string());
    }
    values
}

/// Load and parse a `.env` file from the given path. Missing files yield
/// `None` instead of an error so callers can use this as a soft fallback.
pub(crate) fn load_dotenv_file(
    path: &std::path::Path,
) -> Option<std::collections::HashMap<String, String>> {
    let content = std::fs::read_to_string(path).ok()?;
    Some(parse_dotenv(&content))
}

/// Look up `key` in a `.env` file located in the current working directory.
/// Returns `None` when the file is missing, the key is absent, or the value
/// is empty.
pub(crate) fn dotenv_value(key: &str) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let values = load_dotenv_file(&cwd.join(".env"))?;
    values.get(key).filter(|value| !value.is_empty()).cloned()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::error::ApiError;
    use crate::types::{
        InputContentBlock, InputMessage, MessageRequest, ToolChoice, ToolDefinition,
    };

    use super::{
        detect_provider_kind, load_dotenv_file, max_tokens_for_model,
        max_tokens_for_model_with_override, metadata_for_model, model_token_limit, parse_dotenv,
        preflight_message_request, resolve_model_alias, ProviderKind,
    };

    #[test]
    fn strips_nvidia_nim_namespace_prefix() {
        assert_eq!(
            resolve_model_alias("nvidia_nim/z-ai/glm-5.1"),
            "z-ai/glm-5.1"
        );
        assert_eq!(resolve_model_alias("nim/z-ai/glm-5.1"), "z-ai/glm-5.1");
        assert_eq!(resolve_model_alias("qwen/qwen3.5-122b-a10b"), "qwen/qwen3.5-122b-a10b");
    }

    #[test]
    fn passes_through_unknown_model_ids_unchanged() {
        // Users can target any NIM-served model id directly without first
        // having to register an alias here.
        assert_eq!(
            resolve_model_alias("openai/gpt-oss-120b"),
            "openai/gpt-oss-120b"
        );
        assert_eq!(
            resolve_model_alias("nvidia/llama-3.3-nemotron-super-49b"),
            "nvidia/llama-3.3-nemotron-super-49b"
        );
    }

    #[test]
    fn every_model_routes_to_nim() {
        assert_eq!(detect_provider_kind("z-ai/glm-5.1"), ProviderKind::Nim);
        assert_eq!(detect_provider_kind("anything-else"), ProviderKind::Nim);
        let meta = metadata_for_model("kimi").expect("nim metadata");
        assert_eq!(meta.auth_env, "NVIDIA_NIM_API_KEY");
        assert_eq!(meta.base_url_env, "NVIDIA_NIM_BASE_URL");
    }

    #[test]
    fn known_nim_models_carry_token_limit_metadata() {
        let glm = model_token_limit("z-ai/glm-5.1").expect("glm-5.1 limits");
        assert_eq!(glm.context_window_tokens, 131_072);
        let kimi = model_token_limit("moonshotai/kimi-k2.5").expect("kimi limits");
        assert_eq!(kimi.context_window_tokens, 256_000);
    }

    #[test]
    fn max_tokens_falls_back_to_default_for_unknown_models() {
        assert_eq!(max_tokens_for_model("unknown-model-xyz"), 8_192);
        assert_eq!(max_tokens_for_model("z-ai/glm-5.1"), 16_384);
    }

    #[test]
    fn plugin_override_wins_over_model_default() {
        assert_eq!(
            max_tokens_for_model_with_override("z-ai/glm-5.1", Some(12_345)),
            12_345
        );
        assert_eq!(
            max_tokens_for_model_with_override("z-ai/glm-5.1", None),
            16_384
        );
    }

    #[test]
    fn preflight_blocks_oversized_requests_for_known_models() {
        let request = MessageRequest {
            model: "z-ai/glm-5.1".to_string(),
            max_tokens: 16_384,
            messages: vec![InputMessage {
                role: "user".to_string(),
                content: vec![InputContentBlock::Text {
                    text: "x".repeat(600_000),
                }],
            }],
            system: Some("Keep it short.".to_string()),
            tools: Some(vec![ToolDefinition {
                name: "weather".to_string(),
                description: Some("Fetches weather".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": { "city": { "type": "string" } },
                }),
            }]),
            tool_choice: Some(ToolChoice::Auto),
            stream: true,
            ..Default::default()
        };

        let error = preflight_message_request(&request)
            .expect_err("oversized request should be rejected before the provider call");

        match error {
            ApiError::ContextWindowExceeded {
                model,
                context_window_tokens,
                ..
            } => {
                assert_eq!(model, "z-ai/glm-5.1");
                assert_eq!(context_window_tokens, 131_072);
            }
            other => panic!("expected context-window preflight failure, got {other:?}"),
        }
    }

    #[test]
    fn preflight_skips_models_without_known_limits() {
        let request = MessageRequest {
            model: "novel-experimental-model".to_string(),
            max_tokens: 1024,
            messages: vec![InputMessage {
                role: "user".to_string(),
                content: vec![InputContentBlock::Text {
                    text: "x".repeat(600_000),
                }],
            }],
            ..Default::default()
        };

        preflight_message_request(&request)
            .expect("models without context metadata should skip the guarded preflight");
    }

    #[test]
    fn parse_dotenv_extracts_keys_handles_comments_quotes_and_export_prefix() {
        let body = "\
# this is a comment

NVIDIA_NIM_API_KEY=plain-value
NVIDIA_NIM_BASE_URL=\"https://integrate.api.nvidia.com/v1\"
ENABLE_THINKING='true'
export EXTRA_KEY=exported-value
   PADDED_KEY  =  padded-value  
EMPTY_VALUE=
NO_EQUALS_LINE
";
        let values = parse_dotenv(body);
        assert_eq!(
            values.get("NVIDIA_NIM_API_KEY").map(String::as_str),
            Some("plain-value")
        );
        assert_eq!(
            values.get("NVIDIA_NIM_BASE_URL").map(String::as_str),
            Some("https://integrate.api.nvidia.com/v1")
        );
        assert_eq!(values.get("ENABLE_THINKING").map(String::as_str), Some("true"));
        assert_eq!(
            values.get("EXTRA_KEY").map(String::as_str),
            Some("exported-value")
        );
        assert_eq!(
            values.get("PADDED_KEY").map(String::as_str),
            Some("padded-value")
        );
        assert_eq!(values.get("EMPTY_VALUE").map(String::as_str), Some(""));
        assert!(!values.contains_key("NO_EQUALS_LINE"));
    }

    #[test]
    fn load_dotenv_file_reads_keys_from_disk_and_returns_none_when_missing() {
        let temp_root = std::env::temp_dir().join(format!(
            "api-dotenv-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos())
        ));
        std::fs::create_dir_all(&temp_root).expect("create temp dir");
        let env_path = temp_root.join(".env");
        std::fs::write(
            &env_path,
            "NVIDIA_NIM_API_KEY=secret-from-file\n# comment\nENABLE_THINKING=\"1\"\n",
        )
        .expect("write .env");
        let missing_path = temp_root.join("does-not-exist.env");

        let loaded = load_dotenv_file(&env_path).expect("file should load");
        let missing = load_dotenv_file(&missing_path);

        assert_eq!(
            loaded.get("NVIDIA_NIM_API_KEY").map(String::as_str),
            Some("secret-from-file")
        );
        assert_eq!(
            loaded.get("ENABLE_THINKING").map(String::as_str),
            Some("1")
        );
        assert!(missing.is_none());

        let _ = std::fs::remove_dir_all(&temp_root);
    }
}
