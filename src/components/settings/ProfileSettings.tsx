import { useState, useEffect } from "react";
import type { MeetingTemplate } from "../../lib/types";
import { getTemplates, saveTemplate as saveTemplateApi } from "../../lib/tauri";

export function ProfileSettings() {
  const [templates, setTemplates] = useState<MeetingTemplate[]>([]);

  useEffect(() => {
    getTemplates().then(setTemplates).catch(console.error);
  }, []);

  const toggleTemplate = async (id: string) => {
    const updated = templates.map((t) =>
      t.id === id ? { ...t, enabled: !t.enabled } : t,
    );
    setTemplates(updated);
    const target = updated.find((t) => t.id === id);
    if (target) await saveTemplateApi(target);
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-semibold mb-1">Meeting Profiles</h2>
        <p className="text-sm text-[var(--text-muted)]">
          Configure meeting templates for different scenarios.
        </p>
      </div>

      <div className="space-y-3">
        {templates.map((t) => (
          <div
            key={t.id}
            className="bg-[var(--bg-card)] rounded-lg p-4 flex items-center justify-between"
          >
            <div>
              <h3 className="text-sm font-medium">{t.name}</h3>
              <p className="text-xs text-[var(--text-muted)]">
                {t.description}
              </p>
            </div>
            <button
              onClick={() => toggleTemplate(t.id)}
              className={`w-10 h-6 rounded-full transition-colors ${
                t.enabled
                  ? "bg-[var(--accent-purple)]"
                  : "bg-[var(--border)]"
              }`}
            >
              <div
                className={`w-4 h-4 rounded-full bg-white mx-1 transition-transform ${
                  t.enabled ? "translate-x-4" : ""
                }`}
              />
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
