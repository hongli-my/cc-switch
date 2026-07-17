import { useTranslation } from "react-i18next";
import { Controller, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { toast } from "sonner";
import { getPiProviderPresets, type PiProviderPreset } from "@/config/piProviderPresets";

interface PiProviderFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
  editData?: any;
}

const formSchema = z.object({
  name: z.string().min(1, "名称不能为空"),
  providerType: z.string().min(1, "请选择提供商类型"),
  apiKey: z.string().optional(),
  baseUrl: z.string().optional(),
  notes: z.string().optional(),
});

type FormValues = z.infer<typeof formSchema>;

export function PiProviderForm({
  open,
  onOpenChange,
  onSuccess,
  editData,
}: PiProviderFormProps) {
  const { t } = useTranslation();
  const [presets, setPresets] = useState<PiProviderPreset[]>([]);
  const [selectedPreset, setSelectedPreset] = useState<string>("");

  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      name: "",
      providerType: "",
      apiKey: "",
      baseUrl: "",
      notes: "",
    },
  });

  useEffect(() => {
    // 加载预设
    const loadedPresets = getPiProviderPresets();
    setPresets(loadedPresets);
  }, []);

  useEffect(() => {
    if (editData) {
      form.reset({
        name: editData.name || "",
        providerType: editData.settingsConfig?.providerType || "",
        apiKey: editData.settingsConfig?.apiKey || "",
        baseUrl: editData.settingsConfig?.baseUrl || "",
        notes: editData.notes || "",
      });
      setSelectedPreset(editData.settingsConfig?.providerType || "");
    }
  }, [editData, form]);

  const handlePresetChange = (presetId: string) => {
    setSelectedPreset(presetId);
    const preset = presets.find((p) => p.id === presetId);
    if (preset) {
      form.setValue("providerType", preset.provider_type);
      form.setValue("name", preset.name);
      if (preset.base_url) {
        form.setValue("baseUrl", preset.base_url);
      }
    }
  };

  const onSubmit = async (values: FormValues) => {
    try {
      const settingsConfig: any = {
        providerType: values.providerType,
      };

      if (values.apiKey) {
        settingsConfig.apiKey = values.apiKey;
      }

      if (values.baseUrl) {
        settingsConfig.baseUrl = values.baseUrl;
      }

      // 添加环境变量（用于用量统计）
      const env: any = {};
      if (values.apiKey) {
        switch (values.providerType) {
          case "anthropic":
            env.ANTHROPIC_API_KEY = values.apiKey;
            break;
          case "openai":
            env.OPENAI_API_KEY = values.apiKey;
            break;
          case "google":
            env.GEMINI_API_KEY = values.apiKey;
            break;
          case "deepseek":
            env.DEEPSEEK_API_KEY = values.apiKey;
            break;
          case "openrouter":
            env.OPENROUTER_API_KEY = values.apiKey;
            break;
          default:
            env.CUSTOM_API_KEY = values.apiKey;
        }
      }

      if (values.baseUrl) {
        switch (values.providerType) {
          case "anthropic":
            env.ANTHROPIC_BASE_URL = values.baseUrl;
            break;
          case "openai":
            env.OPENAI_BASE_URL = values.baseUrl;
            break;
          default:
            break;
        }
      }

      if (Object.keys(env).length > 0) {
        settingsConfig.env = env;
      }

      const providerData = {
        id: editData?.id || `pi-${Date.now()}`,
        name: values.name,
        settingsConfig,
        notes: values.notes,
        meta: {
          providerType: "pi",
          appType: "pi",
        },
      };

      if (editData) {
        await invoke("update_provider", {
          app: "pi",
          provider: providerData,
          originalId: editData.id,
        });
        toast.success(t("provider.updatedSuccessfully"));
      } else {
        await invoke("add_provider", {
          app: "pi",
          provider: providerData,
        });
        toast.success(t("provider.addedSuccessfully"));
      }

      onOpenChange(false);
      onSuccess?.();
    } catch (error) {
      console.error("Failed to save Pi provider:", error);
      toast.error(t("provider.saveFailed"));
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>
            {editData
              ? t("pi.editProvider", "编辑 Pi 供应商")
              : t("pi.addProvider", "添加 Pi 供应商")}
          </DialogTitle>
          <DialogDescription>
            {t(
              "pi.providerFormDescription",
              "配置 Pi Agent 的 API 供应商。Pi 支持多种 OpenAI 兼容的 API。"
            )}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
          {/* 预设选择 */}
          <div className="space-y-2">
            <Label>{t("pi.preset", "预设")}</Label>
            <Select value={selectedPreset} onValueChange={handlePresetChange}>
              <SelectTrigger>
                <SelectValue placeholder={t("pi.selectPreset", "选择预设...")} />
              </SelectTrigger>
              <SelectContent>
                {presets.map((preset) => (
                  <SelectItem key={preset.id} value={preset.id}>
                    {preset.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* 名称 */}
          <div className="space-y-2">
            <Label htmlFor="name">{t("common.name", "名称")}</Label>
            <Input
              id="name"
              {...form.register("name")}
              placeholder={t("pi.providerNamePlaceholder", "输入供应商名称")}
            />
            {form.formState.errors.name && (
              <p className="text-sm text-red-500">
                {form.formState.errors.name.message}
              </p>
            )}
          </div>

          {/* 提供商类型 */}
          <div className="space-y-2">
            <Label htmlFor="providerType">
              {t("pi.providerType", "提供商类型")}
            </Label>
            <Input
              id="providerType"
              {...form.register("providerType")}
              placeholder="anthropic, openai, gemini..."
              disabled
            />
          </div>

          {/* API Key */}
          <div className="space-y-2">
            <Label htmlFor="apiKey">{t("common.apiKey", "API Key")}</Label>
            <Input
              id="apiKey"
              type="password"
              {...form.register("apiKey")}
              placeholder={t("pi.apiKeyPlaceholder", "输入 API Key")}
            />
          </div>

          {/* Base URL */}
          <div className="space-y-2">
            <Label htmlFor="baseUrl">
              {t("common.baseUrl", "Base URL")}
            </Label>
            <Input
              id="baseUrl"
              {...form.register("baseUrl")}
              placeholder="https://api.example.com/v1"
            />
          </div>

          {/* 备注 */}
          <div className="space-y-2">
            <Label htmlFor="notes">{t("common.notes", "备注")}</Label>
            <Textarea
              id="notes"
              {...form.register("notes")}
              placeholder={t("pi.notesPlaceholder", "可选：添加备注")}
              rows={3}
            />
          </div>

          {/* 按钮 */}
          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              {t("common.cancel", "取消")}
            </Button>
            <Button type="submit">
              {editData
                ? t("common.update", "更新")
                : t("common.add", "添加")}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
