import { useEffect, useRef, useCallback } from "react";
import type { TranscriptSegment } from "../../lib/types";

interface TranscriptMiniProps {
  transcripts: TranscriptSegment[];
}

export function TranscriptMini({ transcripts }: TranscriptMiniProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const userScrolledUp = useRef(false);

  const handleScroll = useCallback(() => {
    const el = containerRef.current;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
    userScrolledUp.current = !atBottom;
  }, []);

  useEffect(() => {
    if (!userScrolledUp.current) {
      bottomRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [transcripts]);

  const formatOffset = (secs: number) => {
    const m = String(Math.floor(secs / 60)).padStart(2, "0");
    const s = String(Math.floor(secs % 60)).padStart(2, "0");
    return `${m}:${s}`;
  };

  return (
    <div
      ref={containerRef}
      onScroll={handleScroll}
      className="px-4 py-3 border-b border-[var(--border)] max-h-48 overflow-y-auto"
    >
      <h2 className="text-sm font-medium text-[var(--text-secondary)] mb-2">
        实时转写
      </h2>
      {transcripts.length > 0 ? (
        <div className="space-y-1.5">
          {transcripts.slice(-10).map((seg) => (
            <div key={seg.offset_secs} className="flex gap-2 text-xs">
              <span className="text-[var(--text-muted)] shrink-0 font-mono">
                {formatOffset(seg.offset_secs)}
              </span>
              <span className="text-[var(--text-primary)]">{seg.text}</span>
            </div>
          ))}
          <div ref={bottomRef} />
        </div>
      ) : (
        <p className="text-xs text-[var(--text-muted)]">等待语音输入...</p>
      )}
    </div>
  );
}
