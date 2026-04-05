import { useState, useRef } from "react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { NarrowView } from "./components/narrow/NarrowView";
import { FullView } from "./components/full/FullView";
import { SettingsView } from "./components/settings/SettingsView";
import { MeetingHistory } from "./components/history/MeetingHistory";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { useRecording } from "./hooks/useRecording";

type View = "narrow" | "full" | "settings" | "history";

const NARROW_SIZE = { width: 420, height: 840 };
const FULL_SIZE = { width: 1200, height: 840 };

function isWideView(v: View) {
  return v === "full" || v === "history";
}

export default function App() {
  const [view, setView] = useState<View>("narrow");
  // prevView 记录进入 settings 前的主视图，用 ref 避免闭包问题
  const prevViewRef = useRef<View>("narrow");
  const events = useTauriEvents();
  const recording = useRecording();

  const resizeWindow = (target: View) => {
    const win = getCurrentWindow();
    if (isWideView(target)) {
      win.setSize(new LogicalSize(FULL_SIZE.width, FULL_SIZE.height));
    } else {
      win.setSize(new LogicalSize(NARROW_SIZE.width, NARROW_SIZE.height));
    }
  };

  const switchView = (target: View) => {
    // 设置页面继承当前窗口大小，不 resize
    if (target === "settings") {
      prevViewRef.current = view === "settings" ? prevViewRef.current : view;
      setView(target);
      return;
    }
    resizeWindow(target);
    if (view !== "settings") {
      prevViewRef.current = view;
    }
    setView(target);
  };

  const goBack = () => {
    const back = prevViewRef.current;
    resizeWindow(back);
    setView(back);
  };

  return (
    <div className="h-screen bg-[var(--bg-primary)] text-[var(--text-primary)]">
      {view === "narrow" && (
        <NarrowView
          onSettings={() => switchView("settings")}
          onFullView={() => switchView("full")}
          onHistory={() => switchView("history")}
          events={events}
          recording={recording}
        />
      )}
      {view === "full" && (
        <FullView
          onNarrowView={() => switchView("narrow")}
          onSettings={() => switchView("settings")}
          events={events}
          recording={recording}
        />
      )}
      {view === "settings" && (
        <SettingsView onBack={goBack} />
      )}
      {view === "history" && (
        <MeetingHistory onBack={goBack} />
      )}
    </div>
  );
}
