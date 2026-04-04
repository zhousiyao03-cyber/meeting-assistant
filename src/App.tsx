import { useState } from "react";
import { NarrowView } from "./components/narrow/NarrowView";
import { FullView } from "./components/full/FullView";
import { SettingsView } from "./components/settings/SettingsView";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { useRecording } from "./hooks/useRecording";

type View = "narrow" | "full" | "settings";

export default function App() {
  const [view, setView] = useState<View>("narrow");
  const events = useTauriEvents();
  const recording = useRecording();

  return (
    <div className="h-screen bg-[var(--bg-primary)] text-[var(--text-primary)]">
      {view === "narrow" && (
        <NarrowView
          onSettings={() => setView("settings")}
          onFullView={() => setView("full")}
          events={events}
          recording={recording}
        />
      )}
      {view === "full" && (
        <FullView
          onNarrowView={() => setView("narrow")}
          onSettings={() => setView("settings")}
          events={events}
        />
      )}
      {view === "settings" && (
        <SettingsView onBack={() => setView("narrow")} />
      )}
    </div>
  );
}
