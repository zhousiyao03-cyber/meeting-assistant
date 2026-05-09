import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import type { AppConfig, UserPlan } from "../../lib/types";
import { getConfig, saveConfig as saveConfigApi, getUserPlan } from "../../lib/tauri";
import { AudioSettings } from "./AudioSettings";
import { LLMSettings } from "./LLMSettings";
import { ProfileSettings } from "./ProfileSettings";
import { LanguageSettings } from "./LanguageSettings";
import { BYOKeySettings } from "./BYOKeySettings";
import { LicenseInput } from "../license/LicenseInput";

type SettingsTab =
  | "audio"
  | "llm"
  | "language"
  | "license"
  | "byo"
  | "profiles"
  | "hotkeys"
  | "about";

interface SettingsViewProps {
  onBack: () => void;
}

export function SettingsView({ onBack }: SettingsViewProps) {
  const { t } = useTranslation();
  const [tab, setTab] = useState<SettingsTab>("audio");
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [plan, setPlan] = useState<UserPlan | null>(null);

  useEffect(() => {
    getConfig().then(setConfig).catch(console.error);
    getUserPlan().then(setPlan).catch(console.error);
  }, []);

  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleConfigChange = useCallback((newConfig: AppConfig) => {
    setConfig(newConfig);
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current);
    }
    saveTimeoutRef.current = setTimeout(() => {
      saveConfigApi(newConfig).catch(console.error);
    }, 500);
  }, []);

  useEffect(() => {
    return () => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
    };
  }, []);

  const tabs: { id: SettingsTab; labelKey: string; icon: string }[] = [
    { id: "audio", labelKey: "settings.tabs.audio", icon: "🎙" },
    { id: "llm", labelKey: "settings.tabs.llm", icon: "🤖" },
    { id: "language", labelKey: "settings.tabs.language", icon: "🌐" },
    { id: "license", labelKey: "settings.tabs.license", icon: "🔑" },
    { id: "byo", labelKey: "settings.tabs.byo", icon: "🛠" },
    { id: "profiles", labelKey: "settings.tabs.profile", icon: "📋" },
  ];

  return (
    <div className="flex h-screen bg-[var(--bg-primary)]">
      <div className="w-56 border-r border-[var(--border)] bg-[var(--bg-secondary)] p-4">
        <div className="mb-6">
          <button
            onClick={onBack}
            className="text-sm text-[var(--text-muted)] hover:text-[var(--text-primary)]"
          >
            ← Back
          </button>
          <h2 className="text-xs text-[var(--text-muted)] mt-4">
            {t("settings.title")}
          </h2>
        </div>
        <div className="space-y-1">
          {tabs.map((tabInfo) => (
            <button
              key={tabInfo.id}
              onClick={() => setTab(tabInfo.id)}
              className={`w-full text-left px-3 py-2 rounded text-sm flex items-center gap-2 ${
                tab === tabInfo.id
                  ? "bg-[var(--accent-purple)]/20 text-[var(--accent-purple)]"
                  : "text-[var(--text-secondary)] hover:bg-[var(--bg-card)]"
              }`}
            >
              <span>{tabInfo.icon}</span>
              {t(tabInfo.labelKey)}
            </button>
          ))}
        </div>
      </div>

      <div className="flex-1 p-8 overflow-y-auto">
        {config && tab === "audio" && (
          <AudioSettings config={config} onChange={handleConfigChange} />
        )}
        {config && tab === "llm" && (
          <LLMSettings config={config} onChange={handleConfigChange} />
        )}
        {tab === "language" && <LanguageSettings />}
        {tab === "license" && plan && (
          <LicenseInput currentPlan={plan} onUpdated={setPlan} />
        )}
        {tab === "byo" && <BYOKeySettings />}
        {tab === "profiles" && <ProfileSettings />}
      </div>
    </div>
  );
}
