# Confide Week 5 — i18n + Anthropic Direct + Prompt Caching + BYO UI

> **For agentic workers:** REQUIRED SUB-SKILL: Use gecc-dev:subagent-driven-development (recommended) or gecc-dev:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** UI 中英切换 + 中文 license email 正常 + 切到 Anthropic 直连 + Prompt Caching 启用让简历 RAG 成本降 80% + BYO key 模式 UI（设置里可切自带 OpenAI/Anthropic key）。Week 5 同时启动 Apple Developer Account 申请。

**Domain:** general

**Architecture:**
- 客户端 i18next + zh-CN.json + en-US.json
- LLM Provider 抽象 + 切到 Anthropic 直连（替代 alpha 期 llmgate）
- Anthropic Sonnet 4.6 + prompt caching `cache_control: ephemeral`
- BYO mode UI 在 Settings：填 OpenAI/Anthropic key、切换 provider
- Resend 双语邮件模板（在 Workers 端发送，触发于 webhook）
- 充值页 i18n（?lang= query）

**Tech Stack:** react-i18next、Anthropic SDK pattern via reqwest、tauri config

**Spec reference:** `docs/specs/2026-05-09-overseas-meeting-copilot-design.md` Section 7 + 1.7

**Prerequisite:** Week 4 完成；CF Workers 部署；license 链路工作

---

## File Structure

```
src/
├── i18n/                                   [Create dir]
│   ├── config.ts                           [Create] i18next 配置
│   └── locales/
│       ├── zh-CN.json                      [Create]
│       └── en-US.json                      [Create]
├── components/
│   ├── settings/
│   │   ├── LanguageSettings.tsx            [Create]
│   │   ├── BYOKeySettings.tsx              [Create]
│   │   └── SettingsView.tsx                [Modify] 加 Language + BYO tab
│   ├── narrow/ControlBar.tsx               [Modify] 用 t()
│   ├── narrow/AdvicePanel.tsx              [Modify] 用 t()
│   ├── meeting/NewMeetingDialog.tsx        [Modify] 用 t()
│   ├── onboarding/PermissionGate.tsx       [Modify] 用 t()
│   └── ... 其他组件渐进 i18n（MVP 不必每个组件改完）
├── lib/
│   ├── tauri.ts                            [Modify] 加 BYO commands
│   └── types.ts                            [Modify] 加 byo 字段
└── main.tsx                                [Modify] import i18n config

src-tauri/
├── Cargo.toml                              [Modify] 加 once_cell 已有
└── src/
    ├── i18n/                               [Create dir]
    │   └── mod.rs                          [Create] 后端字符串表
    ├── llm/                                [Create dir]
    │   ├── mod.rs                          [Create] LlmProvider trait
    │   ├── anthropic.rs                    [Create] AnthropicProvider with caching
    │   ├── openai.rs                       [Create] OpenAiProvider (BYO)
    │   └── confide_proxy.rs                [Create] alpha 期 llmgate 适配
    ├── advisor/engine.rs                   [Modify] 用 LlmProvider 替代直接 reqwest
    ├── storage/config.rs                   [Modify] 加 byo / language_preference 字段
    └── commands.rs                         [Modify] 加 BYO commands

workers/
└── src/
    ├── webhook.ts                          [Modify] 实际发 Resend 邮件
    └── emails.ts                           [Create] 双语邮件模板
```

---

### Task 1: 申请 Apple Developer Account（并行）

**Files:** 无（外部）

- [ ] **Step 1: 提交申请**

去 https://developer.apple.com/programs/enroll/ → Individual ($99/year) → 用真实姓名 + 地址 + 信用卡。

预计 2-7 天审核。Week 6 收口需要 dmg 签名 + 公证，所以 Day 21 必须提交。

- [ ] **Step 2: 在 decision-log.md 记录提交日期**

```
## Week 5 - Apple Developer
- 提交日期: <2026-05-XX>
- 状态: ⏳ pending review
- Team ID: <pending>
```

---

### Task 2: 加 react-i18next + 写两份 locale

**Files:**
- Modify: `package.json`
- Create: `src/i18n/config.ts`
- Create: `src/i18n/locales/zh-CN.json`
- Create: `src/i18n/locales/en-US.json`
- Modify: `src/main.tsx`

