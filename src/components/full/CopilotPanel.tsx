import { useState, useCallback, useRef } from "react";
import type { MeetingSummary, SpeakingAdvice } from "../../lib/types";
import { AdviceCard } from "../shared/AdviceCard";

interface CopilotPanelProps {
  summary: MeetingSummary | null;
  advices: SpeakingAdvice[];
}

export function CopilotPanel({ summary, advices }: CopilotPanelProps) {
  const [width, setWidth] = useState(420);
  const dragging = useRef(false);

  const onMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    dragging.current = true;
    const startX = e.clientX;
    const startWidth = width;

    const onMouseMove = (e: MouseEvent) => {
      if (!dragging.current) return;
      const newWidth = startWidth - (e.clientX - startX);
      setWidth(Math.max(280, Math.min(700, newWidth)));
    };

    const onMouseUp = () => {
      dragging.current = false;
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);
    };

    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
  }, [width]);

  return (
    <div className="flex flex-col border-l border-[var(--border)] relative" style={{ width, minWidth: 280 }}>
      <div
        onMouseDown={onMouseDown}
        className="absolute left-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-[var(--accent-purple)]/40 active:bg-[var(--accent-purple)]/60 z-10"
      />
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
