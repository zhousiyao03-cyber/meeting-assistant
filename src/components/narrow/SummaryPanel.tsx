import type { MeetingSummary } from "../../lib/types";

interface SummaryPanelProps {
  summary: MeetingSummary | null;
}

export function SummaryPanel({ summary }: SummaryPanelProps) {
  return (
    <div className="px-4 py-3 border-b border-[var(--border)]">
      <div className="flex items-center justify-between mb-2">
        <h2 className="text-sm font-medium text-[var(--text-secondary)]">
          会议摘要
        </h2>
        {summary && (
          <span className="text-xs text-[var(--text-muted)]">
            {summary.points.length} 条要点
          </span>
        )}
      </div>
      {summary ? (
        <div className="space-y-1.5">
          {summary.points.map((point, i) => (
            <div key={i} className="flex gap-2 text-sm">
              <span className="text-[var(--text-muted)] shrink-0">•</span>
              <span className="text-[var(--text-primary)]">{point}</span>
            </div>
          ))}
          {summary.current_topic && (
            <div className="mt-2 pt-2 border-t border-[var(--border)]">
              <span className="text-xs text-[var(--text-muted)]">
                当前讨论：
              </span>
              <span className="text-xs text-[var(--accent-purple)]">
                {summary.current_topic}
              </span>
            </div>
          )}
        </div>
      ) : (
        <p className="text-sm text-[var(--text-muted)]">
          等待会议内容...
        </p>
      )}
    </div>
  );
}