- [ ] **Step 1: 加依赖**

```bash
cd /Users/bytedance/meeting-assistant
pnpm add react-i18next i18next i18next-browser-languagedetector
```

- [ ] **Step 2: 写 src/i18n/config.ts**

```typescript
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import zhCN from "./locales/zh-CN.json";
import enUS from "./locales/en-US.json";

function detectInitialLanguage(): string {
  const stored = localStorage.getItem("confide.uiLang");
  if (stored === "zh-CN" || stored === "en-US") return stored;
  const sys = navigator.language.toLowerCase();
  if (sys.startsWith("zh")) return "zh-CN";
  return "en-US";
}

i18n.use(initReactI18next).init({
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
  localStorage.setItem("confide.uiLang", lng);
}
```

- [ ] **Step 3: 写 src/i18n/locales/en-US.json**

```json
{
  "control": {
    "start": "Start",
    "pause": "Pause",
    "resume": "Resume",
    "stop": "Stop",
    "elapsed": "Recorded {{time}}"
  },
  "onboarding": {
    "screenRecording": {
      "title": "Screen Recording permission required",
      "body": "Confide needs Screen Recording access to capture meeting audio (Zoom, Meet, Teams). We never see your screen — only system audio. macOS doesn't provide a separate audio-only permission, which is why this looks scarier than it is.",
      "afterEnable": "After enabling, you must quit and restart Confide for the permission to take effect.",
      "openSettings": "Open System Settings",
      "recheck": "Re-check"
    },
    "macosTooOld": {
      "title": "macOS 13 or later required",
      "body": "Confide uses Apple's ScreenCaptureKit framework to capture meeting audio without requiring third-party drivers like BlackHole. This framework is only available on macOS 13.0 and later."
    }
  },
  "newMeeting": {
    "title": "New Meeting",
    "template": "Template",
    "contextDoc": "Context document (optional)",
    "pickDoc": "Drop or pick PDF / MD / TXT",
    "changeDoc": "Change…",
    "contextNote": "Context note (optional, ≤500 chars)",
    "contextNotePlaceholder": "e.g. This is the Stripe onsite second round, focus on system design.",
    "cancel": "Cancel",
    "start": "Start Recording"
  },
  "billing": {
    "freePlan": "FREE",
    "proPlan": "PRO",
    "ultraPlan": "ULTRA",
    "minRemaining": "{{used}}/{{total}} min",
    "quotaLow": "Less than 1 minute remaining this month",
    "quotaExhausted": "Monthly quota reached",
    "upgrade": "Upgrade Plan",
    "buyMore": "Buy more"
  },
  "settings": {
    "title": "Settings",
    "tabs": {
      "audio": "Audio",
      "llm": "AI Models",
      "language": "Language",
      "license": "License",
      "byo": "BYO Key",
      "profile": "Profile"
    },
    "language": {
      "appLanguage": "App Language",
      "appLanguageDesc": "Controls UI display language",
      "audioLanguage": "Default Audio Language",
      "audioLanguageDesc": "Default ASR language for new meetings"
    },
    "byo": {
      "title": "Bring Your Own Key",
      "desc": "Use your own OpenAI / Anthropic API key. When BYO is active, recordings don't count against your monthly quota.",
      "openaiLabel": "OpenAI API Key",
      "anthropicLabel": "Anthropic API Key",
      "activeLabel": "BYO mode active",
      "saveBtn": "Save"
    }
  },
  "stealth": {
    "indicatorOn": "🛡️ Stealth ON",
    "indicatorOff": "Stealth OFF",
    "tooltip": "⌘⇧H toggle window · ⌘⇧K panic stop"
  }
}
```

- [ ] **Step 4: 写 src/i18n/locales/zh-CN.json**

