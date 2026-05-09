export type Tier = "free" | "pro" | "ultra";

export interface PlanConfig {
  lemonVariantId: string | null;
  priceUsd: number;
  monthlyQuotaSeconds: number;
  overageRatePerMinCents: number;
  resumeRagEnabled: boolean;
  resumeOptimizationCredits: number;
  historyPersistenceDays: number; // -1 = forever
}

export const PLAN_CATALOG: Record<Tier, PlanConfig> = {
  free: {
    lemonVariantId: null,
    priceUsd: 0,
    monthlyQuotaSeconds: 600,
    overageRatePerMinCents: 50,
    resumeRagEnabled: false,
    resumeOptimizationCredits: 0,
    historyPersistenceDays: 7,
  },
  pro: {
    // Replace via decision-log.md Week 0 Task 8
    lemonVariantId: "REPLACE_WITH_PRO_VARIANT_ID",
    priceUsd: 19,
    monthlyQuotaSeconds: 3600,
    overageRatePerMinCents: 35,
    resumeRagEnabled: true,
    resumeOptimizationCredits: 5,
    historyPersistenceDays: -1,
  },
  ultra: {
    lemonVariantId: "REPLACE_WITH_ULTRA_VARIANT_ID",
    priceUsd: 49,
    monthlyQuotaSeconds: 12000,
    overageRatePerMinCents: 25,
    resumeRagEnabled: true,
    resumeOptimizationCredits: 15,
    historyPersistenceDays: -1,
  },
};

export interface License {
  email: string;
  tier: Tier;
  used_this_month_seconds: number;
  resume_optimization_credits_used: number;
  byo_active: boolean;
  auto_topup_enabled: boolean;
  created_at: number;
  renews_at: number | null;
  cancelled_at: number | null;
  device_fingerprints: string[];
  revoked: boolean;
  locale?: "zh-CN" | "en-US";
}

export function newFreeLicense(email: string, locale: "zh-CN" | "en-US" = "en-US"): License {
  return {
    email,
    tier: "free",
    used_this_month_seconds: 0,
    resume_optimization_credits_used: 0,
    byo_active: false,
    auto_topup_enabled: false,
    created_at: Date.now(),
    renews_at: null,
    cancelled_at: null,
    device_fingerprints: [],
    revoked: false,
    locale,
  };
}

export function generateLicenseKey(): string {
  const year = new Date().getFullYear();
  const rand = () => {
    const bytes = crypto.getRandomValues(new Uint8Array(2));
    return Array.from(bytes)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("")
      .toUpperCase();
  };
  const a = rand();
  const b = rand();
  const c = rand();
  const allChars = `${a}${b}${c}`;
  let sum = 0;
  for (let i = 0; i < allChars.length; i++) sum ^= allChars.charCodeAt(i);
  const checksum = sum.toString(36).toUpperCase().padStart(2, "0").slice(0, 2);
  return `confide-${year}-${a}-${b}-${c}-${checksum}`;
}
