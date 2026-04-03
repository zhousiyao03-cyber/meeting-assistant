import { useState, useRef, useCallback } from "react";
import { startRecording, stopRecording } from "../lib/tauri";

export type RecordingStatus = "idle" | "recording" | "paused";

export function useRecording() {
  const [status, setStatus] = useState<RecordingStatus>("idle");
  const [elapsed, setElapsed] = useState(0);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const start = useCallback(
    async (micDevice: string, captureDevice: string) => {
      await startRecording(micDevice, captureDevice);
      setStatus("recording");
      setElapsed(0);
      timerRef.current = setInterval(() => {
        setElapsed((prev) => prev + 1);
      }, 1000);
    },
    [],
  );

  const stop = useCallback(async () => {
    await stopRecording();
    setStatus("idle");
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const pause = useCallback(() => {
    setStatus("paused");
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const resume = useCallback(() => {
    setStatus("recording");
    timerRef.current = setInterval(() => {
      setElapsed((prev) => prev + 1);
    }, 1000);
  }, []);

  const formatTime = (secs: number) => {
    const h = String(Math.floor(secs / 3600)).padStart(2, "0");
    const m = String(Math.floor((secs % 3600) / 60)).padStart(2, "0");
    const s = String(secs % 60).padStart(2, "0");
    return `${h}:${m}:${s}`;
  };

  return {
    status,
    elapsed,
    formattedTime: formatTime(elapsed),
    start,
    stop,
    pause,
    resume,
  };
}
