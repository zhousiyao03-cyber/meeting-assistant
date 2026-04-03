import type { SpeakingAdvice } from "../../lib/types";
import { AdviceCard } from "../shared/AdviceCard";

interface AdvicePanelProps {
  advices: SpeakingAdvice[];
}

export function AdvicePanel({ advices }: AdvicePanelProps) {
  return (
    <div className="px-4 py-3 flex-1 overflow-y-auto">
      <div className="flex items-center justify-between mb-2">
        <h2 className="text-sm font-medium text-[var(--text-secondary)]">
          发言建议
        </h2>
        {advices.length > 0 && (
          <span className="text-xs px-1.5 py-0.5 rounded-full bg-[var(--accent-purple)]/20 text-[var(--accent-purple)]">
            新建议!
          </span>
        )}
      </div>
      {advices.length > 0 ? (
        <div className="space-y-3">
          {advices.map((advice, i) => (
            <AdviceCard key={i} advice={advice} isNew={i === 0} />
          ))}
        </div>
      ) : (
        <p className="text-sm text-[var(--text-muted)]">
          等待合适的发言时机...
        </p>
      )}
    </div>
  );
}
