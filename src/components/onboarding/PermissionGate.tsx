import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  checkScreenRecordingPermission,
  openScreenRecordingSettings,
} from "../../lib/tauri";

interface Props {
  children: React.ReactNode;
}

type Status = "checking" | "ok" | "denied" | "macos-too-old";

export function PermissionGate({ children }: Props) {
  const { t } = useTranslation();
  const [status, setStatus] = useState<Status>("checking");

  useEffect(() => {
    void check();
  }, []);

  async function check() {
    try {
      const r = await checkScreenRecordingPermission();
      if (!r.macos_version_ok) {
        setStatus("macos-too-old");
        return;
      }
      setStatus(r.status === "granted" ? "ok" : "denied");
    } catch (e) {
      console.error("permission check failed:", e);
      setStatus("denied");
    }
  }

  if (status === "checking") {
    return <div className="p-8 text-center text-sm">…</div>;
  }

  if (status === "macos-too-old") {
    return (
      <div className="p-8 max-w-md mx-auto text-[var(--text-primary)]">
        <h2 className="text-xl font-bold mb-3">
          {t("onboarding.macosTooOld.title")}
        </h2>
        <p className="text-sm leading-relaxed">
          {t("onboarding.macosTooOld.body")}
        </p>
      </div>
    );
  }

  if (status === "denied") {
    return (
      <div className="p-8 max-w-md mx-auto text-[var(--text-primary)]">
        <h2 className="text-xl font-bold mb-3">
          {t("onboarding.screenRecording.title")}
        </h2>
        <p className="text-sm mb-4 leading-relaxed">
          {t("onboarding.screenRecording.body")}
        </p>
        <p className="text-sm mb-6">
          {t("onboarding.screenRecording.afterEnable")}
        </p>
        <div className="flex gap-3">
          <button
            className="px-4 py-2 bg-[var(--accent-purple,#7c3aed)] rounded text-white text-sm"
            onClick={() => void openScreenRecordingSettings()}
          >
            {t("onboarding.screenRecording.openSettings")}
          </button>
          <button
            className="px-4 py-2 border border-[var(--border)] rounded text-sm"
            onClick={() => void check()}
          >
            {t("onboarding.screenRecording.recheck")}
          </button>
        </div>
      </div>
    );
  }

  return <>{children}</>;
}
