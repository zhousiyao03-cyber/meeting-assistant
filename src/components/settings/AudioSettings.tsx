import { useState, useEffect } from "react";
import type { AudioDevice, AppConfig } from "../../lib/types";
import { listAudioDevices } from "../../lib/tauri";

interface AudioSettingsProps {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}

export function AudioSettings({ config, onChange }: AudioSettingsProps) {
  const [devices, setDevices] = useState<AudioDevice[]>([]);

  useEffect(() => {
    listAudioDevices().then(setDevices).catch(console.error);
  }, []);

  const update = (partial: Partial<AppConfig["audio"]>) => {
    onChange({ ...config, audio: { ...config.audio, ...partial } });
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-semibold mb-1">Audio Settings</h2>
        <p className="text-sm text-[var(--text-muted)]">
          Configure audio capture for real-time speech recognition during
          meetings.
        </p>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div className="bg-[var(--bg-card)] rounded-lg p-4">
          <label className="text-sm font-medium block mb-2">
            System Audio Source
          </label>
          <select
            value={config.audio.capture_device}
            onChange={(e) => update({ capture_device: e.target.value })}
            className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-3 py-2 text-sm"
          >
            <option value="">Select device...</option>
            {devices.map((d) => (
              <option key={d.id} value={d.id}>
                {d.name}
              </option>
            ))}
          </select>
          <p className="text-xs text-[var(--text-muted)] mt-1">
            选择黑洞等虚拟音频设备 (BlackHole) 来捕获会议音频流
          </p>
        </div>

        <div className="bg-[var(--bg-card)] rounded-lg p-4">
          <label className="text-sm font-medium block mb-2">
            Microphone Input
          </label>
          <select
            value={config.audio.mic_device}
            onChange={(e) => update({ mic_device: e.target.value })}
            className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-3 py-2 text-sm"
          >
            <option value="">Select device...</option>
            {devices.map((d) => (
              <option key={d.id} value={d.id}>
                {d.name}
              </option>
            ))}
          </select>
          <p className="text-xs text-[var(--text-muted)] mt-1">
            选择你的麦克风以捕获自己的声音
          </p>
        </div>

        <div className="bg-[var(--bg-card)] rounded-lg p-4">
          <label className="text-sm font-medium block mb-2">
            Audio Quality
          </label>
          <div className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-3 py-2 text-sm text-[var(--text-muted)]">
            16kHz Mono (Whisper)
          </div>
        </div>

        <div className="bg-[var(--bg-card)] rounded-lg p-4">
          <label className="text-sm font-medium block mb-2">
            Noise Reduction
          </label>
          <div className="flex items-center justify-between">
            <span className="text-sm text-[var(--text-secondary)]">
              Apply real-time noise reduction
            </span>
            <button
              onClick={() =>
                update({ noise_reduction: !config.audio.noise_reduction })
              }
              className={`w-10 h-6 rounded-full transition-colors ${
                config.audio.noise_reduction
                  ? "bg-[var(--accent-purple)]"
                  : "bg-[var(--border)]"
              }`}
            >
              <div
                className={`w-4 h-4 rounded-full bg-white mx-1 transition-transform ${
                  config.audio.noise_reduction ? "translate-x-4" : ""
                }`}
              />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
