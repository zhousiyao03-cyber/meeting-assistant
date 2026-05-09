import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import zhCN from "./locales/zh-CN.json";
import enUS from "./locales/en-US.json";

const STORAGE_KEY = "confide.uiLang";

function detectInitialLanguage(): string {
  if (typeof localStorage !== "undefined") {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "zh-CN" || stored === "en-US") return stored;
  }
  if (typeof navigator !== "undefined") {
    const sys = navigator.language.toLowerCase();
    if (sys.startsWith("zh")) return "zh-CN";
  }
  return "en-US";
}

void i18n.use(initReactI18next).init({
  resources: {
    "zh-CN": { translation: zhCN },
    "en-US": { translation: enUS },
  },
  lng: detectInitialLanguage(),
  fallbackLng: "en-US",
  interpolation: { escapeValue: false },
});

export default i18n;

export function setUiLanguage(lng: "zh-CN" | "en-US") {
  void i18n.changeLanguage(lng);
  if (typeof localStorage !== "undefined") {
    localStorage.setItem(STORAGE_KEY, lng);
  }
}
