import type { MeetingSummary, SpeakingAdvice } from "../../lib/types";
import { AdviceCard } from "../shared/AdviceCard";

interface CopilotPanelProps {
  summary: MeetingSummary | null;
  advices: SpeakingAdvice[];
}

export function CopilotPanel({ summary, advices }: CopilotPanelProps) {
  return (
    <div className="w-80 flex flex-col">
      <div className="px-4 py-3 border-b border-[var(--border)]">
        <h2 className="text-sm font-medium">AI Copilot</h2>
      </div>
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Summary section */}
        <div>
          <h3 className="text-xs text-[var(--accent-orange)] uppercase mb-2">
            ● Meeting Summary
          </h3>
          {summary ? (
            <div className="text-sm text-[var(--text-secondary)] space-y-1">
              {summary.points.map((p, i) => (
                <p key={i}>• {p}</p>
              ))}
              {summary.current_topic && (
                <p className="text-[var(--accent-purple)] mt-2">
                  当前：{summary.current_topic}
                </p>
              )}
            </div>
          ) : (
            <p className="text-sm text-[var(--text-muted)]">
              Analyzing discussion...
            </p>
          )}
        </div>

        {/* Advice section */}
        <div>
          <h3 className="text-xs text-[var(--accent-purple)] uppercase mb-2">
            ✦ Speaking Suggestions
          </h3>
          <div className="space-y-3">
            {advices.map((advice, i) => (
              <AdviceCard key={i} advice={advice} isNew={i === 0} />
            ))}
            {advices.length === 0 && (
              <p className="text-sm text-[var(--text-muted)]">
                Waiting for the right moment...
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
