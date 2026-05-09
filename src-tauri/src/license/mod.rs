pub mod metering;
pub mod storage;
pub mod verify;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Free,
    Pro,
    Ultra,
}

impl Default for Tier {
    fn default() -> Self {
        Tier::Free
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPlan {
    pub tier: Tier,
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

impl Default for UserPlan {
    fn default() -> Self {
        Self::free_default()
    }
}

impl UserPlan {
    pub fn free_default() -> Self {
        Self {
            tier: Tier::Free,
            monthly_quota_seconds: 600,
            used_this_month_seconds: 0,
            overage_rate_per_min_cents: 50,
            resume_rag_enabled: false,
            resume_credits_remaining: 0,
            byo_active: false,
            auto_topup_enabled: false,
            history_persistence_days: 7,
            renews_at: None,
            cancelled_at: None,
        }
    }

    pub fn quota_remaining_seconds(&self) -> i64 {
        (self.monthly_quota_seconds - self.used_this_month_seconds).max(0)
    }
}
