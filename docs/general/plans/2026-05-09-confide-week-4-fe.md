# Confide Week 4 — License + Lemon Squeezy + Monthly Quota

> **For agentic workers:** REQUIRED SUB-SKILL: Use gecc-dev:subagent-driven-development (recommended) or gecc-dev:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 注册 → Free 10min/月 quota → 升级 Pro $19 → 收 license email → 录音中 5 分钟 sync 一次 → 月度续订重置 quota。

**Domain:** general

**Architecture:**
- 客户端 LicenseManager + UserPlan 数据结构（Section 6.2）
- Cloudflare Workers 后端（Hono）部署到 `api.confide.knosi.xyz`
- KV 存 license / plan，D1 存 usage_events / lemonsqueezy_events
- Lemon Squeezy webhook 处理订阅生命周期
- 5 分钟 sync 计量 + 7 天离线缓存

**Tech Stack:** Cloudflare Workers + Hono + KV + D1 + wrangler、Rust `keyring` crate（license key 持久化）

**Spec reference:** `docs/specs/2026-05-09-overseas-meeting-copilot-design.md` Section 6

**Prerequisite:** Week 3 完成；Week 0 Lemon Squeezy 注册 + 创建 Pro/Ultra variant + 拿 webhook secret + Cloudflare wrangler 登录

---

## File Structure

```
workers/                                    [Create dir at repo root]
├── package.json                            [Create]
├── wrangler.toml                           [Create] CF Workers 配置
├── src/
│   ├── index.ts                            [Create] Hono router + 5 endpoints
│   ├── plans.ts                            [Create] PLAN_CATALOG
│   ├── license.ts                          [Create] verify / balance logic
│   ├── usage.ts                            [Create] /usage endpoint
│   ├── webhook.ts                          [Create] Lemon Squeezy webhook handler
│   └── env.d.ts                            [Create] CF Bindings 类型
└── schema.sql                              [Create] D1 schema

src-tauri/
├── Cargo.toml                              [Modify] 加 keyring + reqwest 已有
└── src/
    ├── license/                            [Create dir]
    │   ├── mod.rs                          [Create] LicenseManager + UserPlan
    │   ├── verify.rs                       [Create] HTTP client to /verify-license
    │   ├── metering.rs                     [Create] Meter (5 分钟 sync)
    │   └── storage.rs                      [Create] keyring + cache file
    ├── commands.rs                         [Modify] 加 license commands + meter into start_recording
    ├── main.rs                             [Modify] 加 mod license; spawn meter
    └── lib.rs                              [Modify] pub mod license

src/
├── components/
│   └── license/
│       ├── LicenseInput.tsx                [Create] 输入 license key
│       ├── PlanBadge.tsx                   [Create] 顶部显示当前 plan + quota
│       └── QuotaExhausted.tsx              [Create] 含量用完弹窗
└── lib/tauri.ts                            [Modify] license wrappers
```

---

### Task 1: 创建 Workers 项目

**Files:**
- Create: `workers/package.json`
- Create: `workers/wrangler.toml`
- Create: `workers/schema.sql`

- [ ] **Step 1: 初始化项目**

Run:
```bash
cd /Users/bytedance/meeting-assistant
mkdir -p workers/src
cd workers
```

写 `workers/package.json`:
```json
{
  "name": "confide-workers",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "wrangler dev",
    "deploy": "wrangler deploy",
    "tail": "wrangler tail",
    "typecheck": "tsc --noEmit"
  },
  "dependencies": {
    "hono": "^4.6.0"
  },
  "devDependencies": {
    "@cloudflare/workers-types": "^4.20240620.0",
    "typescript": "^5.6.0",
    "wrangler": "^4.0.0"
  }
}
```

- [ ] **Step 2: 安装依赖**

```bash
cd workers && pnpm install
```

- [ ] **Step 3: 写 wrangler.toml**

```toml
name = "confide-api"
main = "src/index.ts"
compatibility_date = "2026-05-09"
compatibility_flags = ["nodejs_compat"]

# Production
[env.production]
name = "confide-api"
route = { pattern = "api.confide.knosi.xyz/*", custom_domain = true }

[env.production.vars]
ENVIRONMENT = "production"

[[env.production.kv_namespaces]]
binding = "CONFIDE_LICENSES"
id = "<from Week 0 Task 10 decision-log>"

[[env.production.d1_databases]]
binding = "DB"
database_name = "confide-events"
database_id = "<from Week 0 Task 10 decision-log>"

# Dev (uses --remote flag with wrangler dev to hit real KV/D1)
[env.dev]
name = "confide-api-dev"

[[env.dev.kv_namespaces]]
binding = "CONFIDE_LICENSES"
id = "<preview KV id>"
preview_id = "<preview KV id>"

[[env.dev.d1_databases]]
binding = "DB"
database_name = "confide-events"
database_id = "<dev D1 id, can reuse production for MVP>"
```

