import type { RecordingStatus } from "../../hooks/useRecording";

interface ControlBarProps {
  templateName: string;
  formattedTime: string;
  status: RecordingStatus;
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onStop: () => void;
  onSettings: () => void;
  onDocuments: () => void;
}

export function ControlBar({
  templateName,
  formattedTime,
  status,
  onStart,
  onPause,
  onResume,
  onStop,
  onSettings,
  onDocuments,
}: ControlBarProps) {
  return (
    <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--border)]">
      <div className="flex items-center gap-2">
        {status === "recording" && (
          <span className="w-2 h-2 rounded-full bg-[var(--accent-red)] animate-pulse" />
        )}
        <span className="text-sm font-medium">{templateName}</span>
      </div>
      <span className="text-sm text-[var(--text-muted)] font-mono">
        {formattedTime}
      </span>
      <div className="flex items-center gap-1">
        <button
          onClick={onDocuments}
          className="p-1.5 rounded hover:bg-[var(--bg-card)] text-sm"
          title="参考文档"
        >
          📎
        </button>
        <button
          onClick={onSettings}
          className="p-1.5 rounded hover:bg-[var(--bg-card)] text-sm"
          title="设置"
        >
          ⚙
        </button>
        {status === "idle" ? (
          <button
            onClick={onStart}
            className="p-1.5 rounded hover:bg-[var(--bg-card)] text-sm text-[var(--accent-green)]"
            title="开始录制"
          >
            ●
          </button>
        ) : status === "recording" ? (
          <>
            <button
              onClick={onPause}
              className="p-1.5 rounded hover:bg-[var(--bg-card)] text-sm"
              title="暂停"
            >
              ⏸
            </button>
            <button
              onClick={onStop}
              className="p-1.5 rounded hover:bg-[var(--bg-card)] text-sm text-[var(--accent-red)]"
              title="停止"
            >
              ■
            </button>
          </>
        ) : (
          <>
            <button
              onClick={onResume}
              className="p-1.5 rounded hover:bg-[var(--bg-card)] text-sm"
              title="继续"
            >
              ▶
            </button>
            <button
              onClick={onStop}
              className="p-1.5 rounded hover:bg-[var(--bg-card)] text-sm text-[var(--accent-red)]"
              title="停止"
            >
              ■
            </button>
          </>
        )}
      </div>
    </div>
  );
}