```json
{
  "control": {
    "start": "开始",
    "pause": "暂停",
    "resume": "继续",
    "stop": "停止",
    "elapsed": "已录 {{time}}"
  },
  "onboarding": {
    "screenRecording": {
      "title": "需要屏幕录制权限",
      "body": "Confide 需要屏幕录制权限来捕获会议音频（Zoom / Meet / Teams）。我们从不查看你的屏幕——只读取系统音频。macOS 没有单独的音频权限，所以看起来吓人但其实是必要的。",
      "afterEnable": "授权后必须退出并重启 Confide 才能生效。",
      "openSettings": "打开系统设置",
      "recheck": "重新检查"
    },
    "macosTooOld": {
      "title": "需要 macOS 13 或更高版本",
      "body": "Confide 使用 Apple 的 ScreenCaptureKit 框架捕获会议音频，无需安装 BlackHole 等第三方驱动。该框架仅在 macOS 13.0 及以上版本可用。"
    }
  },
  "newMeeting": {
    "title": "新建会议",
    "template": "模板",
    "contextDoc": "上下文文档（可选）",
    "pickDoc": "拖入或选择 PDF / MD / TXT",
    "changeDoc": "更换…",
    "contextNote": "备注（可选，≤500 字）",
    "contextNotePlaceholder": "例：这是 Stripe onsite 第二轮，重点考察系统设计。",
    "cancel": "取消",
    "start": "开始录音"
  },
  "billing": {
    "freePlan": "免费",
    "proPlan": "PRO",
    "ultraPlan": "ULTRA",
    "minRemaining": "{{used}}/{{total}} 分钟",
    "quotaLow": "本月剩余不足 1 分钟",
    "quotaExhausted": "本月含量已用完",
    "upgrade": "升级套餐",
    "buyMore": "购买更多"
  },
  "settings": {
    "title": "设置",
    "tabs": {
      "audio": "音频",
      "llm": "AI 模型",
      "language": "语言",
      "license": "License",
      "byo": "自带 Key",
      "profile": "个人资料"
    },
    "language": {
      "appLanguage": "应用语言",
      "appLanguageDesc": "控制界面显示语言",
      "audioLanguage": "默认音频语言",
      "audioLanguageDesc": "新建会议的默认识别语言"
    },
    "byo": {
      "title": "自带 API Key",
      "desc": "使用你自己的 OpenAI / Anthropic API key。BYO 模式下录音不消耗本月含量。",
      "openaiLabel": "OpenAI API Key",
      "anthropicLabel": "Anthropic API Key",
      "activeLabel": "BYO 模式已启用",
      "saveBtn": "保存"
    }
  },
  "stealth": {
    "indicatorOn": "🛡️ 隐身已开",
    "indicatorOff": "隐身已关",
    "tooltip": "⌘⇧H 切换窗口 · ⌘⇧K 紧急停止"
  }
}
```

- [ ] **Step 5: 在 main.tsx import config**

```typescript
import "./i18n/config";  // 在 React import 之前
```

- [ ] **Step 6: typecheck**

```bash
pnpm typecheck 2>&1 | tail -5
```

---

### Task 3: 让组件用 t()

**Files:**
- Modify: `src/components/narrow/ControlBar.tsx`
- Modify: `src/components/onboarding/PermissionGate.tsx`
- Modify: `src/components/meeting/NewMeetingDialog.tsx`
- Modify: `src/components/license/QuotaExhausted.tsx`
- Modify: `src/components/stealth/StealthIndicator.tsx`

- [ ] **Step 1: ControlBar 替换硬编码字符串**

```tsx
import { useTranslation } from "react-i18next";

export function ControlBar(...) {
  const { t } = useTranslation();
  // 替换 "Start" → t("control.start"), 等等
}
```

- [ ] **Step 2-5: 同样模式替换其他组件**

每个组件 import + 替换硬编码字符串到 `t("namespace.key")`。

**MVP 不强求每个组件全改完**——优先 onboarding / 充值相关 / 错误提示，这些是用户首屏看到的。其他次要组件 v1.0.1 收尾。

- [ ] **Step 6: 检查 [missing key] 警告**

启动 dev：
```bash
OPENAI_API_KEY="<key>" pnpm tauri dev
```

打开 Console，搜 "missing"。修复出现的 key。

- [ ] **Step 7: typecheck**

```bash
pnpm typecheck 2>&1 | tail -5
```

---

### Task 4: 写 LanguageSettings 组件 + 切换实际工作

**Files:**
- Create: `src/components/settings/LanguageSettings.tsx`
- Modify: `src/components/settings/SettingsView.tsx`

- [ ] **Step 1: 写组件**

```tsx
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
```

- [ ] **Step 2: 在 SettingsView 加 tab**

