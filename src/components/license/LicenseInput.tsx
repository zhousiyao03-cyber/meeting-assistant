import { useState } from "react";
import { useTranslation } from "react-i18next";
import { setLicenseKey, clearLicenseKey } from "../../lib/tauri";
import type { UserPlan } from "../../lib/types";

interface Props {
  currentPlan: UserPlan;
  onUpdated: (p: UserPlan) => void;
}

export function LicenseInput({ currentPlan, onUpdated }: Props) {
  const { t } = useTranslation();
  const [key, setKey] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function activate() {
    setLoading(true);
    setError(null);
    try {
      const p = await setLicenseKey(key.trim());
      onUpdated(p);
      setKey("");
    } catch (e: any) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="space-y-3">
      <div className="text-sm text-gray-400">
        {t("license.currentPlan")}:{" "}
        <b>{currentPlan.tier.toUpperCase()}</b>
      </div>
      {currentPlan.tier === "free" ? (
        <>
          <input
            type="text"
            placeholder={t("license.placeholder")}
            className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-3 py-2 text-sm font-mono"
            value={key}
            onChange={(e) => setKey(e.target.value)}
          />
          <button
            onClick={() => void activate()}
            disabled={loading || key.length < 20}
            className="px-4 py-2 bg-[var(--accent-purple,#7c3aed)] text-white rounded text-sm disabled:opacity-40"
          >
            {loading ? t("license.activating") : t("license.activate")}
          </button>
          <a
            href="https://confide.knosi.xyz/pricing"
            className="text-xs text-blue-400 underline block"
            target="_blank"
            rel="noreferrer"
          >
            {t("license.noLicense")}
          </a>
        </>
      ) : (
        <button
          onClick={async () => {
            await clearLicenseKey();
            location.reload();
          }}
          className="text-xs text-red-400 underline"
        >
          {t("license.signOut")}
        </button>
      )}
      {error && <div className="text-xs text-red-400">{error}</div>}
    </div>
  );
}
