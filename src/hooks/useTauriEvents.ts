import { useEffect, useState, useRef } from "react";
import type {
  TranscriptSegment,
  MeetingSummary,
  SpeakingAdvice,
} from "../lib/types";
import {
  onNewTranscript,
  onMeetingSummary,
  onSpeakingAdvice,
} from "../lib/tauri";

export function useTauriEvents() {
  const [transcripts, setTranscripts] = useState<TranscriptSegment[]>([]);
  const [summary, setSummary] = useState<MeetingSummary | null>(null);
  const [advices, setAdvices] = useState<SpeakingAdvice[]>([]);
  const unlisteners = useRef<(() => void)[]>([]);

  useEffect(() => {
    const setup = async () => {
      const u1 = await onNewTranscript((segment) => {
        setTranscripts((prev) => {
          const last = prev[prev.length - 1];
          if (last && last.text === segment.text && Math.abs(last.offset_secs - segment.offset_secs) < 1) {
            return prev;
          }
          return [...prev, segment];
        });
      });
      const u2 = await onMeetingSummary((s) => {
        setSummary(s);
      });
      const u3 = await onSpeakingAdvice((advice) => {
        setAdvices((prev) => [advice, ...prev].slice(0, 10));
      });
      unlisteners.current = [u1, u2, u3];
    };
    setup();

    return () => {
      unlisteners.current.forEach((u) => u());
    };
  }, []);

  const clearAll = () => {
    setTranscripts([]);
    setSummary(null);
    setAdvices([]);
  };

  return { transcripts, summary, advices, clearAll };
}