```tsx
const tabs = ["audio", "llm", "language", "license", "byo", "profile"] as const;

// In render:
{activeTab === "language" && <LanguageSettings />}
```

---

### Task 5: 写 LLM Provider Rust 抽象层

**Files:**
- Create: `src-tauri/src/llm/mod.rs`
- Create: `src-tauri/src/llm/anthropic.rs`
- Create: `src-tauri/src/llm/openai.rs`
- Create: `src-tauri/src/llm/confide_proxy.rs`
- Modify: `src-tauri/src/advisor/engine.rs`

- [ ] **Step 1: 写 llm/mod.rs**

```rust
pub mod anthropic;
pub mod openai;
pub mod confide_proxy;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,        // "system" | "user" | "assistant"
    pub content: String,
}

pub struct ChatOptions {
    pub max_tokens: u32,
    pub temperature: f32,
    pub enable_caching: bool,  // Anthropic prompt caching
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self { max_tokens: 500, temperature: 0.7, enable_caching: true }
    }
}

#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, messages: &[LlmMessage], opts: &ChatOptions) -> Result<String>;
    fn provider_name(&self) -> &'static str;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LlmMode {
    /// Alpha period: route through llmgate (Confide proxy with internal Bytedance LLM)
    ConfideProxy { base_url: String, token: String, model: String },
    /// Production: direct to Anthropic
    Anthropic { api_key: String, model: String },
    /// BYO: user-provided OpenAI key
    UserOpenAi { api_key: String, model: String, base_url: String },
    /// BYO: user-provided Anthropic key
    UserAnthropic { api_key: String, model: String },
}

pub fn create_provider(mode: &LlmMode) -> Box<dyn LlmProvider> {
    match mode {
        LlmMode::ConfideProxy { base_url, token, model } => {
            Box::new(confide_proxy::ConfideProxyProvider::new(base_url, token, model))
        }
        LlmMode::Anthropic { api_key, model } => {
            Box::new(anthropic::AnthropicProvider::new(api_key, model))
        }
        LlmMode::UserOpenAi { api_key, model, base_url } => {
            Box::new(openai::OpenAiProvider::new(api_key, model, base_url))
        }
        LlmMode::UserAnthropic { api_key, model } => {
            Box::new(anthropic::AnthropicProvider::new(api_key, model))
        }
    }
}
```

- [ ] **Step 2: 写 anthropic.rs（含 prompt caching）**

```rust
use anyhow::{anyhow, Result};
use serde_json::json;
use super::{LlmMessage, LlmProvider, ChatOptions};

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat(&self, messages: &[LlmMessage], opts: &ChatOptions) -> Result<String> {
        // Anthropic API expects "system" as separate field, plus user/assistant turns
        let system_msg = messages.iter().find(|m| m.role == "system").map(|m| &m.content);
        let user_assistant: Vec<_> = messages.iter()
            .filter(|m| m.role != "system")
            .map(|m| json!({ "role": m.role, "content": m.content }))
            .collect();

        // Apply prompt caching to system message if enabled
        let system_field = match (system_msg, opts.enable_caching) {
            (Some(s), true) => json!([{
                "type": "text",
                "text": s,
                "cache_control": { "type": "ephemeral" }
            }]),
            (Some(s), false) => json!(s),
            (None, _) => json!(""),
        };

        let body = json!({
            "model": self.model,
            "max_tokens": opts.max_tokens,
            "temperature": opts.temperature,
            "system": system_field,
            "messages": user_assistant,
        });

        let resp = self.client.post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Anthropic API error ({}): {}", status, body));
        }

        let json: serde_json::Value = resp.json().await?;
        let text = json["content"][0]["text"].as_str().unwrap_or("").to_string();
        Ok(text)
    }

    fn provider_name(&self) -> &'static str { "anthropic" }
}
```

- [ ] **Step 3: 写 openai.rs (BYO 用)**

