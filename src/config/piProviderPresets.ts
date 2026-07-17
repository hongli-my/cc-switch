//! Pi Agent 提供商预设配置
//!
//! 为 Pi Agent 提供常用的提供商预设，包括各种 API 兼容的提供商

/// Pi Agent 支持的提供商预设
export interface PiProviderPreset {
  id: string;
  name: string;
  description: string;
  /** 对应 auth.json 中的 provider_type */
  provider_type: string;
  /** 默认 base URL（可选） */
  base_url?: string;
  /** 环境变量模板（用于用量统计等） */
  env_vars: Record<string, string>;
}

/// 获取所有 Pi 提供商预设
export function getPiProviderPresets(): PiProviderPreset[] {
  return [
    {
      id: "pi-anthropic",
      name: "Anthropic (Claude)",
      description: "使用 Anthropic API (Claude 模型)",
      provider_type: "anthropic",
      env_vars: { ANTHROPIC_API_KEY: "" },
    },
    {
      id: "pi-openai",
      name: "OpenAI",
      description: "使用 OpenAI API (GPT 模型)",
      provider_type: "openai",
      env_vars: { OPENAI_API_KEY: "" },
    },
    {
      id: "pi-gemini",
      name: "Google Gemini",
      description: "使用 Google Gemini API",
      provider_type: "google",
      env_vars: { GEMINI_API_KEY: "" },
    },
    {
      id: "pi-deepseek",
      name: "DeepSeek",
      description: "使用 DeepSeek API",
      provider_type: "deepseek",
      env_vars: { DEEPSEEK_API_KEY: "" },
    },
    {
      id: "pi-openrouter",
      name: "OpenRouter",
      description: "使用 OpenRouter API (聚合多个模型)",
      provider_type: "openrouter",
      base_url: "https://openrouter.ai/api/v1",
      env_vars: { OPENROUTER_API_KEY: "" },
    },
    {
      id: "pi-groq",
      name: "Groq",
      description: "使用 Groq API (高速推理)",
      provider_type: "groq",
      env_vars: { GROQ_API_KEY: "" },
    },
    {
      id: "pi-mistral",
      name: "Mistral AI",
      description: "使用 Mistral AI API",
      provider_type: "mistral",
      env_vars: { MISTRAL_API_KEY: "" },
    },
    {
      id: "pi-xai",
      name: "xAI (Grok)",
      description: "使用 xAI Grok API",
      provider_type: "xai",
      env_vars: { XAI_API_KEY: "" },
    },
    {
      id: "pi-together",
      name: "Together AI",
      description: "使用 Together AI API",
      provider_type: "together",
      env_vars: { TOGETHER_API_KEY: "" },
    },
    {
      id: "pi-fireworks",
      name: "Fireworks AI",
      description: "使用 Fireworks AI API",
      provider_type: "fireworks",
      env_vars: { FIREWORKS_API_KEY: "" },
    },
    {
      id: "pi-custom",
      name: "Custom (OpenAI Compatible)",
      description: "自定义 OpenAI 兼容 API",
      provider_type: "custom",
      base_url: "",
      env_vars: { CUSTOM_API_KEY: "" },
    },
  ];
}

/// 根据 provider_type 获取预设配置
export function getPresetByProviderType(
  providerType: string,
): PiProviderPreset | undefined {
  return getPiProviderPresets().find((p) => p.provider_type === providerType);
}

/// 生成 Pi 供应商的默认 settingsConfig
export function generatePiSettingsConfig(
  providerType: string,
  apiKey?: string,
  baseUrl?: string,
): Record<string, unknown> {
  const config: Record<string, unknown> = {
    providerType,
  };

  if (apiKey) {
    config.apiKey = apiKey;
  }

  if (baseUrl) {
    config.baseUrl = baseUrl;
  }

  // 添加环境变量映射（用于用量统计等）
  const envVars: Record<string, string> = {};
  if (apiKey) {
    switch (providerType) {
      case "anthropic":
        envVars.ANTHROPIC_API_KEY = apiKey;
        if (baseUrl) envVars.ANTHROPIC_BASE_URL = baseUrl;
        break;
      case "openai":
        envVars.OPENAI_API_KEY = apiKey;
        if (baseUrl) envVars.OPENAI_BASE_URL = baseUrl;
        break;
      case "google":
        envVars.GEMINI_API_KEY = apiKey;
        if (baseUrl) envVars.GOOGLE_GEMINI_BASE_URL = baseUrl;
        break;
      case "deepseek":
        envVars.DEEPSEEK_API_KEY = apiKey;
        break;
      case "openrouter":
        envVars.OPENROUTER_API_KEY = apiKey;
        break;
      case "groq":
        envVars.GROQ_API_KEY = apiKey;
        break;
      case "mistral":
        envVars.MISTRAL_API_KEY = apiKey;
        break;
      case "xai":
        envVars.XAI_API_KEY = apiKey;
        break;
      case "together":
        envVars.TOGETHER_API_KEY = apiKey;
        break;
      case "fireworks":
        envVars.FIREWORKS_API_KEY = apiKey;
        break;
      default:
        envVars.CUSTOM_API_KEY = apiKey;
    }
  }

  if (Object.keys(envVars).length > 0) {
    config.env = envVars;
  }

  return config;
}