注：填实际 ID 时从 `decision-log.md` Week 0 Task 10 取。

- [ ] **Step 4: 写 D1 schema**

`workers/schema.sql`:
```sql
CREATE TABLE IF NOT EXISTS usage_events (
  event_id      TEXT PRIMARY KEY,
  license_key   TEXT NOT NULL,
  provider      TEXT NOT NULL,
  seconds       REAL NOT NULL,
  ts            INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_usage_license_ts ON usage_events(license_key, ts);

CREATE TABLE IF NOT EXISTS verify_log (
  license_key   TEXT NOT NULL,
  device_id     TEXT NOT NULL,
  ts            INTEGER NOT NULL,
  PRIMARY KEY (license_key, device_id, ts)
);

CREATE TABLE IF NOT EXISTS lemonsqueezy_events (
  id            TEXT PRIMARY KEY,
  license_key   TEXT,
  event_type    TEXT NOT NULL,
  amount_cents  INTEGER,
  ts            INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_lemon_license_ts ON lemonsqueezy_events(license_key, ts);
```

- [ ] **Step 5: 部署 schema 到 D1**

Run（**需要 wrangler login，假设 Week 0 Task 10 已完成**）:
```bash
cd workers
wrangler d1 execute confide-events --file=schema.sql --remote
```

Expected: `🌀 Executing on remote database confide-events ... ✅ executed`。如果错 wrangler 没登录：先跑 `wrangler login`。

---

### Task 2: 写 Workers TS 代码

**Files:**
- Create: `workers/src/env.d.ts`
- Create: `workers/src/plans.ts`
- Create: `workers/src/index.ts`
- Create: `workers/src/license.ts`
- Create: `workers/src/usage.ts`
- Create: `workers/src/webhook.ts`

- [ ] **Step 1: env.d.ts (CF bindings 类型)**

```typescript
export interface Env {
  CONFIDE_LICENSES: KVNamespace;
  DB: D1Database;
  ENVIRONMENT: string;

  // Secrets (set via wrangler secret put)
  LEMONSQUEEZY_WEBHOOK_SECRET: string;
  LEMONSQUEEZY_API_KEY: string;
  ANTHROPIC_API_KEY: string;
  OPENAI_API_KEY: string;
}
```

- [ ] **Step 2: plans.ts (PLAN_CATALOG)**

```typescript
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
}

export function newFreeLicense(email: string): License {
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
  };
}

export function generateLicenseKey(): string {
  const year = new Date().getFullYear();
  const rand = () => {
    const bytes = crypto.getRandomValues(new Uint8Array(2));
    return Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("").toUpperCase();
  };
  const a = rand();
  const b = rand();
  const c = rand();
  // Simple checksum: XOR of all chars mod 36 → base36
  const allChars = `${a}${b}${c}`;
  const sum = Array.from(allChars).reduce((acc, ch) => acc ^ ch.charCodeAt(0), 0);
  const checksum = sum.toString(36).toUpperCase().padStart(2, "0").slice(0, 2);
  return `confide-${year}-${a}-${b}-${c}-${checksum}`;
}
```

- [ ] **Step 3: license.ts**

```typescript
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
```

- [ ] **Step 4: usage.ts**

