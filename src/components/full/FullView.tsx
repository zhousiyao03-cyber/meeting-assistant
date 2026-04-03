import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Sidebar } from "./Sidebar";
import { TranscriptPanel } from "./TranscriptPanel";
import { CopilotPanel } from "./CopilotPanel";
import { useTauriEvents } from "../../hooks/useTauriEvents";
import { loadDocument } from "../../lib/tauri";
import type { LoadedDocument } from "../../lib/types";

interface FullViewProps {
  onNarrowView: () => void;
  onSettings: () => void;
}

export function FullView({ onNarrowView, onSettings }: FullViewProps) {
  const { transcripts, summary, advices } = useTauriEvents();
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
        onStop={onNarrowView}
        onSettings={onSettings}
        documents={documents}
        onAddDocument={handleAddDocument}
      />
      <TranscriptPanel transcripts={transcripts} />
      <CopilotPanel summary={summary} advices={advices} />
    </div>
  );
}
