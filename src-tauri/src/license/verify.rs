use anyhow::Result;
use serde::Deserialize;

use super::{Tier, UserPlan};

const API_BASE: &str = "https://api.confide.knosi.xyz";

#[derive(Deserialize)]
struct PlanResponse {
    pub tier: String,
    pub monthly_quota_seconds: i64,
    pub used_this_month_seconds: i64,
    pub overage_rate_per_min_cents: i32,
    pub resume_rag_enabled: bool,
    pub resume_credits_remaining: i32,
    pub byo_active: bool,
    pub auto_topup_enabled: bool,
    pub history_persistence_days: i32,
    pub renews_at: Option<i64>,
    pub cancelled_at: Option<i64>,
}

pub async fn fetch_plan(key: &str) -> Result<UserPlan> {
    let url = format!("{}/plan/{}", API_BASE, urlencoding::encode(key));
    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(anyhow::anyhow!("License not found"));
    }
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("API error: {}", resp.status()));
    }
    let p: PlanResponse = resp.json().await?;
    let tier = match p.tier.as_str() {
        "pro" => Tier::Pro,
        "ultra" => Tier::Ultra,
        _ => Tier::Free,
    };
    Ok(UserPlan {
        tier,
        monthly_quota_seconds: p.monthly_quota_seconds,
        used_this_month_seconds: p.used_this_month_seconds,
        overage_rate_per_min_cents: p.overage_rate_per_min_cents,
        resume_rag_enabled: p.resume_rag_enabled,
        resume_credits_remaining: p.resume_credits_remaining,
        byo_active: p.byo_active,
        auto_topup_enabled: p.auto_topup_enabled,
        history_persistence_days: p.history_persistence_days,
        renews_at: p.renews_at,
        cancelled_at: p.cancelled_at,
    })
}
