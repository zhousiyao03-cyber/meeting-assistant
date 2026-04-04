import type { AppConfig } from "../../lib/types";

interface LLMSettingsProps {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}

export function LLMSettings({ config, onChange }: LLMSettingsProps) {
  const updateLlm = (partial: Partial<AppConfig["llm"]>) => {
    onChange({ ...config, llm: { ...config.llm, ...partial } });
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-semibold mb-1">AI / LLM Settings</h2>
        <p className="text-sm text-[var(--text-muted)]">
          Configure the language model for real-time meeting analysis and
          suggestion generation.
        </p>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div className="bg-[var(--bg-card)] rounded-lg p-4">
          <label className="text-sm font-medium block mb-2">
            Model Provider
          </label>
          <select
            value={config.llm.base_url}
            onChange={(e) => updateLlm({ base_url: e.target.value })}
            className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-3 py-2 text-sm"
          >
            <option value="https://llmgate.io/v1">OpenAI (LLMGate)</option>
          </select>
          <p className="text-xs text-[var(--text-muted)] mt-1">
            支持 OpenAI 兼容 API 格式，默认走本地 Ollama (localhost:11434)
          </p>
        </div>

        <div className="bg-[var(--bg-card)] rounded-lg p-4">
          <label className="text-sm font-medium block mb-2">API Key</label>
          <input
            type="password"
            value={config.llm.api_key}
            onChange={(e) => updateLlm({ api_key: e.target.value })}
            placeholder="sk-ant-..."
            className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-3 py-2 text-sm"
          />
          <p className="text-xs text-[var(--text-muted)] mt-1">
            Your API key is stored locally and never sent to our servers.
          </p>
        </div>

        <div className="bg-[var(--bg-card)] rounded-lg p-4">
          <label className="text-sm font-medium block mb-2">Model</label>
          <input
            value={config.llm.model}
            onChange={(e) => updateLlm({ model: e.target.value })}
            placeholder="llama3.2"
            className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-3 py-2 text-sm"
          />
        </div>

        <div className="bg-[var(--bg-card)] rounded-lg p-4">
          <label className="text-sm font-medium block mb-2">
            Language Preference
          </label>
          <select
            value={config.language_preference}
            onChange={(e) =>
              onChange({ ...config, language_preference: e.target.value })
            }
            className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-3 py-2 text-sm"
          >
            <option value="auto">Auto (中英混合)</option>
            <option value="zh">中文</option>
            <option value="en">English</option>
          </select>
        </div>
      </div>
    </div>
  );
}
