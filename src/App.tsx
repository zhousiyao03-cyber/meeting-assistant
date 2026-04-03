import { useState, useEffect } from "react";
import { NarrowView } from "./components/narrow/NarrowView";
import { FullView } from "./components/full/FullView";
import { SettingsView } from "./components/settings/SettingsView";
import { SetupGuide } from "./components/shared/SetupGuide";
import { checkWhisperModel } from "./lib/tauri";

type View = "setup" | "narrow" | "full" | "settings";

export default function App() {
  const [view, setView] = useState<View>("narrow");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    checkWhisperModel()
      .then((status) => {
        if (!status.downloaded) {
          setView("setup");
        }
      })
      .catch(() => {
        setView("setup");
      })
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="h-screen flex items-center justify-center bg-[var(--bg-primary)] text-[var(--text-muted)]">
        Loading...
      </div>
    );
  }

  return (
    <div className="h-screen bg-[var(--bg-primary)] text-[var(--text-primary)]">
      {view === "setup" && (
        <SetupGuide onComplete={() => setView("narrow")} />
      )}
      {view === "narrow" && (
        <NarrowView
          onSettings={() => setView("settings")}
          onFullView={() => setView("full")}
        />
      )}
      {view === "full" && (
        <FullView
          onNarrowView={() => setView("narrow")}
          onSettings={() => setView("settings")}
        />
      )}
      {view === "settings" && (
        <SettingsView onBack={() => setView("narrow")} />
      )}
    </div>
  );
}