```rust
use anyhow::{anyhow, Result};
use serde_json::json;
use super::{LlmMessage, LlmProvider, ChatOptions};

pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiProvider {
    pub fn new(api_key: &str, model: &str, base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(&self, messages: &[LlmMessage], opts: &ChatOptions) -> Result<String> {
        let body = json!({
            "model": self.model,
            "messages": messages,
            "temperature": opts.temperature,
            "max_tokens": opts.max_tokens,
        });

        let resp = self.client.post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send().await?;

        if !resp.status().is_success() {
            let s = resp.status();
            let b = resp.text().await.unwrap_or_default();
            return Err(anyhow!("OpenAI API error ({}): {}", s, b));
        }

        let json: serde_json::Value = resp.json().await?;
        Ok(json["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string())
    }

    fn provider_name(&self) -> &'static str { "openai" }
}
```

- [ ] **Step 4: 写 confide_proxy.rs (alpha 期 llmgate 适配)**

```rust
use anyhow::Result;
use super::{LlmMessage, LlmProvider, ChatOptions};
use super::openai::OpenAiProvider;

/// Alpha-period adapter: routes to llmgate.io with the bundled token.
/// llmgate is OpenAI-protocol-compatible, so this is just an OpenAiProvider
/// pointed at the internal Bytedance gateway.
pub struct ConfideProxyProvider {
    inner: OpenAiProvider,
}

impl ConfideProxyProvider {
    pub fn new(base_url: &str, token: &str, model: &str) -> Self {
        Self {
            inner: OpenAiProvider::new(token, model, base_url),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for ConfideProxyProvider {
    async fn chat(&self, messages: &[LlmMessage], opts: &ChatOptions) -> Result<String> {
        self.inner.chat(messages, opts).await
    }
    fn provider_name(&self) -> &'static str { "confide-proxy-llmgate" }
}
```

- [ ] **Step 5: 改 advisor/engine.rs 用新抽象**

`src-tauri/src/advisor/engine.rs` 改 `AdvisorEngine`：

```rust
use crate::llm::{LlmProvider, LlmMessage as LlmMsg, ChatOptions};

pub struct AdvisorEngine {
    provider: Box<dyn LlmProvider>,
}

impl AdvisorEngine {
    pub fn new_with_provider(provider: Box<dyn LlmProvider>) -> Self {
        Self { provider }
    }

    /// Backward-compat: build provider from old AppConfig.llm
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        // Decide provider based on base_url heuristic (Week 5 transitional)
        use crate::llm::{LlmMode, create_provider};
        let mode = if base_url.contains("llmgate") {
            LlmMode::ConfideProxy { base_url: base_url.into(), token: api_key.into(), model: model.into() }
        } else if base_url.contains("anthropic") {
            LlmMode::Anthropic { api_key: api_key.into(), model: model.into() }
        } else {
            LlmMode::UserOpenAi { api_key: api_key.into(), model: model.into(), base_url: base_url.into() }
        };
        Self { provider: create_provider(&mode) }
    }

    async fn chat(&self, messages: &[LlmMsg], max_tokens: u32) -> Result<String> {
        let opts = ChatOptions { max_tokens, temperature: 0.7, enable_caching: true };
        self.provider.chat(messages, &opts).await
    }

    // ... rest unchanged: generate_summary, generate_advice, generate_minutes
}
```

- [ ] **Step 6: 在 lib.rs / main.rs 暴露**

```rust
pub mod llm;
```

- [ ] **Step 7: 编译验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

---

### Task 6: BYO 设置 UI + AppConfig 扩展

**Files:**
- Modify: `src-tauri/src/storage/config.rs`
- Modify: `src-tauri/src/commands.rs`
- Create: `src/components/settings/BYOKeySettings.tsx`
- Modify: `src/components/settings/SettingsView.tsx`

- [ ] **Step 1: 扩展 AppConfig**

```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ByoConfig {
    pub active: bool,
    pub openai_api_key: String,
    pub anthropic_api_key: String,
    pub anthropic_model: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm: LlmConfig,
    pub audio: AudioConfig,
    pub language_preference: String,
    pub analysis_mode: String,
    #[serde(default)]
    pub byo: ByoConfig,
}
```

- [ ] **Step 2: 改 Default 加 byo 默认值**

```rust
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig {
                base_url: "https://llmgate.io/v1".into(),
                api_key: String::new(),
                model: "claude-sonnet-4-6".into(),
            },
            audio: AudioConfig {
                mic_device: String::new(),
                capture_device: String::new(),
                noise_reduction: true,
            },
            language_preference: "auto".into(),
            analysis_mode: "balanced".into(),
            byo: ByoConfig {
                active: false,
                openai_api_key: String::new(),
                anthropic_api_key: String::new(),
                anthropic_model: "claude-sonnet-4-6".into(),
            },
        }
    }
}
```

