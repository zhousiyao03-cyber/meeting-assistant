import { useState, useEffect } from "react";
import { ControlBar } from "./ControlBar";
import { TranscriptMini } from "./TranscriptMini";
import { SummaryPanel } from "./SummaryPanel";
import { AdvicePanel } from "./AdvicePanel";
import { useTauriEvents } from "../../hooks/useTauriEvents";
import { useRecording } from "../../hooks/useRecording";
import { getConfig } from "../../lib/tauri";

interface NarrowViewProps {
  onSettings: () => void;
  onFullView: () => void;
}

export function NarrowView({ onSettings, onFullView: _onFullView }: NarrowViewProps) {
  const { transcripts, summary, advices } = useTauriEvents();
  const recording = useRecording();
  const [micDevice, setMicDevice] = useState("");
  const [captureDevice, setCaptureDevice] = useState("");

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

  return (
    <div className="flex flex-col h-screen bg-[var(--bg-primary)]">
      <ControlBar
        templateName="技术评审会"
        formattedTime={recording.formattedTime}
        status={recording.status}
        onStart={handleStart}
        onPause={recording.pause}
        onResume={recording.resume}
        onStop={recording.stop}
        onSettings={onSettings}
        onDocuments={() => {}}
      />
      <TranscriptMini transcripts={transcripts} />
      <SummaryPanel summary={summary} />
      <AdvicePanel advices={advices} />
    </div>
  );
}
