import { useState, useRef, useCallback, useEffect } from "react";
import { startRecording, stopRecording, pauseRecording, resumeRecording, getRecordingStatus, saveMeeting, getTranscript, generateMeetingMinutes } from "../lib/tauri";
import type { MeetingSummary, SpeakingAdvice } from "../lib/types";

export type RecordingStatus = "idle" | "recording" | "paused";

export function useRecording() {
  const [status, setStatus] = useState<RecordingStatus>("idle");
  const [elapsed, setElapsed] = useState(0);
  const elapsedRef = useRef(0);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const startedAtRef = useRef<string>("");

  // Keep ref in sync with state for use in callbacks
  useEffect(() => { elapsedRef.current = elapsed; }, [elapsed]);

  // Sync with backend state on mount
  useEffect(() => {
    getRecordingStatus().then((s) => {
      if (s.is_recording) {
        setStatus(s.is_paused ? "paused" : "recording");
        setElapsed(s.elapsed_secs);
        if (!s.is_paused) {
          timerRef.current = setInterval(() => {
            setElapsed((prev) => prev + 1);
          }, 1000);
        }
      }
    }).catch(console.error);

    return () => {
      if (timerRef.current) {
        clearInterval(timerRef.current);
      }
    };
  }, []);

  const start = useCallback(
    async (micDevice: string, captureDevice: string) => {
      setStatus("recording");
      setElapsed(0);
      startedAtRef.current = new Date().toISOString();
      timerRef.current = setInterval(() => {
        setElapsed((prev) => prev + 1);
      }, 1000);
      try {
        await startRecording(micDevice, captureDevice);
      } catch (e: any) {
        console.error("Failed to start recording:", e);
        setStatus("idle");
        if (timerRef.current) {
          clearInterval(timerRef.current);
          timerRef.current = null;
        }
      }
    },
    [],
  );

  const stop = useCallback(async (
    summary?: MeetingSummary | null,
    advices?: SpeakingAdvice[],
    templateName?: string,
  ): Promise<string | undefined> => {
    const currentElapsed = elapsedRef.current;
    await stopRecording();
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
    setStatus("idle");
    setElapsed(0);

    // Auto-save meeting history with generated minutes
    try {
      const segments = await getTranscript();
      const transcript = segments.map((s) => s.text).join(" ");
      if (transcript.length > 0) {
        const summaryText = summary ? summary.points.join("\n") : "";

        // Generate meeting minutes via LLM
        let title = templateName || "会议纪要";
        let actionItems = "";
        try {
          const minutes = await generateMeetingMinutes(transcript, summaryText);
          title = minutes.title || title;
          actionItems = minutes.action_items.join("\n");
        } catch (e) {
          console.error("Failed to generate minutes:", e);
        }

        const meetingId = crypto.randomUUID();
        await saveMeeting({
          id: meetingId,
          title,
          template_name: templateName || "未选择模板",
          started_at: startedAtRef.current || new Date().toISOString(),
          duration_secs: currentElapsed,
          transcript,
          summary: summaryText,
          action_items: actionItems,
          advices_json: JSON.stringify(advices || []),
        });

        return meetingId;
      }
    } catch (e) {
      console.error("Failed to save meeting:", e);
    }
    return undefined;
  }, []);

  const pause = useCallback(async () => {
    try {
      await pauseRecording();
    } catch (e) {
      console.error("Failed to pause:", e);
    }
    setStatus("paused");
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const resume = useCallback(async () => {
    try {
      await resumeRecording();
    } catch (e) {
      console.error("Failed to resume:", e);
    }
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