```typescript
import { Env } from "./env";
import { getLicense, putLicense } from "./license";

export interface UsageEvent {
  event_id: string;
  meeting_id: string;
  provider: string;       // "confide" | "byo-openai" | "byo-anthropic"
  seconds_used: number;
  started_at: number;
  ended_at: number;
}

export async function recordUsage(
  env: Env,
  key: string,
  events: UsageEvent[],
): Promise<{ accepted: number; deduped: number }> {
  let accepted = 0;
  let deduped = 0;
  const license = await getLicense(env, key);
  if (!license) throw new Error("license not found");

  for (const evt of events) {
    // Idempotency: skip if event_id seen
    const seen = await env.CONFIDE_LICENSES.get(`event:${evt.event_id}`);
    if (seen) { deduped++; continue; }

    if (evt.provider === "confide") {
      // Counts toward used_this_month_seconds
      license.used_this_month_seconds += evt.seconds_used;
    }
    // BYO providers don't count toward quota

    // Append to D1
    await env.DB.prepare(
      "INSERT INTO usage_events (event_id, license_key, provider, seconds, ts) VALUES (?, ?, ?, ?, ?)"
    ).bind(evt.event_id, key, evt.provider, evt.seconds_used, evt.ended_at).run();

    await env.CONFIDE_LICENSES.put(`event:${evt.event_id}`, "1", { expirationTtl: 86400 * 30 });
    accepted++;
  }

  await putLicense(env, key, license);
  return { accepted, deduped };
}
```

- [ ] **Step 5: webhook.ts**

```typescript
import { Env } from "./env";
import {
  License,
  newFreeLicense,
  generateLicenseKey,
  PLAN_CATALOG,
  Tier,
} from "./plans";
import { getLicense, getKeyByEmail, putLicense, setKeyForEmail } from "./license";

/// Verify Lemon Squeezy webhook signature.
/// Reference: https://docs.lemonsqueezy.com/help/webhooks
async function verifySignature(
  rawBody: string,
  signatureHeader: string | null,
  secret: string,
): Promise<boolean> {
  if (!signatureHeader) return false;
  const enc = new TextEncoder();
  const key = await crypto.subtle.importKey(
    "raw",
    enc.encode(secret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  const sig = await crypto.subtle.sign("HMAC", key, enc.encode(rawBody));
  const hex = Array.from(new Uint8Array(sig))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  // Constant-time compare
  if (hex.length !== signatureHeader.length) return false;
  let diff = 0;
  for (let i = 0; i < hex.length; i++) {
    diff |= hex.charCodeAt(i) ^ signatureHeader.charCodeAt(i);
  }
  return diff === 0;
}

/// Determine which Tier a Lemon Squeezy variant_id corresponds to.
function tierFromVariantId(variantId: string): Tier | null {
  for (const [tier, cfg] of Object.entries(PLAN_CATALOG)) {
    if (cfg.lemonVariantId === variantId) return tier as Tier;
  }
  return null;
}

export async function handleWebhook(
  env: Env,
  rawBody: string,
  signatureHeader: string | null,
): Promise<Response> {
  if (!await verifySignature(rawBody, signatureHeader, env.LEMONSQUEEZY_WEBHOOK_SECRET)) {
    return new Response("invalid signature", { status: 401 });
  }

  const event = JSON.parse(rawBody);
  const eventName = event.meta?.event_name as string;
  const data = event.data;

  // Most relevant events all carry attributes.user_email and attributes.variant_id (for subs)
  const email = data?.attributes?.user_email as string | undefined;
  const variantId = String(data?.attributes?.variant_id ?? "");
  const eventId = String(event.meta?.event_id ?? crypto.randomUUID());

  // Log every event
  await env.DB.prepare(
    "INSERT INTO lemonsqueezy_events (id, license_key, event_type, amount_cents, ts) VALUES (?, ?, ?, ?, ?)"
  ).bind(eventId, "", eventName, data?.attributes?.total ?? null, Date.now()).run().catch(() => {});

  if (!email) {
    return new Response("ok (no email, ignored)", { status: 200 });
  }

  // Find or create license
  let key = await getKeyByEmail(env, email);
  let license: License | null = key ? await getLicense(env, key) : null;
  if (!license) {
    key = generateLicenseKey();
    license = newFreeLicense(email);
    await setKeyForEmail(env, email, key);
  }

  switch (eventName) {
    case "subscription_created":
    case "subscription_payment_success": {
      const tier = tierFromVariantId(variantId);
      if (tier) {
        license.tier = tier;
        license.used_this_month_seconds = 0;
        license.resume_optimization_credits_used = 0;
        license.cancelled_at = null;
        license.revoked = false;
        license.renews_at = data.attributes.renews_at
          ? new Date(data.attributes.renews_at).getTime()
          : null;
      }
      break;
    }
    case "subscription_cancelled": {
      license.cancelled_at = Date.now();
      // Keep tier active until renews_at; cron job (future) downgrades
      break;
    }
    case "subscription_expired":
    case "subscription_payment_failed": {
      license.tier = "free";
      license.cancelled_at = Date.now();
      license.renews_at = null;
      break;
    }
    case "order_refunded": {
      license.revoked = true;
      break;
    }
  }

  await putLicense(env, key!, license);

  // Send license key email on first creation (Resend)
  if (eventName === "subscription_created") {
    await sendLicenseEmail(env, email, key!, license.tier);
  }

  return new Response("ok", { status: 200 });
}

async function sendLicenseEmail(env: Env, email: string, key: string, tier: Tier): Promise<void> {
  // MVP: skip Resend integration here; Lemon Squeezy already sends order receipt.
  // Week 5 Task: implement Resend bilingual templates.
  console.log(`[webhook] Would send license email to ${email}: tier=${tier}, key=${key}`);
}
```

