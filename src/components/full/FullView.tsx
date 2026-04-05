import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Sidebar } from "./Sidebar";
import { TranscriptPanel } from "./TranscriptPanel";
import { CopilotPanel } from "./CopilotPanel";
import { loadDocument } from "../../lib/tauri";
import type { LoadedDocument } from "../../lib/types";

interface FullViewProps {
  onNarrowView: () => void;
  onSettings: () => void;
  events: ReturnType<typeof import("../../hooks/useTauriEvents").useTauriEvents>;
  recording: ReturnType<typeof import("../../hooks/useRecording").useRecording>;
}

export function FullView({ onNarrowView, onSettings, events, recording }: FullViewProps) {
  const { transcripts, summary, advices } = events;
  const [documents, setDocuments] = useState<LoadedDocument[]>([]);

  const handleAddDocument = async () => {
    const selected = await open({
      multiple: false,
      filters: [
        { name: "Documents", extensions: ["md", "txt", "pdf"] },
      ],
    });
    if (selected) {
      try {
        const doc = await loadDocument(selected as string);
        setDocuments((prev) => [...prev, doc]);
      } catch (e) {
        console.error("Failed to load document:", e);
      }
    }
  };

  return (
    <div className="flex h-screen bg-[var(--bg-primary)]">
      <Sidebar
        onNarrowView={onNarrowView}
        onSettings={onSettings}
        documents={documents}
        onAddDocument={handleAddDocument}
        recording={recording}
      />
      <TranscriptPanel transcripts={transcripts} />
      <CopilotPanel summary={summary} advices={advices} />
    </div>
  );
}
