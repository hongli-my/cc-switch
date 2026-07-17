//! Pi Agent 提供商预设配置
//!
//! 为 Pi Agent 提供常用的提供商预设，包括各种 API 兼容的提供商

use serde_json::json;

/// Pi Agent 支持的提供商预设列表
pub struct PiProviderPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider_type: String, // 对应 auth.json 中的 key
    pub base_url: Option<String>,
    pub env_vars: Vec<(String, String)>, // 环境变量模板
}

impl Default for PiProviderPreset {
    fn default() -> Self {
        Self {
            id: "pi-default".to_string(),
            name: "Pi Default".to_string(),
            description: "Pi Agent 默认配置".to_string(),
            provider_type: "anthropic".to_string(),
            base_url: None,
            env_vars: vec![],
        }
    }
}

/// 获取所有 Pi 提供商预设
pub fn get_pi_provider_presets() -> Vec<PiProviderPreset> {
    vec![
        PiProviderPreset {
            id: "pi-anthropic".to_string(),
            name: "Anthropic (Claude)".to_string(),
            description: "使用 Anthropic API (Claude 模型)".to_string(),
            provider_type: "anthropic".to_string(),
            base_url: None,
            env_vars: vec![
                ("ANTHROPIC_API_KEY".to_string(), "".to_string()),
            ],
        },
        PiProviderPreset {
            id: "pi-openai".to_string(),
            name: "OpenAI".to_string(),
            description: "使用 OpenAI API (GPT 模型)".to_string(),
            provider_type: "openai".to_string(),
            base_url: None,
            env_vars: vec![
                ("OPENAI_API_KEY".to_string(), "".to_string()),
            ],
        },
        PiProviderPreset {
            id: "pi-gemini".to_string(),
            name: "Google Gemini".to_string(),
            description: "使用 Google Gemini API".to_string(),
            provider_type: "google".to_string(),
            base_url: None,
            env_vars: vec![
                ("GEMINI_API_KEY".to_string(), "".to_string()),
            ],
        },
        PiProviderPreset {
            id: "pi-deepseek".to_string(),
            name: "DeepSeek".to_string(),
            description: "使用 DeepSeek API".to_string(),
            provider_type: "deepseek".to_string(),
            base_url: None,
            env_vars: vec![
                ("DEEPSEEK_API_KEY".to_string(), "".to_string()),
            ],
        },
        PiProviderPreset {
            id: "pi-openrouter".to_string(),
            name: "OpenRouter".to_string(),
            description: "使用 OpenRouter API (聚合多个模型)".to_string(),
            provider_type: "openrouter".to_string(),
            base_url: Some("https://openrouter.ai/api/v1".to_string()),
            env_vars: vec![
                ("OPENROUTER_API_KEY".to_string(), "".to_string()),
            ],
        },
        PiProviderPreset {
            id: "pi-groq".to_string(),
            name: "Groq".to_string(),
            description: "使用 Groq API (高速推理)".to_string(),
            provider_type: "groq".to_string(),
            base_url: None,
            env_vars: vec![
                ("GROQ_API_KEY".to_string(), "".to_string()),
            ],
        },
        PiProviderPreset {
            id: "pi-mistral".to_string(),
            name: "Mistral AI".to_string(),
            description: "使用 Mistral AI API".to_string(),
            provider_type: "mistral".to_string(),
            base_url: None,
            env_vars: vec![
                ("MISTRAL_API_KEY".to_string(), "".to_string()),
            ],
        },
        PiProviderPreset {
            id: "pi-xai".to_string(),
            name: "xAI (Grok)".to_string(),
            description: "使用 xAI Grok API".to_string(),
            provider_type: "xai".to_string(),
            base_url: None,
            env_vars: vec![
                ("XAI_API_KEY".to_string(), "".to_string()),
            ],
        },
        PiProviderPreset {
            id: "pi-together".to_string(),
            name: "Together AI".to_string(),
            description: "使用 Together AI API".to_string(),
            provider_type: "together".to_string(),
            base_url: None,
            env_vars: vec![
                ("TOGETHER_API_KEY".to_string(), "".to_string()),
            ],
        },
        PiProviderPreset {
            id: "pi-fireworks".to_string(),
            name: "Fireworks AI".to_string(),
            description: "使用 Fireworks AI API".to_string(),
            provider_type: "fireworks".to_string(),
            base_url: None,
            env_vars: vec![
                ("FIREWORKS_API_KEY".to_string(), "".to_string()),
            ],
        },
        PiProviderPreset {
            id: "pi-custom".to_string(),
            name: "Custom (OpenAI Compatible)".to_string(),
            description: "自定义 OpenAI 兼容 API".to_string(),
            provider_type: "custom".to_string(),
            base_url: Some("".to_string()),
            env_vars: vec![
                ("CUSTOM_API_KEY".to_string(), "".to_string()),
            ],
        },
    ]
}

