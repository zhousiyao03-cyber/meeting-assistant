import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import {
  getTemplatesForLocale,
  setActiveTemplate,
  loadReferenceDoc,
  setMeetingContextNote,
  setActiveLocale,
} from "../../lib/tauri";
import type { MeetingTemplate } from "../../lib/types";

interface Props {
  defaultKind: "interview" | "general";
  defaultLocale: "zh-CN" | "en-US";
  onStart: () => void;
  onCancel: () => void;
}

export function NewMeetingDialog({
  defaultKind,
  defaultLocale,
  onStart,
  onCancel,
}: Props) {
  const { t } = useTranslation();
  const [locale, setLocale] = useState<"zh-CN" | "en-US">(defaultLocale);
  const [templates, setTemplates] = useState<MeetingTemplate[]>([]);
  const [selectedTemplateId, setSelectedTemplateId] = useState<string>(
    defaultKind === "interview" ? "job-interview" : "general-meeting",
  );
  const [docName, setDocName] = useState<string>("");
  const [contextNote, setContextNote] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    void getTemplatesForLocale(locale).then(setTemplates).catch(console.error);
  }, [locale]);

  async function handlePickDoc() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Document", extensions: ["pdf", "md", "txt"] }],
    });
    if (path && typeof path === "string") {
      try {
        const filename = await loadReferenceDoc(path);
        setDocName(filename);
      } catch (e) {
        console.error("loadReferenceDoc failed:", e);
        alert("Failed to load document: " + e);
      }
    }
  }

  async function handleStart() {
    setLoading(true);
    try {
      await setActiveLocale(locale);
      await setActiveTemplate(selectedTemplateId);
      await setMeetingContextNote(contextNote);
      onStart();
    } catch (e) {
      console.error(e);
      alert("Failed to prepare meeting: " + e);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center p-4 z-50">
      <div className="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-6 max-w-md w-full">
        <h2 className="text-lg font-bold mb-4">{t("newMeeting.title")}</h2>

        <div className="flex gap-2 mb-4 text-xs">
          <button
            onClick={() => setLocale("en-US")}
            className={`px-2 py-1 rounded ${
              locale === "en-US"
                ? "bg-[var(--accent-purple,#7c3aed)] text-white"
                : "border border-[var(--border)]"
            }`}
          >
            English
          </button>
          <button
            onClick={() => setLocale("zh-CN")}
            className={`px-2 py-1 rounded ${
              locale === "zh-CN"
                ? "bg-[var(--accent-purple,#7c3aed)] text-white"
                : "border border-[var(--border)]"
            }`}
          >
            中文
          </button>
        </div>

        <label className="text-sm block mb-1">{t("newMeeting.template")}</label>
        <select
          className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-2 py-1 mb-4"
          value={selectedTemplateId}
          onChange={(e) => setSelectedTemplateId(e.target.value)}
        >
          {templates.map((tmpl) => (
            <option key={tmpl.id} value={tmpl.id}>
              {tmpl.name}
            </option>
          ))}
        </select>

        <label className="text-sm block mb-1">{t("newMeeting.contextDoc")}</label>
        <div className="flex items-center gap-2 mb-4">
          <button
            type="button"
            onClick={() => void handlePickDoc()}
            className="px-3 py-1 border border-[var(--border)] rounded text-xs"
          >
            {docName ? t("newMeeting.changeDoc") : t("newMeeting.pickDoc")}
          </button>
          {docName && <span className="text-xs text-gray-400">{docName}</span>}
        </div>

        <label className="text-sm block mb-1">{t("newMeeting.contextNote")}</label>
        <textarea
          className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded p-2 text-sm mb-1"
          rows={3}
          maxLength={500}
          value={contextNote}
          onChange={(e) => setContextNote(e.target.value)}
          placeholder={t("newMeeting.contextNotePlaceholder")}
        />
        <div className="text-xs text-gray-500 text-right mb-4">
          {contextNote.length}/500
        </div>

        <div className="flex gap-2 justify-end">
          <button
            onClick={onCancel}
            className="px-4 py-2 border border-[var(--border)] rounded text-sm"
            disabled={loading}
          >
            {t("newMeeting.cancel")}
          </button>
          <button
            onClick={() => void handleStart()}
            className="px-4 py-2 bg-[var(--accent-purple,#7c3aed)] text-white rounded text-sm disabled:opacity-40"
            disabled={loading}
          >
            {loading ? t("newMeeting.loading") : t("newMeeting.start")}
          </button>
        </div>
      </div>
    </div>
  );
}