- [ ] **Step 3: 写 BYOKeySettings 组件**

```tsx
import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { getConfig, saveConfig } from "../../lib/tauri";
import type { AppConfig } from "../../lib/types";

export function BYOKeySettings() {
  const { t } = useTranslation();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [openaiKey, setOpenaiKey] = useState("");
  const [anthropicKey, setAnthropicKey] = useState("");
  const [active, setActive] = useState(false);

  useEffect(() => {
    void getConfig().then((c) => {
      setConfig(c);
      setOpenaiKey(c.byo?.openai_api_key ?? "");
      setAnthropicKey(c.byo?.anthropic_api_key ?? "");
      setActive(c.byo?.active ?? false);
    });
  }, []);

  async function handleSave() {
    if (!config) return;
    await saveConfig({
      ...config,
      byo: {
        active,
        openai_api_key: openaiKey,
        anthropic_api_key: anthropicKey,
        anthropic_model: config.byo?.anthropic_model ?? "claude-sonnet-4-6",
      },
    });
    alert("Saved");
  }

  return (
    <div className="space-y-4">
      <h3 className="font-medium">{t("settings.byo.title")}</h3>
      <p className="text-xs text-gray-400">{t("settings.byo.desc")}</p>

      <div>
        <label className="text-sm block mb-1">{t("settings.byo.openaiLabel")}</label>
        <input
          type="password"
          value={openaiKey}
          onChange={(e) => setOpenaiKey(e.target.value)}
          className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-2 py-1 text-sm font-mono"
          placeholder="sk-..."
        />
      </div>

      <div>
        <label className="text-sm block mb-1">{t("settings.byo.anthropicLabel")}</label>
        <input
          type="password"
          value={anthropicKey}
          onChange={(e) => setAnthropicKey(e.target.value)}
          className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-2 py-1 text-sm font-mono"
          placeholder="sk-ant-..."
        />
      </div>

      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={active}
          onChange={(e) => setActive(e.target.checked)}
        />
        {t("settings.byo.activeLabel")}
      </label>

      <button onClick={() => void handleSave()} className="px-4 py-2 bg-[var(--accent-purple)] text-white rounded text-sm">
        {t("settings.byo.saveBtn")}
      </button>
    </div>
  );
}
```

- [ ] **Step 4: 在 SettingsView 加 BYO tab**

引用 `<BYOKeySettings />` 在 `byo` tab 下渲染。

- [ ] **Step 5: 编译验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
pnpm typecheck 2>&1 | tail -5
```

---

### Task 7: Resend 邮件双语模板（在 Workers 实现）

**Files:**
- Create: `workers/src/emails.ts`
- Modify: `workers/src/webhook.ts`

- [ ] **Step 1: 写 emails.ts**

```typescript
import { Tier } from "./plans";

type Locale = "zh-CN" | "en-US";

export function licenseEmail(locale: Locale, key: string, tier: Tier) {
  if (locale === "zh-CN") {
    return {
      subject: `你的 Confide license: ${tier.toUpperCase()}`,
      html: `
        <div style="font-family: -apple-system, sans-serif; max-width: 480px;">
          <h1 style="font-size: 22px;">感谢订阅 Confide ${tier.toUpperCase()}</h1>
          <p>你的 license key:</p>
          <code style="display: block; font-size: 16px; padding: 14px; background: #f5f5f5; border-radius: 6px; word-break: break-all;">${key}</code>
          <p style="margin-top: 18px; font-size: 14px; color: #666;">
            打开 Confide → 设置 → License → 输入此 key 激活。
          </p>
          <p style="font-size: 12px; color: #999; margin-top: 24px;">
            遇到问题：hello@confide.knosi.xyz
          </p>
        </div>
      `,
    };
  }
  return {
    subject: `Your Confide ${tier.toUpperCase()} license`,
    html: `
      <div style="font-family: -apple-system, sans-serif; max-width: 480px;">
        <h1 style="font-size: 22px;">Thanks for subscribing to Confide ${tier.toUpperCase()}</h1>
        <p>Your license key:</p>
        <code style="display: block; font-size: 16px; padding: 14px; background: #f5f5f5; border-radius: 6px; word-break: break-all;">${key}</code>
        <p style="margin-top: 18px; font-size: 14px; color: #666;">
          Open Confide → Settings → License → enter this key to activate.
        </p>
        <p style="font-size: 12px; color: #999; margin-top: 24px;">
          Need help? hello@confide.knosi.xyz
        </p>
      </div>
    `,
  };
}