- [ ] **Step 6: index.ts (Hono router)**

```typescript
import { Hono } from "hono";
import { cors } from "hono/cors";
import { Env } from "./env";
import { getLicense, planInfoFromLicense } from "./license";
import { recordUsage } from "./usage";
import { handleWebhook } from "./webhook";

const app = new Hono<{ Bindings: Env }>();

app.use("/*", cors({
  origin: ["tauri://localhost", "http://localhost:1420"],
  allowMethods: ["GET", "POST"],
  allowHeaders: ["Content-Type", "Authorization"],
}));

app.get("/", (c) => c.text("Confide API"));

app.get("/plan/:key", async (c) => {
  const license = await getLicense(c.env, c.req.param("key"));
  if (!license) return c.json({ error: "not_found" }, 404);
  if (license.revoked) return c.json({ error: "revoked" }, 403);
  return c.json(planInfoFromLicense(license));
});

app.post("/usage", async (c) => {
  const { key, events } = await c.req.json();
  if (typeof key !== "string" || !Array.isArray(events)) {
    return c.json({ error: "bad_request" }, 400);
  }
  try {
    const r = await recordUsage(c.env, key, events);
    const license = await getLicense(c.env, key);
    return c.json({ ...r, plan: license ? planInfoFromLicense(license) : null });
  } catch (e: any) {
    return c.json({ error: e.message }, 500);
  }
});

app.post("/lemonsqueezy-webhook", async (c) => {
  const rawBody = await c.req.text();
  const signature = c.req.header("X-Signature");
  return handleWebhook(c.env, rawBody, signature);
});

app.post("/recover-license", async (c) => {
  const { email } = await c.req.json();
  if (typeof email !== "string") return c.json({ error: "bad_request" }, 400);
  const { getKeyByEmail } = await import("./license");
  const key = await getKeyByEmail(c.env, email);
  if (!key) return c.json({ error: "not_found" }, 404);
  // MVP: Resend integration deferred to Week 5
  console.log(`[recover] license recovery requested for ${email}: ${key}`);
  return c.json({ ok: true });
});

export default app;
```

- [ ] **Step 7: typecheck**

`workers/tsconfig.json`:
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ES2022",
    "moduleResolution": "Bundler",
    "lib": ["ES2022"],
    "types": ["@cloudflare/workers-types"],
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src/**/*"]
}
```

Run:
```bash
cd /Users/bytedance/meeting-assistant/workers
pnpm typecheck 2>&1 | tail -10
```

Expected: 0 errors。

---

### Task 3: 部署 Workers + 配置 secrets

**Files:** 无（CLI 操作）

- [ ] **Step 1: 设置 secrets**

```bash
cd /Users/bytedance/meeting-assistant/workers
wrangler secret put LEMONSQUEEZY_WEBHOOK_SECRET --env production
# 粘贴 Week 0 Task 8 拿到的 webhook signing secret

wrangler secret put LEMONSQUEEZY_API_KEY --env production
# 暂留空（v1.0.5 用于 auto top-up）

wrangler secret put ANTHROPIC_API_KEY --env production
# Week 0 Task 5 拿到的 Anthropic key

wrangler secret put OPENAI_API_KEY --env production
# Week 0 Task 6 拿到的 OpenAI key
```

- [ ] **Step 2: deploy**

```bash
wrangler deploy --env production
```

Expected: 输出 `Deployed confide-api ... https://api.confide.knosi.xyz/...`。

- [ ] **Step 3: 验证 deployment**

```bash
curl https://api.confide.knosi.xyz/
```

Expected: `Confide API`

- [ ] **Step 4: 在 Lemon Squeezy 改 webhook URL**

