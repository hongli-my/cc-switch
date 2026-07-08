import React from "react";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { AppId } from "@/lib/api/types";
import { APP_IDS, APP_ICON_MAP } from "@/config/appConfig";
import {
  OpencodeProfileDropdown,
} from "@/components/common/OpencodeProfileDropdown";
import type { OpenCodeProfile } from "@/lib/api/skills";

interface AppToggleGroupProps {
  apps: Partial<Record<AppId, boolean>>;
  onToggle: (app: AppId, enabled: boolean) => void;
  appIds?: AppId[];
  /** opencode 当前选中的 profile 名（"" = default/整体） */
  opencodeProfiles?: string[];
  /** 提供此回调时，opencode 图标切换为 profile 多选下拉模式 */
  onOpencodeProfilesChange?: (profiles: string[]) => void;
  /** opencode 可选 profile 列表（由上层统一查好后传入，避免每行各自发请求） */
  opencodeAvailableProfiles?: OpenCodeProfile[];
}

export const AppToggleGroup: React.FC<AppToggleGroupProps> = ({
  apps,
  onToggle,
  appIds = APP_IDS,
  opencodeProfiles,
  onOpencodeProfilesChange,
  opencodeAvailableProfiles,
}) => {
  return (
    <div className="flex items-center gap-1.5 flex-shrink-0">
      {appIds.map((app) => {
        const { label, icon, activeClass } = APP_ICON_MAP[app];
        const enabled = apps[app];

        // opencode：当上层提供 onOpencodeProfilesChange 时，启用 profile 多选下拉，
        // 而不是普通的启用/禁用 toggle。
        if (app === "opencode" && onOpencodeProfilesChange) {
          return (
            <OpencodeProfileDropdown
              key={app}
              selected={opencodeProfiles ?? []}
              profiles={opencodeAvailableProfiles ?? []}
              onChange={onOpencodeProfilesChange}
              triggerIcon={icon}
              activeClass={activeClass}
            />
          );
        }

        // 普通 toggle：其余 app，以及 opencode 的向后兼容场景
        // （ImportSkillsDialog / UnifiedMcpPanel 仍使用简单 toggle）
        return (
          <Tooltip key={app}>
            <TooltipTrigger asChild>
              <button
                type="button"
                onClick={() => onToggle(app, !enabled)}
                className={`w-7 h-7 rounded-lg flex items-center justify-center transition-all ${
                  enabled ? activeClass : "opacity-35 hover:opacity-70"
                }`}
              >
                {icon}
              </button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
              <p>
                {label}
                {enabled ? " ✓" : ""}
              </p>
            </TooltipContent>
          </Tooltip>
        );
      })}
    </div>
  );
};
