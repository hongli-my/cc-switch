// Pi Agent 提供商预设配置
//
// 为 Pi Agent 提供常用的提供商预设（OpenAI 兼容的各类 API）。
// 注意：字段命名保持 snake_case（provider_type / base_url），与
// src/components/providers/forms/PiProviderForm.tsx 的访问方式一致。

export interface PiProviderPreset {
  id: string;
  name: string;
  description: string;
  /** 对应 auth.json 中的 provider key */
  provider_type: string;
  base_url?: string;
}

/// 获取所有 Pi 提供商预设
export function getPiProviderPresets(): PiProviderPreset[] {
  return [
    {
      id: "pi-anthropic",
      name: "Anthropic (Claude)",
      description: "使用 Anthropic API (Claude 模型)",
      provider_type: "anthropic",
    },
    {
      id: "pi-openai",
      name: "OpenAI",
      description: "使用 OpenAI API (GPT 模型)",
      provider_type: "openai",
    },
    {
      id: "pi-gemini",
      name: "Google Gemini",
      description: "使用 Google Gemini API",
      provider_type: "google",
    },
    {
      id: "pi-deepseek",
      name: "DeepSeek",
      description: "使用 DeepSeek API",
      provider_type: "deepseek",
    },
    {
      id: "pi-openrouter",
      name: "OpenRouter",
      description: "使用 OpenRouter API (聚合多个模型)",
      provider_type: "openrouter",
      base_url: "https://openrouter.ai/api/v1",
    },
    {
      id: "pi-groq",
      name: "Groq",
      description: "使用 Groq API (高速推理)",
      provider_type: "groq",
    },
    {
      id: "pi-mistral",
      name: "Mistral AI",
      description: "使用 Mistral AI API",
      provider_type: "mistral",
    },
    {
      id: "pi-xai",
      name: "xAI (Grok)",
      description: "使用 xAI Grok API",
      provider_type: "xai",
    },
    {
      id: "pi-together",
      name: "Together AI",
      description: "使用 Together AI API",
      provider_type: "together",
    },
    {
      id: "pi-fireworks",
      name: "Fireworks AI",
      description: "使用 Fireworks AI API",
      provider_type: "fireworks",
    },
    {
      id: "pi-custom",
      name: "Custom (OpenAI Compatible)",
      description: "自定义 OpenAI 兼容 API",
      provider_type: "custom",
      base_url: "",
    },
  ];
}

/// 根据 provider_type 获取预设配置
export function getPresetByProviderType(
  providerType: string,
): PiProviderPreset | undefined {
  return getPiProviderPresets().find((p) => p.provider_type === providerType);
}