去 Lemon Squeezy Dashboard > Settings > Webhooks → 编辑刚才的 webhook → URL 改成 `https://api.confide.knosi.xyz/lemonsqueezy-webhook` → 保存。

---

### Task 4: 客户端 LicenseManager Rust 实现

**Files:**
- Create: `src-tauri/src/license/mod.rs`
- Create: `src-tauri/src/license/storage.rs`
- Create: `src-tauri/src/license/verify.rs`
- Create: `src-tauri/src/license/metering.rs`
- Modify: `src-tauri/Cargo.toml`（加 keyring）

- [ ] **Step 1: 加 keyring 依赖**

```toml
# Week 4: license key persistence in macOS keychain
keyring = "3"
```

- [ ] **Step 2: 写 license/mod.rs**

```rust
pub mod storage;
pub mod verify;
pub mod metering;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Tier { Free, Pro, Ultra }

impl Default for Tier {
    fn default() -> Self { Tier::Free }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
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
```

- [ ] **Step 3: 写 license/storage.rs**

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

const KEYRING_SERVICE: &str = "app.voicenote.confide";
const KEYRING_USER: &str = "license_key";

pub fn get_license_key() -> Result<Option<String>> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    match entry.get_password() {
        Ok(s) => Ok(Some(s)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn set_license_key(key: &str) -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    entry.set_password(key)?;
    Ok(())
}

pub fn clear_license_key() -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    let _ = entry.delete_credential();
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CachedPlan {
    pub plan: super::UserPlan,
    pub cached_at: i64,
    pub pending_usage: Vec<super::metering::UsageEvent>,
}

fn cache_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home dir"))?;
    let dir = home.join(".meeting-assistant");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("license-cache.json"))
}

pub fn load_cached() -> Result<Option<CachedPlan>> {
    let path = cache_path()?;
    if !path.exists() { return Ok(None); }
    let s = fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&s)?))
}

pub fn save_cached(c: &CachedPlan) -> Result<()> {
    let path = cache_path()?;
    fs::write(path, serde_json::to_string_pretty(c)?)?;
    Ok(())
}
```

- [ ] **Step 4: 写 license/verify.rs**

```rust
use anyhow::Result;
use serde::Deserialize;
use super::UserPlan;

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
        "pro" => super::Tier::Pro,
        "ultra" => super::Tier::Ultra,
        _ => super::Tier::Free,
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
```

加 `urlencoding` 依赖到 Cargo.toml：
```toml
urlencoding = "2"
```

- [ ] **Step 5: 写 license/metering.rs**

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

const API_BASE: &str = "https://api.confide.knosi.xyz";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UsageEvent {
    pub event_id: String,
    pub meeting_id: String,
    pub provider: String,    // "confide" | "byo-openai" | "byo-anthropic"
    pub seconds_used: f64,
    pub started_at: i64,
    pub ended_at: i64,
}

pub struct Meter {
    pub meeting_id: String,
    pub provider: String,
    pub started_at: SystemTime,
    pub last_sync_at: SystemTime,
    pub accumulated_seconds: f64,
}

impl Meter {
    pub fn new(meeting_id: String, provider: String) -> Self {
        let now = SystemTime::now();
        Self {
            meeting_id,
            provider,
            started_at: now,
            last_sync_at: now,
            accumulated_seconds: 0.0,
        }
    }

    /// Returns Some(UsageEvent) if a sync is due (5 minutes elapsed since last sync).
    pub fn maybe_create_event(&mut self) -> Option<UsageEvent> {
        let total_elapsed = self.started_at.elapsed().ok()?.as_secs_f64();
        let unsynced = total_elapsed - self.accumulated_seconds;
        if unsynced < 300.0 { return None; }

        let now = SystemTime::now();
        let evt = UsageEvent {
            event_id: format!("{}-{}", self.meeting_id, uuid::Uuid::new_v4()),
            meeting_id: self.meeting_id.clone(),
            provider: self.provider.clone(),
            seconds_used: unsynced,
            started_at: self.last_sync_at.duration_since(SystemTime::UNIX_EPOCH).ok()?.as_secs() as i64,
            ended_at: now.duration_since(SystemTime::UNIX_EPOCH).ok()?.as_secs() as i64,
        };
        self.accumulated_seconds += unsynced;
        self.last_sync_at = now;
        Some(evt)
    }

    pub fn create_final_event(&mut self) -> Option<UsageEvent> {
        let total_elapsed = self.started_at.elapsed().ok()?.as_secs_f64();
        let unsynced = total_elapsed - self.accumulated_seconds;
        if unsynced < 1.0 { return None; }
        let now = SystemTime::now();
        let evt = UsageEvent {
            event_id: format!("{}-{}", self.meeting_id, uuid::Uuid::new_v4()),
            meeting_id: self.meeting_id.clone(),
            provider: self.provider.clone(),
            seconds_used: unsynced,
            started_at: self.last_sync_at.duration_since(SystemTime::UNIX_EPOCH).ok()?.as_secs() as i64,
            ended_at: now.duration_since(SystemTime::UNIX_EPOCH).ok()?.as_secs() as i64,
        };
        self.accumulated_seconds += unsynced;
        Some(evt)
    }
}

pub async fn sync_usage(key: &str, events: Vec<UsageEvent>) -> Result<()> {
    if events.is_empty() { return Ok(()); }
    let url = format!("{}/usage", API_BASE);
    let body = serde_json::json!({ "key": key, "events": events });
    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&body).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("Usage sync failed: {}", resp.status()));
    }
    Ok(())
}
```

