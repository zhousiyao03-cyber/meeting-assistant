import { useTranslation } from "react-i18next";
import { setUiLanguage } from "../../i18n/config";

export function LanguageSettings() {
  const { t, i18n } = useTranslation();

  return (
    <div className="space-y-4">
      <div>
        <label className="text-sm font-medium block mb-1">
          {t("settings.language.appLanguage")}
        </label>
        <p className="text-xs text-gray-500 mb-2">
          {t("settings.language.appLanguageDesc")}
        </p>
        <select
          value={i18n.language}
          onChange={(e) =>
            setUiLanguage(e.target.value as "zh-CN" | "en-US")
          }
          className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-2 py-1"
        >
          <option value="en-US">English</option>
          <option value="zh-CN">中文</option>
        </select>
      </div>
    </div>
  );
}
