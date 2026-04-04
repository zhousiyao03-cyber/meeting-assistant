import { useState, useEffect } from "react";
import type { MeetingTemplate, LoadedDocument } from "../../lib/types";
import { getTemplates, setActiveTemplate } from "../../lib/tauri";

interface SidebarProps {
  onStop: () => void;
  onSettings: () => void;
  documents: LoadedDocument[];
  onAddDocument: () => void;
}

export function Sidebar({
  onStop,
  onSettings,
  documents,
  onAddDocument,
}: SidebarProps) {
  const [templates, setTemplates] = useState<MeetingTemplate[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState<string>("");

  useEffect(() => {
    getTemplates().then((t) => {
      setTemplates(t);
      if (t.length > 0) { setSelectedTemplate(t[0].id); setActiveTemplate(t[0].id); }
    });
  }, []);

  return (
    <div className="w-64 border-r border-[var(--border)] flex flex-col bg-[var(--bg-secondary)]">
      <div className="p-4 border-b border-[var(--border)]">
        <h1 className="text-lg font-semibold">Meeting Copilot</h1>
        <span className="text-xs text-[var(--accent-green)]">● LIVE</span>
      </div>

      <div className="p-4 flex-1 overflow-y-auto">
        <h3 className="text-xs text-[var(--text-muted)] uppercase mb-2">
          会议类型
        </h3>
        <div className="space-y-1 mb-6">
          {templates.map((t) => (
            <button
              key={t.id}
              onClick={() => { setSelectedTemplate(t.id); setActiveTemplate(t.id); }}
              className={`w-full text-left px-3 py-2 rounded text-sm ${
                selectedTemplate === t.id
                  ? "bg-[var(--accent-purple)]/20 text-[var(--accent-purple)]"
                  : "text-[var(--text-secondary)] hover:bg-[var(--bg-card)]"
              }`}
            >
              {t.name}
            </button>
          ))}
        </div>

        <h3 className="text-xs text-[var(--text-muted)] uppercase mb-2">
          参考文档
        </h3>
        <div className="space-y-1">
          {documents.map((doc, i) => (
            <div
              key={i}
              className="flex items-center gap-2 px-3 py-2 rounded bg-[var(--bg-card)] text-sm"
            >
              <span className="text-[var(--text-muted)]">📄</span>
              <span className="truncate text-[var(--text-secondary)]">
                {doc.filename}
              </span>
            </div>
          ))}
          <button
            onClick={onAddDocument}
            className="w-full text-left px-3 py-2 rounded text-sm text-[var(--text-muted)] hover:bg-[var(--bg-card)]"
          >
            + Add document
          </button>
        </div>
      </div>

      <div className="p-4 border-t border-[var(--border)] space-y-2">
        <button
          onClick={onStop}
          className="w-full py-2 rounded bg-[var(--accent-red)]/20 text-[var(--accent-red)] text-sm hover:bg-[var(--accent-red)]/30"
        >
          ⏹ Stop Recording
        </button>
        <button
          onClick={onSettings}
          className="w-full py-2 rounded bg-[var(--bg-card)] text-[var(--text-secondary)] text-sm hover:bg-[var(--bg-card-hover)]"
        >
          Settings
        </button>
      </div>
    </div>
  );
}
