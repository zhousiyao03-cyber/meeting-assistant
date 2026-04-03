import { useState } from "react";
import { Sidebar } from "./Sidebar";
import { TranscriptPanel } from "./TranscriptPanel";
import { CopilotPanel } from "./CopilotPanel";
import { useTauriEvents } from "../../hooks/useTauriEvents";
import type { LoadedDocument } from "../../lib/types";

interface FullViewProps {
  onNarrowView: () => void;
  onSettings: () => void;
}

export function FullView({ onNarrowView, onSettings }: FullViewProps) {
  const { transcripts, summary, advices } = useTauriEvents();
  const [documents, setDocuments] = useState<LoadedDocument[]>([]);

  return (
    <div className="flex h-screen bg-[var(--bg-primary)]">
      <Sidebar
        onStop={onNarrowView}
        onSettings={onSettings}
        documents={documents}
        onAddDocument={() => {
          // TODO: open file dialog via Tauri
        }}
      />
      <TranscriptPanel transcripts={transcripts} />
      <CopilotPanel summary={summary} advices={advices} />
    </div>
  );
}