- [ ] **Step 6: 在 lib.rs / main.rs 暴露**

`src-tauri/src/lib.rs`: `pub mod license;`
`src-tauri/src/main.rs`: `mod license;`

- [ ] **Step 7: 编译验证**

```bash
cd /Users/bytedance/meeting-assistant
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 通过。如果错 keyring 平台依赖：keyring 3.x 需要 macOS Keychain entitlements，开发模式应该 OK。

---

### Task 5: 接 license 到 Tauri commands + start_recording 加 meter

**Files:**
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: 加 license commands**

文件末尾加：

```rust
// --- License ---

use crate::license::{self, UserPlan};

#[command]
pub async fn get_user_plan() -> Result<UserPlan, String> {
    let key = license::storage::get_license_key().map_err(|e| e.to_string())?;
    if let Some(k) = key {
        match license::verify::fetch_plan(&k).await {
            Ok(p) => {
                let cached = license::storage::CachedPlan {
                    plan: p.clone(),
                    cached_at: chrono::Utc::now().timestamp(),
                    pending_usage: vec![],
                };
                let _ = license::storage::save_cached(&cached);
                Ok(p)
            }
            Err(e) => {
                // Fall back to cached
                if let Ok(Some(c)) = license::storage::load_cached() {
                    let age_days = (chrono::Utc::now().timestamp() - c.cached_at) / 86400;
                    if age_days <= 7 {
                        return Ok(c.plan);
                    }
                }
                Err(format!("Cannot verify license: {}", e))
            }
        }
    } else {
        Ok(UserPlan::free_default())
    }
}

#[command]
pub async fn set_license_key(key: String) -> Result<UserPlan, String> {
    license::storage::set_license_key(&key).map_err(|e| e.to_string())?;
    let plan = license::verify::fetch_plan(&key).await.map_err(|e| e.to_string())?;
    Ok(plan)
}

#[command]
pub async fn clear_license_key() -> Result<(), String> {
    license::storage::clear_license_key().map_err(|e| e.to_string())
}
```

- [ ] **Step 2: 在 main.rs 注册 3 个新 command**

```rust
            commands::get_user_plan,
            commands::set_license_key,
            commands::clear_license_key,
