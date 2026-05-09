import { Env } from "./env";
import { License, PLAN_CATALOG, Tier } from "./plans";

export async function getLicense(env: Env, key: string): Promise<License | null> {
  return await env.CONFIDE_LICENSES.get<License>(`license:${key}`, "json");
}

export async function putLicense(env: Env, key: string, license: License): Promise<void> {
  await env.CONFIDE_LICENSES.put(`license:${key}`, JSON.stringify(license));
}

export async function getKeyByEmail(env: Env, email: string): Promise<string | null> {
  return await env.CONFIDE_LICENSES.get(`email:${email.toLowerCase()}`);
}

export async function setKeyForEmail(env: Env, email: string, key: string): Promise<void> {
  await env.CONFIDE_LICENSES.put(`email:${email.toLowerCase()}`, key);
}

export interface PlanInfo {
  tier: Tier;
  monthly_quota_seconds: number;
  used_this_month_seconds: number;
  overage_rate_per_min_cents: number;
  resume_rag_enabled: boolean;
  resume_credits_remaining: number;
  byo_active: boolean;
  auto_topup_enabled: boolean;
  history_persistence_days: number;
  renews_at: number | null;
  cancelled_at: number | null;
}

export function planInfoFromLicense(license: License): PlanInfo {
  const cfg = PLAN_CATALOG[license.tier];
  return {
    tier: license.tier,
    monthly_quota_seconds: cfg.monthlyQuotaSeconds,
    used_this_month_seconds: license.used_this_month_seconds,
    overage_rate_per_min_cents: cfg.overageRatePerMinCents,
    resume_rag_enabled: cfg.resumeRagEnabled,
    resume_credits_remaining: cfg.resumeOptimizationCredits - license.resume_optimization_credits_used,
    byo_active: license.byo_active,
    auto_topup_enabled: license.auto_topup_enabled,
    history_persistence_days: cfg.historyPersistenceDays,
    renews_at: license.renews_at,
    cancelled_at: license.cancelled_at,
  };
}
