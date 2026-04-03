import type { SpeakingAdvice } from "../../lib/types";

interface AdviceCardProps {
  advice: SpeakingAdvice;
  isNew?: boolean;
}

export function AdviceCard({ advice, isNew = false }: AdviceCardProps) {
  return (
    <div
      className={`rounded-lg p-3 transition-opacity duration-1000 ${
        isNew
          ? "bg-[var(--accent-purple)]/15 border border-[var(--accent-purple)]/40"
          : "bg-[var(--bg-card)] opacity-60"
      }`}
    >
      <div className="flex items-center gap-2 mb-2">
        <span className="text-[var(--accent-purple)] text-sm">✦</span>
        <span className="text-sm text-[var(--text-secondary)]">
          {advice.reason}
        </span>
      </div>
      <div className="text-sm leading-relaxed mb-2">
        <span className="text-[var(--text-muted)] text-xs">建议说：</span>
        <p className="mt-1 text-[var(--text-primary)]">
          &ldquo;{advice.suggestion}&rdquo;
        </p>
      </div>
      {advice.angle && (
        <div className="text-xs text-[var(--accent-purple)]">
          角度：{advice.angle}
        </div>
      )}
    </div>
  );
}
