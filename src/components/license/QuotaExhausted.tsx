import { useTranslation } from "react-i18next";

export function QuotaExhausted({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
      <div className="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-6 max-w-sm">
        <h2 className="text-lg font-bold mb-3">{t("billing.quotaExhausted")}</h2>
        <p className="text-sm text-gray-300 mb-4">{t("billing.quotaExhaustedBody")}</p>
        <a
          href="https://confide.knosi.xyz/pricing"
          target="_blank"
          rel="noreferrer"
          className="block px-4 py-2 bg-[var(--accent-purple,#7c3aed)] text-white rounded text-sm text-center mb-2"
        >
          {t("billing.upgrade")}
        </a>
        <button
          onClick={onClose}
          className="block w-full text-xs text-gray-400 mt-2"
        >
          {t("billing.close")}
        </button>
      </div>
    </div>
  );
}