```

- [ ] **Step 3: 在 start_recording 加 meter spawn**

在 Week 1 修改的 start_recording 函数里，**ASR loop 之外、advisor loop 之前**加 meter loop：

```rust
// === Meter loop: 5-minute sync to Confide cloud ===
let state_for_meter: SharedRecordingState = Arc::clone(&state);
let win_for_meter = window.clone();
tokio::spawn(async move {
    let key = match license::storage::get_license_key() {
        Ok(Some(k)) => k,
        _ => {
            eprintln!("[meter] No license key; skipping metering loop (free trial)");
            return;
        }
    };
    let meeting_id = uuid::Uuid::new_v4().to_string();
    let provider = "confide".to_string(); // BYO 模式 v1.0 以后再分
    let mut meter = license::metering::Meter::new(meeting_id, provider);

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

        let recording = {
            let rec = state_for_meter.lock().await;
            rec.is_recording
        };
        if !recording {
            // Final sync
            if let Some(evt) = meter.create_final_event() {
                let _ = license::metering::sync_usage(&key, vec![evt]).await;
            }
            break;
        }

        if let Some(evt) = meter.maybe_create_event() {
            match license::metering::sync_usage(&key, vec![evt.clone()]).await {
                Ok(()) => {
                    eprintln!("[meter] synced {} sec", evt.seconds_used);
                }
                Err(e) => {
                    eprintln!("[meter] sync failed (will retry): {}", e);
                    // TODO v1.0.5: persist to pending_usage cache for offline recovery
                }
            }
            // Refresh plan after sync
            match license::verify::fetch_plan(&key).await {
                Ok(plan) => {
                    let _ = win_for_meter.emit("plan-updated", &plan);
                    if plan.quota_remaining_seconds() < 60 {
                        let _ = win_for_meter.emit("quota-low", plan.quota_remaining_seconds());
                    }
                    if plan.quota_remaining_seconds() <= 0 && !plan.auto_topup_enabled {
                        let _ = win_for_meter.emit("quota-exhausted", ());
                        // Stop recording
                        let mut rec = state_for_meter.lock().await;
                        rec.is_recording = false;
                        break;
                    }
                }
                Err(e) => eprintln!("[meter] plan refresh failed: {}", e),
            }
        }
    }
});
```

- [ ] **Step 4: 编译验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 通过。

---

### Task 6: 前端 license UI

**Files:**
- Create: `src/components/license/LicenseInput.tsx`
- Create: `src/components/license/PlanBadge.tsx`
- Create: `src/components/license/QuotaExhausted.tsx`
- Modify: `src/lib/tauri.ts`
- Modify: `src/lib/types.ts`
- Modify: `src/components/settings/SettingsView.tsx`（加 License tab）

- [ ] **Step 1: 加 types**

```typescript
export type Tier = "free" | "pro" | "ultra";

export interface UserPlan {
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
```

- [ ] **Step 2: 加 wrappers**

```typescript
export const getUserPlan = () => invoke<UserPlan>("get_user_plan");
export const setLicenseKey = (key: string) =>
  invoke<UserPlan>("set_license_key", { key });
export const clearLicenseKey = () => invoke<void>("clear_license_key");

export const onPlanUpdated = (h: (p: UserPlan) => void) =>
  listen<UserPlan>("plan-updated", (e) => h(e.payload));
export const onQuotaLow = (h: (remaining: number) => void) =>
  listen<number>("quota-low", (e) => h(e.payload));
export const onQuotaExhausted = (h: () => void) =>
  listen("quota-exhausted", () => h());
```

- [ ] **Step 3: 写 LicenseInput.tsx**

```tsx
import { useState } from "react";
import { setLicenseKey, clearLicenseKey } from "../../lib/tauri";
import type { UserPlan } from "../../lib/types";

export function LicenseInput({ currentPlan, onUpdated }: {
  currentPlan: UserPlan;
  onUpdated: (p: UserPlan) => void;
}) {
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
        Current plan: <b>{currentPlan.tier.toUpperCase()}</b>
      </div>
      {currentPlan.tier === "free" ? (
        <>
          <input
            type="text"
            placeholder="confide-2026-XXXX-XXXX-XXXX-XX"
            className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-3 py-2 text-sm font-mono"
            value={key}
            onChange={(e) => setKey(e.target.value)}
          />
          <button
            onClick={() => void activate()}
            disabled={loading || key.length < 20}
            className="px-4 py-2 bg-[var(--accent-purple)] text-white rounded text-sm"
          >
            {loading ? "Activating…" : "Activate License"}
          </button>
          <a
            href="https://confide.knosi.xyz/pricing"
            className="text-xs text-blue-400 underline"
            target="_blank"
            rel="noreferrer"
          >
            Don't have a license? Get one →
          </a>
        </>
      ) : (
        <button
          onClick={() => void clearLicenseKey()}
          className="text-xs text-red-400 underline"
        >
          Sign out / Remove license
        </button>
      )}
      {error && <div className="text-xs text-red-400">{error}</div>}
    </div>
  );
}
```

- [ ] **Step 4: 写 PlanBadge.tsx**

```tsx
import type { UserPlan } from "../../lib/types";

