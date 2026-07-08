import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Checkbox } from "@/components/ui/checkbox";
import { cn } from "@/lib/utils";
import type { OpenCodeProfile } from "@/lib/api/skills";

export interface OpencodeProfileDropdownProps {
  /** 当前选中的 profile 名（"" = default/整体） */
  selected: string[];
  /** 可选项（来自 useOpencodeProfiles） */
  profiles: OpenCodeProfile[];
  /** 用户改动后回调，传新的 selected 数组（多选，不互斥） */
  onChange: (profiles: string[]) => void;
  /** opencode 图标（由 AppToggleGroup 传入，保持视觉一致） */
  triggerIcon: React.ReactNode;
  /** 激活样式类（由 AppToggleGroup 传入） */
  activeClass: string;
}

/** default profile 的 name 约定为空串 */
const DEFAULT_PROFILE_NAME = "";

export const OpencodeProfileDropdown: React.FC<
  OpencodeProfileDropdownProps
> = ({ selected, profiles, onChange, triggerIcon, activeClass }) => {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);

  const selectedSet = new Set(selected);
  const count = selected.length;
  const isActive = count > 0;

  // 命名 profile（排除 default），按名字排序，保证下拉顺序稳定
  const namedProfiles = profiles
    .filter((p) => p.name !== DEFAULT_PROFILE_NAME && p.name)
    .sort((a, b) => a.name.localeCompare(b.name));

  const toggle = (name: string) => {
    const next = selectedSet.has(name)
      ? selected.filter((n) => n !== name)
      : [...selected, name];
    onChange(next);
  };

  const overallLabel = t("skills.opencode.overall", {
    defaultValue: "整体",
  });
  const defaultTag = t("skills.opencode.defaultTag", {
    defaultValue: "default",
  });
  const headerLabel = t("skills.opencode.profilesHeader", {
    defaultValue: "OpenCode Profiles",
  });

  const renderRow = (
    key: string,
    display: React.ReactNode,
    tag?: string,
  ) => {
    const name = key === "__default__" ? DEFAULT_PROFILE_NAME : key;
    const isSelected = selectedSet.has(name);
    return (
      <div
        key={key}
        role="checkbox"
        aria-checked={isSelected}
        tabIndex={0}
        onClick={() => toggle(name)}
        onKeyDown={(e) => {
          if (e.key === " " || e.key === "Enter") {
            e.preventDefault();
            toggle(name);
          }
        }}
        className="flex items-center gap-2 w-full px-2 py-1.5 rounded-md hover:bg-accent hover:text-accent-foreground cursor-pointer text-sm select-none transition-colors"
      >
        {/* pointer-events-none + tabIndex=-1：把点击/键盘交给外层 div，
            避免与 Radix Checkbox 内部 button 双重触发；Checkbox 仅做视觉反映 */}
        <Checkbox
          checked={isSelected}
          tabIndex={-1}
          className="pointer-events-none"
        />
        <span className="flex-1 truncate text-foreground/90">{display}</span>
        {tag && (
          <span className="text-[10px] text-muted-foreground/70 shrink-0">
            {tag}
          </span>
        )}
      </div>
    );
  };

  // 触发器 hover 提示：OpenCode · N
  const titleText = `OpenCode${count > 0 ? ` · ${count}` : ""}`;

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button
          type="button"
          title={titleText}
          className={cn(
            "relative w-7 h-7 rounded-lg flex items-center justify-center transition-all",
            isActive ? activeClass : "opacity-35 hover:opacity-70",
          )}
        >
          {triggerIcon}
          {count > 0 && (
            <span
              className="absolute -top-1 -right-1 min-w-[14px] h-[14px] px-1 rounded-full bg-indigo-500 text-white text-[9px] font-semibold leading-none flex items-center justify-center ring-2 ring-background"
              aria-hidden="true"
            >
              {count}
            </span>
          )}
        </button>
      </PopoverTrigger>
      <PopoverContent
        align="end"
        sideOffset={6}
        collisionPadding={8}
        avoidCollisions
        className="z-[100] w-52 p-1 border-border-default"
      >
        <div className="px-2 pt-1.5 pb-1 text-[10px] font-medium uppercase tracking-wide text-muted-foreground/60">
          {headerLabel}
        </div>
        {/* 第一行永远是「整体」(default) */}
        {renderRow(
          "__default__",
          <span>
            {overallLabel}{" "}
            <span className="text-muted-foreground/60">({defaultTag})</span>
          </span>,
        )}
        {namedProfiles.length > 0 && (
          <div
            className="my-1 h-px bg-border-default/60"
            role="separator"
          />
        )}
        {namedProfiles.map((p) => renderRow(p.name, <span>{p.name}</span>))}
      </PopoverContent>
    </Popover>
  );
};
