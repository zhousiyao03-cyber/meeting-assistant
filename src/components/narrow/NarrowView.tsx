import { useState, useEffect } from "react";
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
  const { transcripts, summary, advices } = events;
  const [micDevice, setMicDevice] = useState("");
  const [captureDevice, setCaptureDevice] = useState("");
  const [activeTemplateName] = useState("项目同步会");

  useEffect(() => {
    getConfig()
      .then((cfg) => {
        setMicDevice(cfg.audio.mic_device);
        setCaptureDevice(cfg.audio.capture_device);
      })
      .catch(console.error);
  }, []);

  const handleStart = () => {
    if (!micDevice || !captureDevice) {
      onSettings();
      return;
    }
    recording.start(micDevice, captureDevice);
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

  return (
    <div className="flex flex-col h-screen bg-[var(--bg-primary)]">
      <ControlBar
        templateName={activeTemplateName}
        formattedTime={recording.formattedTime}
        status={recording.status}
        onStart={handleStart}
        onPause={recording.pause}
        onResume={recording.resume}
        onStop={() => recording.stop(summary, advices, activeTemplateName)}
        onSettings={onSettings}
        onDocuments={handleDocuments}
        onFullView={onFullView}
      />
      <TranscriptMini transcripts={transcripts} />
      <SummaryPanel summary={summary} />
      <AdvicePanel advices={advices} />
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