export async function sendEmail(
  resendApiKey: string,
  to: string,
  email: { subject: string; html: string },
): Promise<void> {
  const r = await fetch("https://api.resend.com/emails", {
    method: "POST",
    headers: {
      Authorization: `Bearer ${resendApiKey}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      from: "Confide <hello@confide.knosi.xyz>",
      to: [to],
      subject: email.subject,
      html: email.html,
    }),
  });
  if (!r.ok) {
    console.error("Resend send failed:", await r.text());
  }
}
```

- [ ] **Step 2: 加 RESEND_API_KEY 到 Env**

`workers/src/env.d.ts` 加：
```typescript
RESEND_API_KEY: string;
```

部署 secret:
```bash
cd workers
wrangler secret put RESEND_API_KEY --env production
# 粘贴 Week 0 Task 9 的 Resend key
```

- [ ] **Step 3: 在 webhook.ts 调 sendEmail**

替换 `sendLicenseEmail` 函数：

```typescript
import { licenseEmail, sendEmail } from "./emails";

async function sendLicenseEmail(env: Env, email: string, key: string, tier: Tier): Promise<void> {
  // MVP: locale 暂用 en-US 默认；v1.0.5 从 license.locale 字段取
  const tmpl = licenseEmail("en-US", key, tier);
  await sendEmail(env.RESEND_API_KEY, email, tmpl);
}
```

- [ ] **Step 4: 重 deploy**

```bash
cd workers && wrangler deploy --env production
```

- [ ] **Step 5: typecheck**

```bash
pnpm typecheck 2>&1 | tail -5
```

---

### Task 8: 端到端验证

- [ ] **Step 1: 切 UI 中文，确认所有改过的组件中文化**

启动 → Settings > Language > 中文 → 关闭 settings 看主界面。

Expected: 录音按钮 "开始" / control bar / new meeting dialog 都是中文。

- [ ] **Step 2: 验证 BYO key 模式**

Settings > BYO Key → 填 OpenAI key → 勾 active → save。

录音 → Tauri terminal 应显示 advisor 用 OpenAI provider（看日志 `provider_name = "openai"`）。

服务端余额（PlanBadge）**不变化**（BYO 模式）。

- [ ] **Step 3: 验证 Anthropic 直连 + caching**

把 Settings > AI Models 的 base_url 改 `https://api.anthropic.com/v1`、api_key 填 Anthropic key、model 填 `claude-sonnet-4-6`。

录音 → 第一次 advice 调用应正常。

打开 Anthropic Console > Usage → 看 cache_creation_input_tokens / cache_read_input_tokens。第一次会有 creation，后续 advice 应该出现 cache_read（说明 caching 工作）。

- [ ] **Step 4: 验证 license email**

Lemon test mode 订阅 → 等 1-2 分钟看邮件。Expected: 收到 "Your Confide PRO license" 邮件 + license key。

- [ ] **Step 5: 标 Week 5 完成**

```
## Week 5 完成
- 日期: <2026-05-XX>
- 验收: ✅ UI 中英切换 / Anthropic 直连 + caching / BYO 模式 / 双语 email
- Apple Dev: <pending / approved / Team ID>
```

---

## Week 5 完成标志（Acceptance Criteria）

对应 design Section 9 AC：
- ✅ G1 系统 locale = zh-* 时默认中文 UI
- ✅ G2 切语言后无 [missing] 占位符
- ✅ G3 邮件按 license 创建时的 locale 发送（Week 5 实际 en-US fallback；v1.0.5 完整）
- ✅ F3 license email ≤5 分钟到达
- ✅ F8 BYO 不消耗 Confide 余额
- ✅ Section 1.7 Prompt Caching 启用

下一步：进 Week 6 — 收口 + 充值页 + 部署 + 自验证。
