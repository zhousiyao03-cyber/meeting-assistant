import { ControlBar } from "./ControlBar";
import { SummaryPanel } from "./SummaryPanel";
import { AdvicePanel } from "./AdvicePanel";
import { useTauriEvents } from "../../hooks/useTauriEvents";
import { useRecording } from "../../hooks/useRecording";

interface NarrowViewProps {
  onSettings: () => void;
  onFullView: () => void;
}

export function NarrowView({ onSettings, onFullView }: NarrowViewProps) {
  const { summary, advices } = useTauriEvents();
  const recording = useRecording();

  return (
    <div className="flex flex-col h-screen bg-[var(--bg-primary)]">
      <ControlBar
        templateName="技术评审会"
        formattedTime={recording.formattedTime}
        status={recording.status}
        onPause={recording.pause}
        onResume={recording.resume}
        onStop={recording.stop}
        onSettings={onSettings}
        onDocuments={() => {}}
      />
      <SummaryPanel summary={summary} />
      <AdvicePanel advices={advices} />
    </div>
  );
}
