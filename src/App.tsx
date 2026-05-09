import { useState, useRef, useEffect } from "react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { NarrowView } from "./components/narrow/NarrowView";
import { FullView } from "./components/full/FullView";
import { SettingsView } from "./components/settings/SettingsView";
import { MeetingHistory } from "./components/history/MeetingHistory";
import { useTauriEvents } from "./hooks/useTauriEvents";
import { useRecording } from "./hooks/useRecording";
import { PermissionGate } from "./components/onboarding/PermissionGate";
import { WelcomeFlow, isOnboardingDone } from "./components/onboarding/WelcomeFlow";
import { NewMeetingDialog } from "./components/meeting/NewMeetingDialog";
import { QuotaExhausted } from "./components/license/QuotaExhausted";
import {
  getUserPlan,
  onPanicStop,
  onMenuNewMeeting,
  onPlanUpdated,
  onQuotaExhausted,
  stopRecording,
} from "./lib/tauri";
import type { UserPlan } from "./lib/types";

type View = "narrow" | "full" | "settings" | "history";

const NARROW_SIZE = { width: 320, height: 500 };
const FULL_SIZE = { width: 1200, height: 840 };

function isWideView(v: View) {
  return v === "full" || v === "history";
}

export default function App() {
  const [onboardingDone, setOnboardingDone] = useState(isOnboardingDone());
  const [view, setView] = useState<View>("narrow");
  const prevViewRef = useRef<View>("narrow");
  const events = useTauriEvents();
  const recording = useRecording();
  const [_plan, setPlan] = useState<UserPlan | null>(null);
  const [showNewMeetingDialog, setShowNewMeetingDialog] = useState(false);
  const [newMeetingKind, setNewMeetingKind] = useState<"interview" | "general">("general");
  const [showQuotaExhausted, setShowQuotaExhausted] = useState(false);

  // Initial plan load
  useEffect(() => {
    void getUserPlan().then(setPlan).catch(console.error);
  }, []);

  // Stealth + license event listeners
  useEffect(() => {
    let unlistens: Array<() => void> = [];
    void (async () => {
      unlistens.push(
        await onPanicStop(async () => {
          try {
            await stopRecording();
          } catch (e) {
            console.error("panic stop failed:", e);
          }
        }),
      );
      unlistens.push(
        await onMenuNewMeeting((kind) => {
          setNewMeetingKind(kind);
          setShowNewMeetingDialog(true);
        }),
      );
      unlistens.push(
        await onPlanUpdated((p) => setPlan(p)),
      );
      unlistens.push(
        await onQuotaExhausted(() => setShowQuotaExhausted(true)),
      );
    })();
    return () => unlistens.forEach((u) => u());
  }, []);

  const resizeWindow = (target: View) => {
    const win = getCurrentWindow();
    if (isWideView(target)) {
      void win.setSize(new LogicalSize(FULL_SIZE.width, FULL_SIZE.height));
    } else {
      void win.setSize(new LogicalSize(NARROW_SIZE.width, NARROW_SIZE.height));
    }
  };

  const switchView = (target: View) => {
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

  if (!onboardingDone) {
    return <WelcomeFlow onDone={() => setOnboardingDone(true)} />;
  }

  return (
    <PermissionGate>
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
        {view === "settings" && <SettingsView onBack={goBack} />}
        {view === "history" && <MeetingHistory onBack={goBack} />}

        {showNewMeetingDialog && (
          <NewMeetingDialog
            defaultKind={newMeetingKind}
            defaultLocale="en-US"
            onStart={() => {
              setShowNewMeetingDialog(false);
              setView("narrow");
            }}
            onCancel={() => setShowNewMeetingDialog(false)}
          />
        )}
        {showQuotaExhausted && (
          <QuotaExhausted onClose={() => setShowQuotaExhausted(false)} />
        )}
      </div>
    </PermissionGate>
  );
}
