import { useState, useEffect } from "react";
import type { AppConfig } from "../../lib/types";
import { getConfig, saveConfig as saveConfigApi } from "../../lib/tauri";
import { AudioSettings } from "./AudioSettings";
import { LLMSettings } from "./LLMSettings";
import { ProfileSettings } from "./ProfileSettings";

type SettingsTab = "audio" | "llm" | "profiles" | "hotkeys" | "about";

interface SettingsViewProps {
  onBack: () => void;
}

export function SettingsView({ onBack }: SettingsViewProps) {
  const [tab, setTab] = useState<SettingsTab>("audio");
  const [config, setConfig] = useState<AppConfig | null>(null);

  useEffect(() => {
    getConfig().then(setConfig).catch(console.error);
  }, []);

  const handleConfigChange = async (newConfig: AppConfig) => {
    setConfig(newConfig);
    await saveConfigApi(newConfig);
  };

  const tabs: { id: SettingsTab; label: string; icon: string }[] = [
    { id: "audio", label: "Audio Settings", icon: "🎙" },
    { id: "llm", label: "AI / LLM", icon: "🤖" },
    { id: "profiles", label: "Meeting Profiles", icon: "📋" },
    { id: "hotkeys", label: "Hotkeys", icon: "⌨" },
    { id: "about", label: "About", icon: "○" },
  ];

  return (
    <div className="flex h-screen bg-[var(--bg-primary)]">
      {/* Sidebar nav */}
      <div className="w-56 border-r border-[var(--border)] bg-[var(--bg-secondary)] p-4">
        <div className="mb-6">
          <button
            onClick={onBack}
            className="text-sm text-[var(--text-muted)] hover:text-[var(--text-primary)]"
          >
            ← Meeting Copilot
          </button>
          <h2 className="text-xs text-[var(--text-muted)] mt-4">Settings</h2>
        </div>
        <div className="space-y-1">
          {tabs.map((t) => (
            <button
              key={t.id}
              onClick={() => setTab(t.id)}
              className={`w-full text-left px-3 py-2 rounded text-sm flex items-center gap-2 ${
                tab === t.id
                  ? "bg-[var(--accent-purple)]/20 text-[var(--accent-purple)]"
                  : "text-[var(--text-secondary)] hover:bg-[var(--bg-card)]"
              }`}
            >
              <span>{t.icon}</span>
              {t.label}
            </button>
          ))}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 p-8 overflow-y-auto">
        {config && tab === "audio" && (
          <AudioSettings config={config} onChange={handleConfigChange} />
        )}
        {config && tab === "llm" && (
          <LLMSettings config={config} onChange={handleConfigChange} />
        )}
        {tab === "profiles" && <ProfileSettings />}
        {tab === "hotkeys" && (
          <div>
            <h2 className="text-xl font-semibold">Hotkeys</h2>
            <p className="text-sm text-[var(--text-muted)] mt-2">
              Coming soon.
            </p>
          </div>
        )}
        {tab === "about" && (
          <div>
            <h2 className="text-xl font-semibold">About</h2>
            <p className="text-sm text-[var(--text-secondary)] mt-2">
              Meeting Copilot v0.1.0
            </p>
            <p className="text-sm text-[var(--text-muted)] mt-1">
              本地优先的 AI 会议助手。音频处理和语音识别全部在本地完成。
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
