import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { isStealthOn, onStealthChanged, setStealthMode } from "../../lib/tauri";

export function StealthIndicator() {
  const { t } = useTranslation();
  const [on, setOn] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void (async () => {
      try {
        setOn(await isStealthOn());
      } catch {}
      unlisten = await onStealthChanged((v) => setOn(v));
    })();
    return () => unlisten?.();
  }, []);

  return (
    <button
      onClick={() => void setStealthMode(!on)}
      className={`text-xs px-2 py-1 rounded transition-colors ${
        on
          ? "bg-red-900/40 text-red-300 border border-red-700/50"
          : "bg-gray-800/40 text-gray-400 border border-gray-700/50"
      }`}
      title={t("stealth.tooltip")}
    >
      {on ? t("stealth.indicatorOn") : t("stealth.indicatorOff")}
    </button>
  );
}
