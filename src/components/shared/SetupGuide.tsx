import { useState, useEffect } from "react";
import { downloadWhisperModel, onModelDownloadProgress } from "../../lib/tauri";

type SetupStep = "blackhole" | "model" | "config" | "done";

interface SetupGuideProps {
  onComplete: () => void;
}

export function SetupGuide({ onComplete }: SetupGuideProps) {
  const [step, setStep] = useState<SetupStep>("blackhole");
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloading, setDownloading] = useState(false);

  useEffect(() => {
    const unsub = onModelDownloadProgress((p) => {
      if (p.total > 0) {
        setDownloadProgress(Math.round((p.downloaded / p.total) * 100));
      }
    });
    return () => { unsub.then(u => u()); };
  }, []);

  const handleDownloadModel = async () => {
    setDownloading(true);
    try {
      await downloadWhisperModel();
      setStep("config");
    } catch (e) {
      console.error("Download failed:", e);
    }
    setDownloading(false);
  };

  return (
    <div className="flex flex-col items-center justify-center h-screen bg-[var(--bg-primary)] p-8">
      <h1 className="text-2xl font-bold mb-8">Welcome to Meeting Copilot</h1>

      {step === "blackhole" && (
        <div className="max-w-md text-center space-y-4">
          <h2 className="text-lg font-medium">Step 1: 安装 BlackHole 虚拟音频</h2>
          <p className="text-sm text-[var(--text-secondary)]">
            BlackHole 是免费的 macOS 虚拟音频驱动，用于捕获远程会议的音频。
          </p>
          <ol className="text-sm text-[var(--text-secondary)] text-left space-y-2">
            <li>1. 下载并安装 BlackHole 2ch</li>
            <li>2. 打开 macOS &quot;Audio MIDI Setup&quot;</li>
            <li>3. 创建 &quot;多输出设备&quot;，包含你的耳机 + BlackHole 2ch</li>
            <li>4. 将系统音频输出设为该多输出设备</li>
          </ol>
          <div className="flex gap-3 justify-center mt-4">
            <button
              onClick={() => setStep("model")}
              className="px-4 py-2 rounded bg-[var(--accent-purple)] text-white text-sm"
            >
              已完成，继续 →
            </button>
            <button
              onClick={() => setStep("model")}
              className="px-4 py-2 rounded bg-[var(--bg-card)] text-[var(--text-secondary)] text-sm"
            >
              稍后设置
            </button>
          </div>
        </div>
      )}

      {step === "model" && (
        <div className="max-w-md text-center space-y-4">
          <h2 className="text-lg font-medium">Step 2: 下载语音识别模型</h2>
          <p className="text-sm text-[var(--text-secondary)]">
            Whisper medium 模型 (~1.5GB)，用于本地语音转文字。
          </p>
          {downloading ? (
            <div className="w-full bg-[var(--bg-card)] rounded-full h-3">
              <div
                className="bg-[var(--accent-purple)] h-3 rounded-full transition-all"
                style={{ width: `${downloadProgress}%` }}
              />
            </div>
          ) : (
            <button
              onClick={handleDownloadModel}
              className="px-4 py-2 rounded bg-[var(--accent-purple)] text-white text-sm"
            >
              开始下载
            </button>
          )}
          <p className="text-xs text-[var(--text-muted)]">
            {downloading ? `${downloadProgress}%` : "下载后模型将保存在本地"}
          </p>
        </div>
      )}

      {step === "config" && (
        <div className="max-w-md text-center space-y-4">
          <h2 className="text-lg font-medium">Step 3: 配置 LLM</h2>
          <p className="text-sm text-[var(--text-secondary)]">
            请在设置中配置 LLM API 地址和密钥，用于会议总结和发言建议。
          </p>
          <button
            onClick={onComplete}
            className="px-4 py-2 rounded bg-[var(--accent-purple)] text-white text-sm"
          >
            开始使用 →
          </button>
        </div>
      )}
    </div>
  );
}
