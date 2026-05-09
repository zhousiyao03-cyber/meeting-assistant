import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  checkScreenRecordingPermission,
  openScreenRecordingSettings,
  listAudioDevices,
  saveConfig,
  getConfig,
} from "../../lib/tauri";
import { setUiLanguage } from "../../i18n/config";
import type { AudioDevice } from "../../lib/types";

const ONBOARDING_DONE_KEY = "confide.onboardingDone";

export function isOnboardingDone(): boolean {
  return typeof localStorage !== "undefined" && localStorage.getItem(ONBOARDING_DONE_KEY) === "true";
}

export function WelcomeFlow({ onDone }: { onDone: () => void }) {
  const { t, i18n } = useTranslation();
  const [step, setStep] = useState(0);
  const totalSteps = 4;

  function next() {
    if (step + 1 >= totalSteps) {
      localStorage.setItem(ONBOARDING_DONE_KEY, "true");
      onDone();
    } else {
      setStep(step + 1);
    }
  }

  return (
    <div className="fixed inset-0 bg-[var(--bg-primary)] z-50 flex items-center justify-center p-6 text-[var(--text-primary)]">
      <div className="max-w-md w-full">
        <div className="text-xs text-gray-500 mb-2">
          Step {step + 1} of {totalSteps}
        </div>

        {step === 0 && (
          <div>
            <h1 className="text-2xl font-bold mb-4">{t("onboarding.welcome.title")}</h1>
            <p className="text-sm text-gray-400 mb-6">{t("onboarding.welcome.tagline")}</p>
            <p className="text-sm mb-2">{t("onboarding.welcome.chooseLang")}</p>
            <div className="flex gap-2 mb-6">
              <button
                onClick={() => setUiLanguage("en-US")}
                className={`px-4 py-2 border rounded ${
                  i18n.language === "en-US"
                    ? "bg-[var(--accent-purple,#7c3aed)] text-white border-transparent"
                    : "border-[var(--border)]"
                }`}
              >
                English
              </button>
              <button
                onClick={() => setUiLanguage("zh-CN")}
                className={`px-4 py-2 border rounded ${
                  i18n.language === "zh-CN"
                    ? "bg-[var(--accent-purple,#7c3aed)] text-white border-transparent"
                    : "border-[var(--border)]"
                }`}
              >
                中文
              </button>
            </div>
            <button
              onClick={next}
              className="px-6 py-2 bg-[var(--accent-purple,#7c3aed)] text-white rounded"
            >
              {t("onboarding.welcome.continue")}
            </button>
          </div>
        )}

        {step === 1 && <ScreenRecordingStep onContinue={next} />}
        {step === 2 && <MicrophoneStep onContinue={next} />}
        {step === 3 && (
          <div>
            <h2 className="text-xl font-bold mb-4">{t("onboarding.done.title")}</h2>
            <p className="text-sm text-gray-400 mb-6">{t("onboarding.done.body")}</p>
            <button
              onClick={next}
              className="px-6 py-2 bg-[var(--accent-purple,#7c3aed)] text-white rounded"
            >
              {t("onboarding.done.cta")}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

function ScreenRecordingStep({ onContinue }: { onContinue: () => void }) {
  const { t } = useTranslation();
  const [status, setStatus] = useState<"checking" | "denied" | "granted">("checking");

  async function check() {
    try {
      const r = await checkScreenRecordingPermission();
      setStatus(r.status === "granted" ? "granted" : "denied");
      if (r.status === "granted") {
        setTimeout(onContinue, 600);
      }
    } catch {
      setStatus("denied");
    }
  }

  useEffect(() => {
    void check();
  }, []);

  return (
    <div>
      <h2 className="text-xl font-bold mb-3">
        {t("onboarding.screenRecording.title")}
      </h2>
      <p className="text-sm text-gray-400 mb-6">
        {t("onboarding.screenRecording.body")}
      </p>
      {status === "denied" && (
        <div className="space-y-3">
          <button
            onClick={() => void openScreenRecordingSettings()}
            className="px-4 py-2 bg-[var(--accent-purple,#7c3aed)] text-white rounded"
          >
            {t("onboarding.screenRecording.openSettings")}
          </button>
          <button
            onClick={() => void check()}
            className="px-4 py-2 border border-[var(--border)] rounded ml-2"
          >
            {t("onboarding.screenRecording.recheck")}
          </button>
          <p className="text-xs text-gray-500">
            {t("onboarding.screenRecording.afterEnable")}
          </p>
        </div>
      )}
      {status === "granted" && (
        <div className="text-green-400">{t("onboarding.screenRecording.granted")}</div>
      )}
    </div>
  );
}

function MicrophoneStep({ onContinue }: { onContinue: () => void }) {
  const { t } = useTranslation();
  const [devices, setDevices] = useState<AudioDevice[]>([]);

  async function load() {
    try {
      const d = await listAudioDevices();
      setDevices(d);
      if (d.length > 0) {
        const cfg = await getConfig();
        await saveConfig({
          ...cfg,
          audio: { ...cfg.audio, mic_device: d[0].name },
        });
      }
    } catch (e) {
      console.error(e);
    }
  }

  useEffect(() => {
    void load();
  }, []);

  return (
    <div>
      <h2 className="text-xl font-bold mb-3">{t("onboarding.microphone.title")}</h2>
      <p className="text-sm text-gray-400 mb-4">
        {t("onboarding.microphone.body")}
      </p>
      <select
        className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-2 py-1 mb-6"
        onChange={async (e) => {
          const cfg = await getConfig();
          await saveConfig({
            ...cfg,
            audio: { ...cfg.audio, mic_device: e.target.value },
          });
        }}
      >
        {devices.map((d) => (
          <option key={d.id} value={d.id}>
            {d.name}
          </option>
        ))}
      </select>
      <button
        onClick={onContinue}
        className="px-6 py-2 bg-[var(--accent-purple,#7c3aed)] text-white rounded"
      >
        {t("onboarding.welcome.continue")}
      </button>
    </div>
  );
}
