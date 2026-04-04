import { useState, useEffect } from "react";
import type { MeetingRecord } from "../../lib/types";
import { listMeetings, deleteMeeting } from "../../lib/tauri";

interface MeetingHistoryProps {
  onBack: () => void;
}

export function MeetingHistory({ onBack }: MeetingHistoryProps) {
  const [meetings, setMeetings] = useState<MeetingRecord[]>([]);
  const [selected, setSelected] = useState<MeetingRecord | null>(null);

  useEffect(() => {
    listMeetings().then(setMeetings).catch(console.error);
  }, []);

  const handleDelete = async (id: string) => {
    try {
      await deleteMeeting(id);
      setMeetings((prev) => prev.filter((m) => m.id !== id));
      if (selected?.id === id) setSelected(null);
    } catch (e) {
      console.error("Failed to delete:", e);
    }
  };

  const exportMarkdown = (meeting: MeetingRecord) => {
    const duration = `${Math.floor(meeting.duration_secs / 60)}分${meeting.duration_secs % 60}秒`;
    const actionItems = meeting.action_items
      ? meeting.action_items.split("\n").map((a) => `- [ ] ${a}`).join("\n")
      : "无";
    const summaryPoints = meeting.summary
      ? meeting.summary.split("\n").map((s) => `- ${s}`).join("\n")
      : "无";

    const md = `# ${meeting.title || meeting.template_name}

**日期**: ${new Date(meeting.started_at).toLocaleString("zh-CN")}
**时长**: ${duration}
**会议类型**: ${meeting.template_name}

## 会议要点

${summaryPoints}

## 行动项

${actionItems}

## 完整转录

${meeting.transcript}
`;

    const blob = new Blob([md], { type: "text/markdown" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${meeting.title || "会议纪要"}-${meeting.started_at.slice(0, 10)}.md`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const formatDate = (iso: string) => {
    try {
      return new Date(iso).toLocaleString("zh-CN", {
        month: "numeric",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return iso;
    }
  };

  const formatDuration = (secs: number) => {
    if (secs < 60) return `${secs}秒`;
    return `${Math.floor(secs / 60)}分${secs % 60}秒`;
  };

  return (
    <div className="flex h-screen bg-[var(--bg-primary)]">
      {/* Sidebar: meeting list */}
      <div className="w-72 border-r border-[var(--border)] bg-[var(--bg-secondary)] flex flex-col">
        <div className="p-4 border-b border-[var(--border)]">
          <button
            onClick={onBack}
            className="text-sm text-[var(--text-muted)] hover:text-[var(--text-primary)]"
          >
            ← 返回
          </button>
          <h2 className="text-lg font-semibold mt-2">会议历史</h2>
          <p className="text-xs text-[var(--text-muted)]">
            {meetings.length} 场会议
          </p>
        </div>
        <div className="flex-1 overflow-y-auto">
          {meetings.map((m) => (
            <button
              key={m.id}
              onClick={() => setSelected(m)}
              className={`w-full text-left px-4 py-3 border-b border-[var(--border)] hover:bg-[var(--bg-card)] transition-colors ${
                selected?.id === m.id
                  ? "bg-[var(--accent-purple)]/10 border-l-2 border-l-[var(--accent-purple)]"
                  : ""
              }`}
            >
              <div className="text-sm font-medium truncate">
                {m.title || m.template_name}
              </div>
              <div className="text-xs text-[var(--text-muted)] mt-1 flex gap-2">
                <span>{formatDate(m.started_at)}</span>
                <span>{formatDuration(m.duration_secs)}</span>
              </div>
            </button>
          ))}
          {meetings.length === 0 && (
            <div className="p-4 text-sm text-[var(--text-muted)] text-center">
              暂无会议记录
            </div>
          )}
        </div>
      </div>

      {/* Content: meeting detail */}
      <div className="flex-1 overflow-y-auto p-6">
        {selected ? (
          <div className="max-w-3xl">
            <div className="flex items-start justify-between mb-6">
              <div>
                <h1 className="text-xl font-semibold">
                  {selected.title || selected.template_name}
                </h1>
                <p className="text-sm text-[var(--text-muted)] mt-1">
                  {formatDate(selected.started_at)} · {formatDuration(selected.duration_secs)} · {selected.template_name}
                </p>
              </div>
              <div className="flex gap-2">
                <button
                  onClick={() => exportMarkdown(selected)}
                  className="px-3 py-1.5 text-xs bg-[var(--accent-purple)] text-white rounded hover:opacity-90"
                >
                  导出 Markdown
                </button>
                <button
                  onClick={() => handleDelete(selected.id)}
                  className="px-3 py-1.5 text-xs bg-red-500/20 text-red-400 rounded hover:bg-red-500/30"
                >
                  删除
                </button>
              </div>
            </div>

            {/* Summary */}
            {selected.summary && (
              <section className="mb-6">
                <h2 className="text-sm font-medium text-[var(--accent-orange)] uppercase mb-2">
                  会议要点
                </h2>
                <div className="text-sm text-[var(--text-secondary)] space-y-1">
                  {selected.summary.split("\n").map((p, i) => (
                    <p key={i}>• {p}</p>
                  ))}
                </div>
              </section>
            )}

            {/* Action Items */}
            {selected.action_items && (
              <section className="mb-6">
                <h2 className="text-sm font-medium text-[var(--accent-purple)] uppercase mb-2">
                  行动项
                </h2>
                <div className="text-sm text-[var(--text-secondary)] space-y-1">
                  {selected.action_items.split("\n").map((a, i) => (
                    <p key={i}>☐ {a}</p>
                  ))}
                </div>
              </section>
            )}

            {/* Transcript */}
            <section>
              <h2 className="text-sm font-medium text-[var(--text-muted)] uppercase mb-2">
                完整转录
              </h2>
              <div className="text-sm text-[var(--text-secondary)] leading-relaxed bg-[var(--bg-card)] rounded-lg p-4 max-h-96 overflow-y-auto">
                {selected.transcript}
              </div>
            </section>
          </div>
        ) : (
          <div className="flex items-center justify-center h-full text-sm text-[var(--text-muted)]">
            选择一场会议查看详情
          </div>
        )}
      </div>
    </div>
  );
}
