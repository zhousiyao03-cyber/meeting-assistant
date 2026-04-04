import { useEffect, useRef, useCallback } from "react";
import type { TranscriptSegment } from "../../lib/types";

interface TranscriptPanelProps {
  transcripts: TranscriptSegment[];
}

export function TranscriptPanel({ transcripts }: TranscriptPanelProps) {
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
    <div className="flex-1 flex flex-col border-r border-[var(--border)]">
      <div className="px-4 py-3 border-b border-[var(--border)] flex items-center justify-between">
        <h2 className="text-sm font-medium">Live Transcript</h2>
        <span className="text-xs text-[var(--accent-green)]">● LIVE</span>
      </div>
      <div
        ref={containerRef}
        onScroll={handleScroll}
        className="flex-1 overflow-y-auto p-4 space-y-3"
      >
        {transcripts.map((seg) => (
          <div key={seg.offset_secs} className="flex gap-3">
            <span className="text-xs text-[var(--text-muted)] shrink-0 pt-0.5 font-mono">
              {formatOffset(seg.offset_secs)}
            </span>
            <p className="text-sm text-[var(--text-primary)] leading-relaxed">
              {seg.text}
            </p>
          </div>
        ))}
        {transcripts.length === 0 && (
          <p className="text-sm text-[var(--text-muted)] text-center mt-8">
            Transcribing...
          </p>
        )}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}
