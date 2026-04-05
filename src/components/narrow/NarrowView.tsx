import { useState, useEffect, useRef } from "react";
import { ControlBar } from "./ControlBar";
import { TranscriptMini } from "./TranscriptMini";
import { SummaryPanel } from "./SummaryPanel";
import { AdvicePanel } from "./AdvicePanel";
import { getConfig, loadReferenceDoc } from "../../lib/tauri";
import { open } from "@tauri-apps/plugin-dialog";

interface NarrowViewProps {
  onSettings: () => void;
  onFullView: () => void;
  onHistory: () => void;
  events: ReturnType<typeof import("../../hooks/useTauriEvents").useTauriEvents>;
  recording: ReturnType<typeof import("../../hooks/useRecording").useRecording>;
}

export function NarrowView({ onSettings, onFullView, onHistory, events, recording }: NarrowViewProps) {
  const { transcripts, summary, advices, clearAll } = events;
  const [micDevice, setMicDevice] = useState("");
  const [captureDevice, setCaptureDevice] = useState("");
  const [activeTemplateName] = useState("项目同步会");
  const [meetingEnded, setMeetingEnded] = useState(false);
  const endTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    getConfig()
      .then((cfg) => {
        setMicDevice(cfg.audio.mic_device);
        setCaptureDevice(cfg.audio.capture_device);
      })
      .catch(console.error);
    return () => {
      if (endTimerRef.current) clearTimeout(endTimerRef.current);
    };
  }, []);

  const handleStart = () => {
    if (!micDevice || !captureDevice) {
      onSettings();
      return;
    }
    setMeetingEnded(false);
    clearAll();
    recording.start(micDevice, captureDevice);
  };

  const handleStop = async () => {
    await recording.stop(summary, advices, activeTemplateName);
    setMeetingEnded(true);
    // 3 秒后自动回到 idle 新建会议界面
    endTimerRef.current = setTimeout(() => {
      setMeetingEnded(false);
      clearAll();
    }, 3000);
  };

  const handleDocuments = async () => {
    const file = await open({
      multiple: false,
      filters: [{ name: "Documents", extensions: ["md", "txt", "text", "pdf"] }],
    });
    if (file) {
      try {
        const filename = await loadReferenceDoc(file);
        console.log("Loaded reference doc:", filename);
      } catch (e) {
        console.error("Failed to load document:", e);
      }
    }
  };

  const isActive = recording.status === "recording" || recording.status === "paused";

  return (
    <div className="flex flex-col h-screen bg-[var(--bg-primary)]">
      <ControlBar
        templateName={activeTemplateName}
        formattedTime={recording.formattedTime}
        status={recording.status}
        onStart={handleStart}
        onPause={recording.pause}
        onResume={recording.resume}
        onStop={handleStop}
        onSettings={onSettings}
        onDocuments={handleDocuments}
        onFullView={onFullView}
      />
      {isActive ? (
        <>
          <TranscriptMini transcripts={transcripts} />
          <SummaryPanel summary={summary} />
          <AdvicePanel advices={advices} />
        </>
      ) : meetingEnded ? (
        <div className="flex-1 flex flex-col items-center justify-center gap-3 px-6">
          <span className="text-2xl">✓</span>
          <p className="text-sm font-medium">会议已结束</p>
          <p className="text-xs text-[var(--text-muted)]">会议记录已自动保存</p>
          <button
            onClick={() => { setMeetingEnded(false); clearAll(); }}
            className="mt-2 px-4 py-2 rounded-lg bg-[var(--accent-purple)] text-white text-sm hover:opacity-90 transition-opacity"
          >
            + 开始新会议
          </button>
        </div>
      ) : (
        <div className="flex-1 flex flex-col items-center justify-center gap-4 px-6">
          <button
            onClick={handleStart}
            className="w-full py-3 rounded-lg bg-[var(--accent-purple)] text-white text-sm font-medium hover:opacity-90 transition-opacity"
          >
            + 新建会议
          </button>
          <p className="text-xs text-[var(--text-muted)] text-center">
            点击开始录制并获取实时会议辅助
          </p>
        </div>
      )}
      <div className="border-t border-[var(--border)] px-4 py-2 flex justify-end">
        <button
          onClick={onHistory}
          className="text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
        >
          历史记录
        </button>
      </div>
    </div>
  );
}
