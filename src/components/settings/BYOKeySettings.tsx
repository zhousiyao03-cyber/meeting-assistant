import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { getConfig, saveConfig } from "../../lib/tauri";
import type { AppConfig } from "../../lib/types";

export function BYOKeySettings() {
  const { t } = useTranslation();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [openaiKey, setOpenaiKey] = useState("");
  const [anthropicKey, setAnthropicKey] = useState("");
  const [active, setActive] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    void getConfig().then((c) => {
      setConfig(c);
      setOpenaiKey(c.byo?.openai_api_key ?? "");
      setAnthropicKey(c.byo?.anthropic_api_key ?? "");
      setActive(c.byo?.active ?? false);
    });
  }, []);

  async function handleSave() {
    if (!config) return;
    await saveConfig({
      ...config,
      byo: {
        active,
        openai_api_key: openaiKey,
        anthropic_api_key: anthropicKey,
        anthropic_model: config.byo?.anthropic_model ?? "claude-sonnet-4-6",
      },
    });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  }

  return (
    <div className="space-y-4">
      <h3 className="font-medium">{t("settings.byo.title")}</h3>
      <p className="text-xs text-gray-400">{t("settings.byo.desc")}</p>

      <div>
        <label className="text-sm block mb-1">{t("settings.byo.openaiLabel")}</label>
        <input
          type="password"
          value={openaiKey}
          onChange={(e) => setOpenaiKey(e.target.value)}
          className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-2 py-1 text-sm font-mono"
          placeholder="sk-..."
        />
      </div>

      <div>
        <label className="text-sm block mb-1">{t("settings.byo.anthropicLabel")}</label>
        <input
          type="password"
          value={anthropicKey}
          onChange={(e) => setAnthropicKey(e.target.value)}
          className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-2 py-1 text-sm font-mono"
          placeholder="sk-ant-..."
        />
      </div>

      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={active}
          onChange={(e) => setActive(e.target.checked)}
        />
        {t("settings.byo.activeLabel")}
      </label>

      <div className="flex items-center gap-3">
        <button
          onClick={() => void handleSave()}
          className="px-4 py-2 bg-[var(--accent-purple,#7c3aed)] text-white rounded text-sm"
        >
          {t("settings.byo.saveBtn")}
        </button>
        {saved && (
          <span className="text-xs text-green-400">
            {t("settings.byo.saved")}
          </span>
        )}
      </div>
    </div>
  );
}