/// 根据 provider_type 获取预设配置
pub fn get_preset_by_provider_type(provider_type: &str) -> Option<PiProviderPreset> {
    get_pi_provider_presets()
        .into_iter()
        .find(|p| p.provider_type == provider_type)
}

/// 生成 Pi 供应商的默认 settingsConfig
pub fn generate_pi_settings_config(
    provider_type: &str,
    api_key: Option<String>,
    base_url: Option<String>,
) -> serde_json::Value {
    let mut config = json!({
        "providerType": provider_type,
    });

    if let Some(key) = api_key {
        config["apiKey"] = json!(key);
    }

    if let Some(url) = base_url {
        config["baseUrl"] = json!(url);
    }

    // 添加环境变量映射（用于用量统计等）
    let env_vars = match provider_type {
        "anthropic" => json!({
            "ANTHROPIC_API_KEY": api_key.unwrap_or_default(),
            "ANTHROPIC_BASE_URL": base_url.unwrap_or_default(),
        }),
        "openai" => json!({
            "OPENAI_API_KEY": api_key.unwrap_or_default(),
            "OPENAI_BASE_URL": base_url.unwrap_or_default(),
        }),
        "google" => json!({
            "GEMINI_API_KEY": api_key.unwrap_or_default(),
            "GOOGLE_GEMINI_BASE_URL": base_url.unwrap_or_default(),
        }),
        "deepseek" => json!({
            "DEEPSEEK_API_KEY": api_key.unwrap_or_default(),
        }),
        "openrouter" => json!({
            "OPENROUTER_API_KEY": api_key.unwrap_or_default(),
        }),
        "groq" => json!({
            "GROQ_API_KEY": api_key.unwrap_or_default(),
        }),
        "mistral" => json!({
            "MISTRAL_API_KEY": api_key.unwrap_or_default(),
        }),
        "xai" => json!({
            "XAI_API_KEY": api_key.unwrap_or_default(),
        }),
        "together" => json!({
            "TOGETHER_API_KEY": api_key.unwrap_or_default(),
        }),
        "fireworks" => json!({
            "FIREWORKS_API_KEY": api_key.unwrap_or_default(),
        }),
        _ => json!({}),
    };

    config["env"] = env_vars;
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pi_provider_presets() {
        let presets = get_pi_provider_presets();
        assert!(!presets.is_empty());
        assert!(presets.iter().any(|p| p.provider_type == "anthropic"));
        assert!(presets.iter().any(|p| p.provider_type == "openai"));
    }

    #[test]
    fn test_generate_pi_settings_config() {
        let config = generate_pi_settings_config(
            "anthropic",
            Some("sk-ant-123".to_string()),
            Some("https://api.anthropic.com".to_string()),
        );

        assert_eq!(config["providerType"], "anthropic");
        assert_eq!(config["apiKey"], "sk-ant-123");
        assert_eq!(config["baseUrl"], "https://api.anthropic.com");
        assert_eq!(config["env"]["ANTHROPIC_API_KEY"], "sk-ant-123");
    }

    #[test]
    fn test_get_preset_by_provider_type() {
        let preset = get_preset_by_provider_type("openai");
        assert!(preset.is_some());
        assert_eq!(preset.unwrap().name, "OpenAI");
    }
}