export function PlanBadge({ plan }: { plan: UserPlan }) {
  const remaining = Math.max(0, plan.monthly_quota_seconds - plan.used_this_month_seconds);
  const minRemaining = Math.floor(remaining / 60);
  const totalMin = Math.floor(plan.monthly_quota_seconds / 60);

  return (
    <div className="text-xs flex items-center gap-2">
      <span className={`px-2 py-0.5 rounded ${
        plan.tier === "free" ? "bg-gray-700 text-gray-300"
        : plan.tier === "pro" ? "bg-purple-900 text-purple-200"
        : "bg-yellow-900 text-yellow-200"
      }`}>
        {plan.tier.toUpperCase()}
      </span>
      <span className="text-gray-400">{minRemaining}/{totalMin} min</span>
    </div>
  );
}
```

- [ ] **Step 5: 写 QuotaExhausted.tsx**

```tsx
export function QuotaExhausted({ onClose }: { onClose: () => void }) {
  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
      <div className="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-6 max-w-sm">
        <h2 className="text-lg font-bold mb-3">Monthly quota reached</h2>
        <p className="text-sm text-gray-300 mb-4">
          You've used your full monthly meeting time. Upgrade to continue:
        </p>
        <a
          href="https://confide.knosi.xyz/pricing"
          target="_blank"
          rel="noreferrer"
          className="block px-4 py-2 bg-[var(--accent-purple)] text-white rounded text-sm text-center mb-2"
        >
          Upgrade Plan
        </a>
        <button onClick={onClose} className="block w-full text-xs text-gray-400">
          Close
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 6: 在 SettingsView 加 License tab**

打开 `src/components/settings/SettingsView.tsx`，在 tabs 数组加 "license"。在对应 panel 渲染 `<LicenseInput>`。

- [ ] **Step 7: 在 NarrowView ControlBar 显示 PlanBadge**

打开 `src/components/narrow/ControlBar.tsx`，加：

```tsx
import { PlanBadge } from "../license/PlanBadge";

// 顶部加：
{plan && <PlanBadge plan={plan} />}
```

`plan` 通过 props 从 App.tsx 传下来。在 App.tsx 顶部 useState `<UserPlan>` 并 useEffect 调 getUserPlan() + onPlanUpdated。

- [ ] **Step 8: 编译验证**

```bash
pnpm typecheck 2>&1 | tail -5
```

---

### Task 7: 端到端验证

- [ ] **Step 1: deploy workers + 用 Lemon Squeezy test mode 走付费流程**

如果 Lemon Squeezy test mode 启用：用 4242 4242 4242 4242 信用卡。

订阅 Confide Pro $19 → 看 Lemon Squeezy webhook 触发 → `wrangler tail --env production` 应该看到 `subscription_created` event。

- [ ] **Step 2: 调 KV 验证 license 已创建**

```bash
wrangler kv key get --binding=CONFIDE_LICENSES "email:<your-email>" --env production --remote
```

Expected: 返回 license key。

- [ ] **Step 3: 客户端输入 license**

启动 Confide → Settings > License → 粘贴 key → Activate。

Expected: PlanBadge 从 "FREE 0/10" 变 "PRO 0/60"。

- [ ] **Step 4: 跑 6 分钟录音验证 sync**

打开任意会议录 6 分钟。

Expected:
- ~5 分钟时 Tauri terminal 出现 `[meter] synced X sec`
- PlanBadge 变 "PRO 5/60" 或类似

- [ ] **Step 5: 标 Week 4 完成**

```
## Week 4 完成
- 日期: <2026-05-XX>
- 验收: ✅ Lemon test 卡订阅 Pro → license email → 客户端激活 → PlanBadge 正确 → 5min sync 成功
- v1.0.5 待办:
  - Resend 邮件实际发送
  - Auto top-up 实际向 Lemon 发起 charge
  - pending_usage 离线队列持久化
```

---

## Week 4 完成标志（Acceptance Criteria）

对应 design Section 9 AC F1-F12：
- ✅ F1 注册即得 Free 10min
- ✅ F2 含量用完后录音停 + 弹升级
- ⏳ F3 license email 5 分钟到达（Resend Week 5 实现，MVP Week 4 仅 KV 创建）
- ✅ F4 license key 输入后 plan 立即显示
- ✅ F5 录音中 5 分钟 sync
- ⏳ F6 离线 7 天可用（缓存逻辑就绪，需要断网真测）
- ⏳ F7 多设备 lease 5 分钟（v1.0.5）
- ✅ F9 月度续订 reset quota（webhook 实现）
- ⏳ F10/F11 auto top-up（v1.0.5）

下一步：进 Week 5 — i18n + 切 Anthropic 直连 + Prompt Caching + BYO UI。
