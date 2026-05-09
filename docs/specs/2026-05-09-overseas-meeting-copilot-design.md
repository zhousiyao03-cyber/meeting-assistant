# Confide — Overseas Meeting Copilot Design

**Date**: 2026-05-09
**Codename**: Confide（最终产品名 PH 上线前定）
**Author**: Zhou Siyao
**Status**: Approved design, ready for plan generation

---

## 0. 文档定位

把现有 macOS 桌面 demo（`meeting-assistant`，Tauri + Rust + React）改造为面向海外市场的商业产品。这份 design 不是"全新建项目"，而是把现有 ~4500 行代码的 demo 升级到能上 Producthunt 的 MVP。

**target market 优先级**：
1. 海外华人 IC / 经理（英文会议高压场景）
2. 中文母语者的中文面试场景
3. 海外多语种知识工作者（v1.1 扩展）
4. ~~国内用户~~（v1.1 再考虑微信支付/支付宝）

**MVP 完成定义（一句话）**：你自己能在 6-8 周内用 Confide 完成一次真实英文面试 + 一次真实中文 1:1，stealth 不漏陷，付费链路不丢钱，5 个朋友试用反馈正面。

---

## 1. 商业模式与定价

### 1.1 定位

> "Confide — Real-time meeting copilot powered by industry-best AI stack. Speak with confidence in any language."

主品牌叙事：**会议 Copilot（含面试场景）**。不走 Cluely 式激进文案，走 Final Round 式"隐晦"姿态。Landing page 主页讲会议，`/interview` 子页讲面试 + stealth。

### 1.2 技术 stack 即卖点

| 组件 | Confide 用什么 |
|---|---|
| ASR | **GPT-Realtime-Whisper**（OpenAI 2026-05-07 发布的 SOTA 流式 ASR） |
| LLM | **Claude Sonnet 4.6**（Anthropic 主力模型） |
| 音频管线 | ScreenCaptureKit 原生（macOS 13+，无需 BlackHole） |
| Stealth | NSWindowSharingType + 进程伪装 + 全局快捷键 |
| 隐私 | 音频不存档，转录文本 24h 后删除 |

Landing page 文案：**"Powered by Claude 4.6 — Anthropic's most capable model + GPT-Realtime-Whisper"**。

### 1.3 SKU 与定价（3 档订阅）

```
┌─ Free ──────────────────────────────────────────┐
│ $0/月 — 永久免费                                 │
│   • 10 minutes / month                          │
│   • All languages + 全部模板                     │
│   • Cloud ASR + Sonnet 4.6                       │
│   • Stealth 模式                                 │
│   • BYO key 模式（用自己的 key、不消耗含量）       │
│   • 简历 RAG  ❌ 禁用（Pro 解锁）                  │
│   • 通话历史保存 7 天                              │
│                                                  │
│ → 永久免费、0 信用卡门槛、PLG 流量入口             │
└──────────────────────────────────────────────────┘

┌─ Pro ────────────────────────── 最受欢迎 ────────┐
│ $19/月                                           │
│   • 60 minutes / month                          │
│   • Free 全部 +                                  │
│   • 简历 RAG（无限次）                            │
│   • 通话历史 永久保存                              │
│   • 简历优化（5 份/月，v1.0.5）                   │
│   • 超额 $0.35/min                               │
│                                                  │
│ → 主路径，目标 80% 付费用户                        │
└──────────────────────────────────────────────────┘

┌─ Ultra ──────────────────────────────────────────┐
│ $49/月                                           │
│   • 200 minutes / month                          │
│   • Pro 全部 +                                   │
│   • 简历优化（15 份/月，v1.0.5）                  │
│   • 行业题库（v1.1）                              │
│   • 优先支持（24h 响应）                          │
│   • 超额 $0.25/min                               │
│                                                  │
│ → 重度用户 + 高 LTV                               │
└──────────────────────────────────────────────────┘

订阅按月续订。取消订阅后剩余含量本月仍可用、月底失效。
```

### 1.4 功能矩阵

```
                        Free    Pro     Ultra
转录 + AI advice         ✅      ✅      ✅
Stealth 模式             ✅      ✅      ✅
全部模板（面试 + 会议）   ✅      ✅      ✅
中英双语                 ✅      ✅      ✅
BYO key 模式             ✅      ✅      ✅
通话历史保存             7 天     永久     永久
简历 RAG                ❌      ✅      ✅
简历优化（v1.0.5）       —      5/月     15/月
行业题库（v1.1）         —      —       ✅
优先支持                 —      —       ✅
含量                    10 min   60 min   200 min
超额单价                $0.50   $0.35    $0.25
```

### 1.5 单 SKU 利润模型

按 Lemon Squeezy 6.5% + $0.50 手续费、Whisper $1.02/h、Sonnet $0.54/h（**含 prompt caching 优化后实际更低**）估算。Free 用户禁用 RAG 后单成本从 $0.21 降到 ~$0.05/月。

| Plan | 用户付 | 满用净利 | 毛利率 | 半用净利 | 毛利率 |
|---|---|---|---|---|---|
| Free | $0 | -$0.05 | — | -$0.02 | — |
| Pro $19 | $19 | $14.90 | **78%** | $16.95 | **89%** |
| Ultra $49 | $49 | $35.65 | **73%** | $42.35 | **86%** |

**平均毛利率约 80%**——SaaS 健康线优秀。订阅模式 + prompt caching 让简历 RAG 几乎免费。

**Free 用户经济**：1000 个 Free 用户每月成本约 $50（ASR）。视为"营销获客投入"——1% Free→Pro 转化即净正。

**超额用户最赚钱**：Pro 用户超额时 $0.35/min 收入 - $0.026/min 成本 = $0.32/min 净利（91% 毛利）。订阅模式下不必怕用户用得多。

### 1.6 模型升级路径（保留）

MVP 只用 Sonnet 4.6。**Opus 4.7 路径保留在 v1.1**——LLM Provider 抽象层让加 Opus 不用改架构。Landing page MVP 阶段不提 Opus，避免"既然有 Opus 为什么默认是 Sonnet"的用户疑问。

### 1.7 Prompt Caching（重要成本优化）

启用 Anthropic Sonnet 4.6 prompt caching：用户简历 chunks 在一场会议内被 30+ 次 advice 调用复用，cache hit 后 input 价格降到 1/10。60min 面试场景 LLM 成本从 ~$0.25 降到 ~$0.04（约 80% 节省）。

实现：每次 LLM 调用 system prompt 加 `cache_control: { type: "ephemeral" }`。Week 5 切 Anthropic 直连时同步启用。

---

## 2. 架构变更

### 2.1 现状（demo）

```
[BlackHole 2ch] ──┐
                  │
[mic via cpal] ───┼─→ [SharedBuffer 双通道] ─→ [SenseVoice + Silero VAD]
                  │                              │
                  │                              ▼
                  │                         [TranscriptStore]
                  │                              │
                  │                              ▼
                  │                         [Advisor (rules + llmgate.io)]
                  │                              │
                  │                              ▼
                  │                         [本地 SQLite + 本地 config]
```

**问题**：BlackHole 装机失败率高、llmgate 海外用不了、无账号无付费、模板字节内部化、UI 单中文。

### 2.2 目标架构（MVP after）

```
┌─────────────────── Confide Desktop App (macOS 13+) ─────────────────┐
│                                                                       │
│  ScreenCaptureKit ──┐                                                 │
│  (system audio)     │                                                 │
│                     ├─→ AudioCapture trait ─→ SharedBuffer            │
│  cpal mic ──────────┘                              │                  │
│                                                    ▼                  │
│                                          [VAD / segment]              │
│                                                    │                  │
│                                                    ▼                  │
│                       ┌──────────── AsrProvider trait ────────────┐  │
│                       │  GPT-Realtime-Whisper (via Confide proxy) │  │
│                       │  (v1.1) LocalSenseVoiceProvider           │  │
│                       └────────────────────┬───────────────────────┘  │
│                                            │                          │
│                                            ▼                          │
│                                    [TranscriptStore]                  │
│                                            │                          │
│                                            ▼                          │
│                                    [Advisor + rules]                  │
│                                            │                          │
│                       ┌──────────── LlmProvider trait ─────────────┐  │
│                       │  AnthropicProvider (Sonnet 4.6)            │  │
│                       │  ConfideProxyProvider (alpha→llmgate)      │  │
│                       │  ByoOpenAiProvider / ByoAnthropicProvider  │  │
│                       └────────────────────┬───────────────────────┘  │
│                                            │                          │
│  ┌─────────────────┐                       │                          │
│  │ LicenseManager  │ ←────── 5 min sync ───┤                          │
│  │ UserPlan        │                       │                          │
│  └────────┬────────┘                       │                          │
└───────────┼────────────────────────────────┼──────────────────────────┘
            │                                │
            ▼                                ▼
   ┌────────────────────────────────────────────────────────┐
   │ Confide Cloud (CF Workers + Hono + KV + D1)            │
   │   /balance, /usage, /audio-proxy/whisper,              │
   │   /llm-proxy/chat, /lemonsqueezy-webhook               │
   └────────────────────────────────────────────────────────┘
            │
            ▼
   ┌────────────┬─────────────┬────────────┐
   │ OpenAI     │ Anthropic   │ Lemon      │
   │ Realtime   │ Sonnet 4.6  │ Squeezy    │
   │ (Whisper)  │             │ (MoR)      │
   └────────────┴─────────────┴────────────┘
```

### 2.3 5 项关键变更

1. **音频管线**：BlackHole 砍掉，ScreenCaptureKit 替代。详见 §4。
2. **ASR Provider 抽象层**：MVP 默认 GPT-Realtime-Whisper（通过 Confide proxy）。
3. **LLM Provider 抽象层**：alpha 走 llmgate（Sonnet 4.6），Week 5 切 Anthropic 直连。
4. **License Manager + Hour Counter**：本地缓存 7 天 + 5 分钟 sync 到云。
5. **i18n 框架**：UI / Audio / Advice 三层语言独立。

### 2.4 保留不动的 demo 代码（约 60-70%）

| 模块 | 状态 |
|---|---|
| `audio/buffer.rs` 环形缓冲 | 保留，接 ScreenCaptureKit 输出 |
| `transcript/store.rs` | 保留 |
| `advisor/rules.rs` 触发器 | 保留逻辑，关键词需 i18n |
| `advisor/engine.rs` parse 函数 | 保留 |
| `storage/history.rs` SQLite | 保留 |
| `documents/loader.rs` PDF/MD/TXT loader | 保留 + 加 OCR fallback |
| 前端 React 组件骨架 | 保留 + i18n 注入 |
| `useRecording`, `useTauriEvents` hooks | 保留 |
| `whisper/` 目录（SenseVoice + Silero VAD 集成） | **保留代码但 MVP 不默认启用**，v1.1 作为本地 ASR 选项重新激活 |

### 2.5 新增模块

```
src-tauri/src/
├── asr/                    ← 新增
│   ├── mod.rs              (AsrProvider trait)
│   └── whisper_realtime.rs (GPT-Realtime-Whisper 客户端)
├── llm/                    ← 新增
│   ├── mod.rs              (LlmProvider trait)
│   ├── anthropic.rs
│   ├── confide_proxy.rs    (alpha 阶段走 llmgate)
│   └── byo.rs
├── license/                ← 新增
│   ├── mod.rs              (LicenseManager)
│   ├── verify.rs
│   └── metering.rs         (Meter, 5 分钟 sync)
├── stealth/                ← 新增
│   ├── mod.rs
│   └── window.rs           (NSWindowSharingType)
├── shortcuts.rs            ← 新增（全局快捷键）
└── i18n/                   ← 新增（Rust 后端字符串表）
    └── mod.rs
```

---

## 3. 模板系统重设计

### 3.1 模板范围

MVP 仅 2 个模板，每个双语：

| Template | 场景 | 触发器侧重 | "穿"什么 |
|---|---|---|---|
| `job-interview` | 你被面试 | `on_question_to_user` 为主 | 简历 PDF + 目标岗位 JD（可选） |
| `general-meeting` | 1:1 / standup / 客户会 / tech review 全归这里 | 5 条规则全开 | 会议 agenda / 产品文档（可选） |

砍掉 `one-on-one` / `customer-call` / `team-standup`——其逻辑通过 general-meeting 的 context_note + 文档喂入实现，更灵活。

### 3.2 文件结构

```
src-tauri/templates/
├── zh-CN/
│   ├── job-interview.json
│   └── general-meeting.json
└── en-US/
    ├── job-interview.json
    └── general-meeting.json
```

模板按 **audio language** 加载（不是 UI language）。原因：英文面试要用英文模板。

### 3.3 prompt 草稿（存档，详见附录 A）

每个模板包含：
- `id`, `name`, `description`, `language`
- `role_persona_template`（带 `{{job_title}}` 等占位）
- `system_prompt`（≤30 字 advice 输出格式约束）
- `trigger_config`
- `advice_cooldown_seconds`

### 3.4 触发器系统

新增 1 条 + 改 1 条：

| 触发器 | 状态 |
|---|---|
| `on_ask_opinion` | 保留，关键词英文化 |
| `on_domain_topic` | 保留，关键词英文化 |
| `on_decision_point` | 保留，关键词英文化 |
| `on_discussion_stuck` | 保留 |
| `on_question_to_user` | **新增** |
| `custom_keywords` | 保留 |

`on_question_to_user` 检测：句末 `?`/`？` + 用户名 / 职称 / "you" / "你" 等指向用户的词。

### 3.5 "穿"的产品流程

```
新建会议
  ↓
[选择模板] Job Interview / General Meeting × zh/en
  ↓
[本场会议上下文]
  📎 拖入 PDF / MD / TXT（绑定到本场会议，不全局）
  ✏️ 备注（context_note，≤500 字）
  ↓
[语言] 中文 / English / 自动（v1.1）
  ↓
[开始录音]
```

PDF 解析：text-based PDF 用 `pdf-extract`；扫描件 fallback 到 `tesseract-rs` OCR（Week 3 测性能，不行先关 OCR、要求 text-based）。

### 3.6 Context 注入到 LLM

`generate_advice()` 拼 system prompt 时：

```
1. 模板 system_prompt
2. + context_note（如非空）
3. + 从 documents 按当前 transcript 关键词挑 top-3 chunks（保留 demo 现有算法）
4. + 用户本场 transcript 最近窗口
```

简历内容从 PDF 读出后塞进 expertise_context，LLM 拿到拼好的句子。**不做 `{{job_title}}` 字段化表单**——靠 PDF + context_note 解决。

### 3.7 用户填什么 / 不填什么

| 信息 | 来源 |
|---|---|
| 简历内容（项目、技能） | PDF 拖入 |
| 本场会议特殊上下文 | context_note 文本框 |
| 角色身份（"我是面试候选人"） | 模板内置 |

---

## 4. 音频管线（ScreenCaptureKit）

### 4.1 系统要求

**macOS 13.0+**。砍掉 macOS 12 BlackHole fallback（推 v1.1）。

### 4.2 实现

`screencapturekit-rs` crate 集成。Week 0 写 100 行 demo 验证可用性 + crate 维护活跃度。如不可用，fallback 到 `objc2` 直接 binding。

```rust
pub trait SystemAudioCapture: Send + Sync {
    fn start(&mut self, on_pcm: Box<dyn Fn(&[f32]) + Send + Sync>) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
}

pub struct ScreenCaptureKitBackend { /* macOS 13+ */ }

pub fn create_system_audio_capture() -> Result<Box<dyn SystemAudioCapture>> {
    if macos_version_at_least(13, 0) {
        Ok(Box::new(ScreenCaptureKitBackend::new()))
    } else {
        Err(anyhow!("Confide requires macOS 13+. Older versions in v1.1."))
    }
}
```

### 4.3 权限处理

ScreenCaptureKit 需 Screen Recording 权限（即使只录音频，Apple 限制）。Onboarding 弹窗主动解释：

> "Confide needs Screen Recording access to capture meeting audio. We never see your screen — only system audio. macOS doesn't have a separate audio-only permission."

授权后**强制重启 app**（macOS 限制）。

麦克风权限走 cpal 默认路径，首次自动弹。

### 4.4 双通道独立 VAD + ASR

保留 demo 现有架构：mic 一路 / system 一路独立 VAD + Whisper transcribe。Transcript segment 区分 `me` / `other`——这对面试模板至关重要（必须知道是面试官在问还是用户在说）。

### 4.5 采样率

ScreenCaptureKit 默认 48kHz、cpal 设备原生率不定 → 全部 `rubato` 重采样到 16kHz（GPT-Realtime-Whisper 接受 16kHz mono）。

### 4.6 应用白名单（哪些 app 抓音频）

ScreenCaptureKit `SCContentFilter` 默认包含：

```rust
const DEFAULT_INCLUDED_APPS: &[&str] = &[
    "us.zoom.xos",
    "com.microsoft.teams2",
    "com.google.Chrome",
    "com.apple.Safari",
    "com.hnc.Discord",
    "com.apple.FaceTime",
];

const DEFAULT_EXCLUDED_APPS: &[&str] = &[
    "com.spotify.client",
    "com.apple.Music",
    "com.apple.Notes",
];
```

MVP 用户**不能改**白名单（UI 简化）。v1.1 加用户自定义。

### 4.7 蓝牙耳机引导

蓝牙耳机部分 codec 不输出到 ScreenCaptureKit（macOS 已知 bug）。Onboarding 加显式警告 + "切到内置扬声器"建议。

---

## 5. Stealth 模式

### 5.1 设计目标

让 Confide 在以下场景对面试官完全不可见：
- Zoom / Meet / Teams 共享屏幕
- macOS 屏幕录制
- Dock / Cmd+Tab / Activity Monitor 进程列表

### 5.2 4 层实现

**Layer 1：Dock + Cmd+Tab 隐藏**

```xml
<!-- Info.plist -->
<key>LSUIElement</key>
<true/>
```

App 不出现在 Dock / Cmd+Tab / 程序坞右键菜单。用户从**菜单栏图标**打开（类似 Bartender 风格）。

**Layer 2：屏幕共享时窗口不可见（核心）**

```rust
use objc2_app_kit::{NSWindow, NSWindowSharingType};

pub fn hide_from_screen_capture(window: &NSWindow) {
    unsafe { window.setSharingType(NSWindowSharingType::None); }
}
```

效果：Zoom / Meet / Teams 共享屏幕时面试官看不到 Confide。`screencapture` 命令也截不到。

**Layer 3：进程伪装**

打包时 `Info.plist` 设：
```xml
<key>CFBundleName</key>
<string>VoiceNote</string>
<key>CFBundleDisplayName</key>
<string>VoiceNote</string>
```

Activity Monitor / `ps aux` 看到的进程名是 "VoiceNote"。MVP 走策略 A（固定名），不做策略 B（用户选伪装名）。

**Layer 4：全局快捷键**

| 快捷键 | 行为 |
|---|---|
| `⌘⇧H` | toggle 显示 / 隐藏窗口 |
| `⌘⇧K` | **panic key**：立即隐藏窗口 + 停录 + 后台保留（不退出） |
| `⌘⇧↑` / `⌘⇧↓` | 调整窗口透明度（10%-100%） |
| `⌘⇧1` / `⌘⇧2` / `⌘⇧3` | 切换提词位置（左上 / 右上 / 中央） |

### 5.3 Stealth UI

```
菜单栏图标（默认）
    ↓
点击 → 弹出菜单：
   • New Interview Meeting
   • New General Meeting
   • Stealth Mode: [ ON ]
   • Show Window  (⌘⇧H)
   • Quit

窗口本身：
   • 默认透明度 90%
   • 默认尺寸 320×500
   • 默认位置：屏幕右上角
   • Always on top + frameless
   • Sharing Type = .none
```

颜色方案：深灰半透明 `rgba(20,20,22,0.85)`。低饱和度让意外暴露时最不引人注意。

### 5.4 默认值

- **面试模板**：stealth 默认 ON
- **日常会议模板**：stealth 默认 OFF（开了反而让用户找不到窗口）

### 5.5 风险

| 风险 | 缓解 |
|---|---|
| 用户在公司管理设备上跑 Confide 被 IT 检测 | Onboarding 警告 |
| 摄像头反射显示器 → 面试官从瞳孔反光看到提词 | landing page 显式警告 + 建议调暗屏幕 |
| Karat / HireVue 反作弊面试平台检测 | 明确列为"不支持"平台，不假装绕过 |
| 用户被发现起诉 Confide | EULA "不为面试结果背书 / 用户自担合规风险" |
| App Store 审核拒 | 不上 App Store，从官网 .dmg 分发 |
| AI 提词造成虚假陈述 | system_prompt 强约束"必须引用简历真实项目" |

---

## 6. License + 计费（Lemon Squeezy + CF Workers）

### 6.1 分工

**Lemon Squeezy 负责**（用他们 SDK）：
- Hosted checkout + 收钱
- License key 自动生成 + 邮件发送（含双语模板）
- Tax / VAT 报税（Merchant of Record 模式）
- 退款 / chargeback 处理
- License activate / deactivate API

**Confide CF Workers 负责**（自写）：
- 时长余额 / Hour Balance 状态
- 5 分钟 sync 计量
- LLM Proxy（转发 Anthropic）
- Audio Proxy / token 签发（转发 OpenAI Realtime）
- 余额扣减 + 滥用监测降速

### 6.2 客户端

```rust
pub struct LicenseManager {
    key: Option<String>,
    cached_plan: UserPlan,
    last_sync: SystemTime,
    pending_usage: Vec<UsageEvent>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Tier { Free, Pro, Ultra }

#[derive(Serialize, Deserialize, Clone)]
pub struct UserPlan {
    pub tier: Tier,
    pub monthly_quota_seconds: i64,            // Free=600, Pro=3600, Ultra=12000
    pub used_this_month_seconds: i64,
    pub overage_rate_per_min_cents: i32,       // Free=50, Pro=35, Ultra=25
    pub auto_topup_enabled: bool,              // 默认 true，用户可关
    pub renews_at: i64,                        // Lemon Squeezy 订阅续订时间
    pub byo_active: bool,                      // 用户是否切到 BYO key 模式
    pub resume_rag_enabled: bool,              // Free=false, Pro/Ultra=true
    pub resume_optimization_credits: i32,      // 月度 reset, Pro=5, Ultra=15 (v1.0.5)
    pub history_persistence_days: i32,         // Free=7, Pro/Ultra=∞ (-1 表示永久)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UsageEvent {
    pub event_id: String,                      // UUIDv7，幂等键
    pub meeting_id: String,
    pub provider: String,                      // "confide" | "byo-openai" | "byo-anthropic"
    pub seconds_used: f64,
    pub started_at: i64,
    pub ended_at: i64,
}
```

### 6.3 计量逻辑（订阅含量 + 超额）

```rust
// 每 5 分钟调一次
pub async fn maybe_sync(&mut self, license: &mut LicenseManager) -> Result<()> {
    let elapsed = self.started_at.elapsed()?.as_secs_f64() - self.accumulated_seconds;
    if elapsed < 300.0 { return Ok(()); }

    // 1. BYO 模式不计入含量
    if !license.cached_plan.byo_active {
        license.cached_plan.used_this_month_seconds += elapsed as i64;
    }
    self.accumulated_seconds += elapsed;

    // 2. 加待发送队列
    license.pending_usage.push(UsageEvent {
        event_id: uuid::v7().to_string(),
        meeting_id: self.meeting_id.clone(),
        provider: self.provider.clone(),
        seconds_used: elapsed,
        started_at: self.last_sync_at.timestamp(),
        ended_at: now_unix(),
    });
    self.last_sync_at = SystemTime::now();

    // 3. 推到云（失败不阻塞录音）
    if let Err(e) = license.flush_pending().await { tracing::warn!("Sync failed: {}", e); }

    // 4. 含量 / 超额检查
    let remaining = license.cached_plan.monthly_quota_seconds - license.cached_plan.used_this_month_seconds;
    if remaining < 60 && remaining > 0 {
        emit_event("quota-low");
    }
    if remaining <= 0 {
        // 含量用完
        if matches!(license.cached_plan.tier, Tier::Free) {
            return Err(anyhow!("Free quota exhausted, upgrade to continue"));
        }
        if !license.cached_plan.auto_topup_enabled {
            return Err(anyhow!("Quota exhausted, enable auto top-up or wait for renewal"));
        }
        // Pro/Ultra + auto_topup: 转入超额计费模式（继续录音）
        emit_event("overage-mode-on");
    }
    Ok(())
}
```

### 6.3.1 月度 Quota Reset

Lemon Squeezy 订阅续订时（webhook `subscription_payment_success`）：服务端 reset `used_this_month_seconds = 0` + reset `resume_optimization_credits` 到 plan 默认值。

取消订阅（webhook `subscription_cancelled`）：保留 `cancelled_at` 字段，剩余含量本月仍可用，月底失效转 Free。

### 6.4 离线策略（7 天）

正常每 5 分钟 sync。离线第 1-7 天本地继续累计 used_this_month_seconds + pending_usage 队列。第 8 天强制要求 sync 一次才能开新会议（"You've been offline for 7 days. Please connect to verify your subscription."）。

sync 恢复时服务端按 event_id 去重；如果服务端 used_this_month_seconds < 客户端，取较大值（用户友好，差额计入 abuse log 但不阻断）。

### 6.5 Confide Proxy（CF Workers）

Hono 框架，5 个核心 endpoint：

```typescript
// workers/src/index.ts
import { Hono } from 'hono';

const app = new Hono<{ Bindings: Env }>();

app.get('/plan/:key', async (c) => {
  const license = await c.env.KV.get(`license:${c.req.param('key')}`, 'json');
  return c.json(license?.plan ?? null);
});

app.post('/usage', async (c) => {
  const { key, events } = await c.req.json();
  // 幂等 + used_this_month_seconds 累加 + D1 写事件流
  // 如果触发超额：计算超额费用，调 Lemon Squeezy create_charge API（auto top-up）
});

app.post('/llm-proxy/chat', async (c) => {
  const { key, messages } = await c.req.json();
  // 验 key + 滥用降速 + 转发 Anthropic Sonnet 4.6 + prompt caching enabled
  // 不存对话内容
});

app.post('/audio-proxy/whisper-token', async (c) => {
  const { key } = await c.req.json();
  // 签发短期 OpenAI ephemeral token（1 小时过期）
  // 客户端拿 token 直连 OpenAI WebSocket
  // 客户端如实上报 ASR 时长（依赖客户端可信度，可接受）
});

app.post('/lemonsqueezy-webhook', async (c) => {
  // 验签 + 处理订阅事件:
  // - subscription_created: 创建 license + 初始化 plan
  // - subscription_payment_success: 月度续订 reset quota + resume credits
  // - subscription_cancelled: 标记 cancelled_at, 月底转 Free
  // - subscription_expired: 切到 Free tier
});
```

### 6.6 D1 schema

```sql
CREATE TABLE usage_events (
  event_id      TEXT PRIMARY KEY,
  license_key   TEXT NOT NULL,
  provider      TEXT NOT NULL,
  seconds       REAL NOT NULL,
  ts            INTEGER NOT NULL
);
CREATE INDEX idx_usage_license_ts ON usage_events(license_key, ts);

CREATE TABLE verify_log (
  license_key   TEXT NOT NULL,
  device_id     TEXT NOT NULL,
  ts            INTEGER NOT NULL,
  PRIMARY KEY (license_key, device_id, ts)
);

CREATE TABLE lemonsqueezy_events (
  id            TEXT PRIMARY KEY,
  license_key   TEXT NOT NULL,
  event_type    TEXT NOT NULL,    -- subscription_created, payment_success, cancelled, expired, charge (overage)
  amount_cents  INTEGER,
  ts            INTEGER NOT NULL
);
CREATE INDEX idx_lemon_license_ts ON lemonsqueezy_events(license_key, ts);
```

KV 存活跃数据（license + plan），D1 存事件流（不可变记录、对账用）。

### 6.7 Plan 配置

```typescript
export const PLAN_CATALOG = {
  free: {
    lemonVariantId: null,                          // 不需要 Lemon Squeezy
    monthlyQuotaSeconds: 600,                      // 10 min
    overageRatePerMinCents: 50,                    // $0.50/min（实际无超额，到限即停）
    autoTopupEnabled: false,
    resumeRagEnabled: false,
    resumeOptimizationCredits: 0,
    historyPersistenceDays: 7,
  },
  pro: {
    lemonVariantId: 'variant_pro_monthly',
    priceUsd: 19,
    monthlyQuotaSeconds: 3600,                     // 60 min
    overageRatePerMinCents: 35,
    autoTopupEnabled: true,
    resumeRagEnabled: true,
    resumeOptimizationCredits: 5,                  // v1.0.5
    historyPersistenceDays: -1,                    // -1 = 永久
  },
  ultra: {
    lemonVariantId: 'variant_ultra_monthly',
    priceUsd: 49,
    monthlyQuotaSeconds: 12000,                    // 200 min
    overageRatePerMinCents: 25,
    autoTopupEnabled: true,
    resumeRagEnabled: true,
    resumeOptimizationCredits: 15,
    historyPersistenceDays: -1,
  },
};
```

订阅按月续订。Lemon Squeezy 处理订阅生命周期 + 续订 + 退款 + 税务。

### 6.8 防滥用（3 层软防护，不硬封）

| 层 | 触发 | 行为 |
|---|---|---|
| Layer 1 | 每次 verify 上报 device_id | 仅记 D1 |
| Layer 2 | 同 license 同时录音 | 后启动会议失败（5 分钟 lease） |
| Layer 3 | 30 天内 >5 个 unique device | LLM Proxy 加 30s 延迟，让滥用者自然流失 |

**不主动封 license**——封了被 Reddit 骂、产生退款纠纷。

### 6.9 Stripe statement descriptor

`statement_descriptor: "VOICENOTE APP"`。与 stealth 进程伪装名一致——信用卡账单上看不出是面试工具。

### 6.10 邮箱绑定

License key 绑邮箱（Lemon Squeezy 收 email）。提供 `/recover-license/:email` endpoint 让用户重发 key（10 分钟限频防爆破）。

---

## 7. i18n 框架

### 7.1 三层语言

| 层 | 控制 | 谁定 |
|---|---|---|
| UI Language | 应用界面 / 菜单 / 错误 | 用户 Settings 选 |
| Audio Language | ASR 识别语言 | 新建会议时选 |
| Advice Language | LLM 提词输出语言 | **跟随 Audio Language**（硬绑定） |

### 7.2 默认值

- **UI Language**：跟随系统 locale（zh-* → 中文，其他 → 英文）
- **Audio Language**：绑定 UI language（UI 中文 → audio 默认中文，每场会议可改）

### 7.3 实现

前端 `react-i18next`，locale 文件：

```
src/i18n/locales/
├── zh-CN.json
└── en-US.json
```

字符串规范：
- 嵌套 key（`control.start` ✅）
- 时间 / 数字 / 日期用 interpolation
- 标点跟 locale（中文全角、英文半角）
- 单复数用 i18next plural

后端 Rust 用静态 string table（`once_cell::Lazy<HashMap>`）。用于通知 / 邮件。

### 7.4 邮件 i18n

License key 邮件、低余额邮件等都双语。Resend 模板按 license 创建时的 locale（Lemon Squeezy webhook 带 `locale` metadata）发送。

### 7.5 Landing page / 充值页

静态 HTML 托管 CF Pages。`?lang=zh-CN` query param 切换。MVP 不做子域 / 路径前缀。

### 7.6 不做的（v1.1+）

- 远程拉文案 hot reload
- 日语 / 韩语 UI（识别支持但 UI 翻译延后）
- RTL 语言
- Auto-detect audio language
- 复杂 plural rule（俄语 6 种）

---

## 8. 6 周路线图

### 8.1 工程量汇总

| Section | 工程量 |
|---|---|
| 2 架构骨架 + 抽象层 | 3 天 |
| 3 模板 + 简历 RAG + OCR | 4 天 |
| 4 音频管线 ScreenCaptureKit | 6.5 天 |
| 5 Stealth + 快捷键 + 窗口 | 7.5 天 |
| 6 License + Lemon Squeezy + 订阅生命周期 + auto top-up + prompt caching | 9 天 |
| 7 i18n | 5.5 天 |
| **合计** | **35.5 天 ≈ 7 周（全职）** |

### 8.2 Week 0（动 code 前并行准备）

- ✅ `confide.knosi.xyz` + `api.confide.knosi.xyz` 子域 DNS + Caddy 反代
- ✅ 确认 knosi.xyz DNS 是否在 Cloudflare（决定 Workers 域名绑定）
- ✅ `screencapturekit-rs` 100 行 demo 验证
- ✅ Anthropic API key（送 $5 试用，alpha 阶段不用、Week 5 切产用）
- ✅ Resend 账号
- ✅ Lemon Squeezy 注册（30 分钟）
- ✅ OpenAI API key + Realtime API access 验证
- ✅ 决定 Confide 最终产品名（codename 临时占位）

砍掉：~~Apple Developer Account~~（Week 5 申请）、~~Stripe Atlas~~（Lemon Squeezy 替代）、~~SenseVoice 模型 host~~（推 v1.1）。

### 8.3 路线图（每周 5 个工作日 = 30 天）

```
Week 1 [音频管线 + GPT-Realtime-Whisper]    目标：录音不需要 BlackHole
├── Day 1-2  ScreenCaptureKit 集成
├── Day 3    权限检查 + onboarding 弹窗
├── Day 4    GPT-Realtime-Whisper WebSocket 集成 + Worker token 签发
└── Day 5    白名单 + 蓝牙引导 + 移除 BlackHole

Week 2 [Stealth + 模板基座]                 目标：Zoom 共享屏幕中隐身
├── Day 6    LSUIElement + 菜单栏图标
├── Day 7    NSWindowSharingType + frameless 极简窗口
├── Day 8    全局快捷键 4 组
├── Day 9    "VoiceNote" 进程伪装打包 + 透明度调节
└── Day 10   测试矩阵：Zoom/Meet/Teams 共享 + 录屏

Week 3 [模板 + 简历 RAG]                    目标：拖入 PDF + 双语面试模板生效
├── Day 11   2 模板 × 双语 prompt 落地
├── Day 12   on_question_to_user 触发器 + 单测
├── Day 13   PDF/MD/TXT 拖入 + 文档绑定到 meeting
├── Day 14   PDF OCR fallback（pdf-extract → tesseract-rs）
└── Day 15   context_note 文本框 + 角色 persona UI

Week 4 [License 上半 + Lemon Squeezy 订阅]   目标：Free 注册 → Pro 订阅 → 月度 quota 工作
├── Day 16-17  CF Workers + Hono + KV + D1 脚手架
├── Day 18     /plan + /usage + license 验证
├── Day 19     LicenseManager 客户端 + 5 分钟 sync + 月度 quota 重置
└── Day 20     Lemon Squeezy 订阅集成 + webhook（created/renewed/cancelled/expired）

Week 5 [License 下半 + i18n + 切 Anthropic 直连 + Prompt Caching]
├── Day 21    Lemon Squeezy 邮件双语模板（Resend）+ 订阅续订邮件
├── Day 22    BYO key 模式 UI（Settings 内置，所有 plan 可切）
├── Day 23    LLM Proxy + 切 Anthropic 直连 + 启用 Prompt Caching + Audio Proxy 计量
├── Day 24    i18next + 抽 zh-CN / 翻 en-US
├── Day 25    Settings 语言选择器 + 邮件 locale 路由 + Auto top-up 开关 UI
└── 【并行】Day 21 提交 Apple Developer Individual 申请

Week 6 [收口 + 自验证]                      目标：自己能用 + 朋友能买 + 关键链路不挂
├── Day 26    Onboarding 完整流程（10min 试用）
├── Day 27    充值页静态站（pricing.html + Lemon hosted checkout）
├── Day 28    部署到 confide.knosi.xyz / api.confide.knosi.xyz
├── Day 29    自己跑通：注册→试用→买 1h→面试模板用→断网 1 天回来 sync
└── Day 30    .dmg 打包（Apple Dev 已批则签 + 公证；未批则 unsigned + 文档教用户右键 Open）
```

### 8.4 砍到 v1.x 的范围

| 砍掉 | 推到 |
|---|---|
| iOS 伴侣 app | v1.1 |
| Windows 支持 | v1.2 |
| macOS 12 BlackHole fallback | v1.1 或永不 |
| 模板编辑器（用户改 prompt） | v1.1 |
| 历史会议搜索 / 过滤 | v1.1 |
| 实时翻译（独立于 advice） | v1.2 |
| 团队工作区 / B2B | v2 |
| Auto-detect audio language | v1.1 |
| 远程拉文案 hot reload | v1.1 |
| 微信支付 / 支付宝 | v1.1 |
| 多设备硬绑定 | v1.1（监测中） |
| 简历优化（Pro/Ultra 月度配额） | **v1.0.5（首发后 2 周补）** |
| 行业题库（Ultra 专属） | v1.1 |
| 面试复盘报告 | v1.0.5 |
| 历史会议 markdown export UI 按钮 | v1.0.1 |
| 应用白名单用户可改 | v1.1 |
| 进程伪装策略 B（用户选名） | v1.1 |
| **本地 SenseVoice ASR**（保留 demo 代码作 v1.1 入口） | v1.1 |
| **Opus 4.7 模型分层** | v1.1 |

### 8.5 每周末"可演示状态"

| Week | 周末能演示 |
|---|---|
| W1 | macOS 13+ 双击 .dmg → 不装 BlackHole 即录到 Zoom 通话 |
| W2 | 录音中切 Stealth → 共享屏幕给同事 → 同事看不到 Confide |
| W3 | 拖入英文简历 PDF + 选 Job Interview EN → 模拟"Tell me about yourself" → AI 用简历给开头 |
| W4 | 注册 → 用 Lemon test mode 买 1h → 收 license email → 录音中扣余额 |
| W5 | 切 UI 中文 → 中文 license email 正常 → 切 BYO 用自己 OpenAI key |
| W6 | confide.knosi.xyz 全套买一遍走通；.dmg 给 3 个朋友试用 |

### 8.6 现实预期

你不是全职。**实际 6 周可能拖到 8-10 周**。Week 0 准备项（Apple/Lemon/域名/keys）等待时间不在你能 ship 时间内，必须 Week 0 并行启动。

---

## 9. 自验证策略 + Acceptance Criteria

### 9.1 7 个 dogfood 场景

详见附录 B。简略：

1. **英文面试**（≥2 页英文简历 + 朋友扮演面试官 + 5 个真实问题）
2. **中文面试**（同 1，中文）
3. **日常会议**（真实 1:1 + agenda 文档拖入）
4. **Stealth 测试**（Zoom 共享屏幕给朋友看是否暴露）
5. **付费链路**（注册→试用→Lemon test card→license email→录音扣余额→断网 1 天恢复）
6. **BYO 模式**（Settings 切到 BYO → 输入自己的 Anthropic/OpenAI key → 录 30 分钟 → 验 used_this_month_seconds 未变化）
7. **i18n 切换**（UI 中英切 + 邮件 locale 正确）

### 9.2 Acceptance Criteria

#### A. 安装与启动

- A1. macOS 13+ 双击 .dmg 完成首次启动
- A2. Onboarding ≤4 步（语言、Screen Recording 权限、麦克风权限、注册邮箱）
- A3. 不需要安装任何第三方驱动
- A3'. 不需要下载任何 ML 模型即可使用
- A4. unsigned dmg 时 onboarding 文档明确告知"右键 → Open"

#### B. 录音与转录

- B1. 系统音频 + 麦克风双通道捕获正确（transcript 区分 me/other）
- B2. 转录质量（GPT-Realtime-Whisper）：英文 WER ≤7%、中文 WER ≤10%、亚洲语种 ≤15%
- B3. 转录延迟 ≤500ms（含网络）
- B4. 60 分钟连续录音不崩溃 / 不掉段
- B5. CPU 占用 ≤15% on M1/M2 / ≤30% on Intel
- B6. 首次启动到能开始录音 ≤30 秒
- B7. OpenAI Realtime API outage 时弹错误，不静默失败

#### C. AI Advice

- C1. 面试模板触发率：检测到面试官提问 advice 出现率 ≥80%
- C2. 通用模板触发率：5 类条件触发 advice 出现率 ≥70%
- C3. Advice 引用简历率：≥80% 包含简历真实内容
- C4. Advice 长度：≤30 中文字 / ≤25 英文词
- C5. Advice 延迟：触发到显示 ≤2s（含 LLM）
- C6. Advice 语言：与 audio language 一致

#### D. Stealth

- D1. NSWindowSharingType 在 Zoom/Meet/Teams 共享屏幕时不可见
- D2. macOS `screencapture` 命令不抓到
- D3. Activity Monitor 进程显示 "VoiceNote"
- D4. 不出现在 Dock / Cmd+Tab
- D5. ⌘⇧H toggle / ⌘⇧K panic / ⌘⇧↑↓ 透明度
- D6. 面试模板默认 stealth ON / 日常会议默认 OFF

#### E. 模板与简历 RAG

- E1. 内置 4 个模板（job-interview × zh+en、general-meeting × zh+en）
- E2. 拖入 PDF/MD/TXT 后 ≤5s 处理（text-based）
- E3. OCR-needed PDF 降级 ≤30s
- E4. 简历内容能被 advice 引用（参考 C3）
- E5. context_note ≤500 字限制 + 字符计数

#### F. License 与计费（订阅模式）

- F1. 注册即得 Free plan，10 min/月含量
- F2. Free 含量用完后录音停 + 弹升级 Pro 提示
- F3. Lemon Squeezy 订阅成功后 license email ≤5 分钟到达
- F4. License key 输入后 plan 信息立即显示（tier、含量、超额单价）
- F5. 录音中 5 分钟 sync 一次
- F6. 离线 7 天可继续录音，第 8 天强制 sync
- F7. 多设备同 license 不冲突（5 分钟 active session lease）
- F8. BYO key 模式录音不计入 used_this_month_seconds
- F9. 月度续订时 used_this_month_seconds reset 为 0
- F10. Pro/Ultra 含量用完且 auto top-up 开启时进入超额模式（继续录音、按 plan 单价计费、Lemon Squeezy 自动 charge）
- F11. Auto top-up 关闭时含量用完弹错误 + 提示充值
- F12. 取消订阅后剩余含量本月仍可用、月底转 Free
- F13. Free 用户拖入 PDF/MD/TXT → UI 提示"简历 RAG 是 Pro 功能" + 升级链接
- F14. Pro/Ultra 简历优化 credits 月度 reset（v1.0.5 实现完后验证）

#### G. i18n

- G1. 系统 locale = zh-* 时默认中文 UI
- G2. UI 切语言后无 `[missing]` 占位符
- G3. 邮件按 license 创建时 locale 发送
- G4. 模板按 audio language 加载

#### H. 隐私与安全

- H1. 音频数据不离开本机（仅 transcript 文本走 LLM/ASR）—— 注：MVP 默认走云 ASR，audio stream 经 OpenAI 处理，但 OpenAI 配置 `do not retain`
- H2. 用户简历 PDF 仅本机存储 + 临时 chunk 进 LLM prompt
- H3. License email from 用 Resend DKIM 签名通过
- H4. Stripe statement descriptor "VOICENOTE APP"

#### I. 质量

- I1. 7 个自验证场景全通过
- I2. 至少 5 个外部 alpha 用户跑通完整付费链路
- I3. 关键路径无 panic / crash
- I4. 1 小时录音内存增长 ≤50MB

### 9.3 MVP 完成定义

> 你自己能在 6-8 周内：用 Confide 完成一次真实英文面试 + 一次真实 1:1，stealth 不漏陷、付费链路不丢钱、5 个朋友试用反馈正面。

---

## 10. 风险清单 + 总览

### 10.1 P0（可能让 MVP 流产）

| # | 风险 | 缓解 |
|---|---|---|
| R1 | `screencapturekit-rs` crate bug 或不维护 | Week 0 100 行 demo 验证；fallback 到 `objc2` 直接 binding |
| R2 | llmgate 离职 / 封禁，付费用户断服 | Week 5 强切 Anthropic 直连；LLM Provider 抽象层让切换 = 改配置 |
| R3 | OpenAI Realtime API outage | 单引擎无 fallback；客户端弹错误 + 主动邮件道歉；保留 1 天工作量储备紧急切 Deepgram |
| R4 | Lemon Squeezy 审核拒 | Week 0 立即注册；备用方案 Paddle |
| R5 | Apple Dev 申请超 1 周 | 接受推延 Week 7；Week 6 给朋友的 dmg 可 unsigned |
| R6 | 你字节工作太忙，每周给 Confide 时间不足 | 接受现实，用 8 周做心理预期 |
| R7 | GPT-Realtime-Whisper 刚发布 2 天有未知 bug | Week 1 集成跑 100 小时测试 |

### 10.2 P1（影响产品质量）

| # | 风险 | 缓解 |
|---|---|---|
| R8 | 新加坡到 OpenAI 延迟超 1s | Week 1 测；超阈值切 OpenAI Asia endpoint 或 CF 边缘加速 |
| R9 | Anthropic API 在新加坡延迟 ≥1.5s | Week 5 测；超阈值 fallback OpenAI |
| R10 | PDF OCR 慢（30s+） | Week 3 测；不行先关 OCR、要求 text-based |
| R11 | ScreenCaptureKit 不抓蓝牙耳机音 | Onboarding 警告 + 切内置扬声器引导 |
| R12 | Karat / HireVue 反作弊检测 | 列为不支持平台 |
| R13 | Lemon Squeezy 6.5% 手续费 | 接受；订阅 + Prompt Caching 让平均毛利仍达 80% |
| R14 | macOS 13+ 砍 ~15% 用户 | 接受；v1.1 加 macOS 12 |
| R15 | 嘈杂环境转录质量崩 | VAD 阈值调；settings 加噪音抑制强度 |
| R16 | OpenAI 价格上涨 | 涨幅 ≤30% 吸收；>30% 调价 |
| R17 | "100% 本地"叙事弱化（云 ASR） | landing page 改 "audio not stored, transcripts deleted in 24h" |
| R18 | v1.0.5 简历优化 2 周内交付压力 | 接受承诺；首发充值页明确"v1.0.5 推出" |
| R19 | 订阅 churn 率高 | 月度续订 webhook 监测；用户取消时弹问卷收集原因；Pro→Free 不强阻断 |
| R20 | Free 用户白嫖 ASR 成本（1000 用户 ~$50/月） | 视为 PLG 营销支出；监测 Free→Pro 转化率，<1% 时考虑收紧 |
| R21 | Auto top-up 误扣引发投诉 | 默认开启但 Settings 显眼可关；超额前 3 分钟内主动通知；首次进入超额模式额外弹确认 |
| R22 | Lemon Squeezy 订阅取消 webhook 漏接 | 客户端启动时调 /plan 主动 sync 一次，比 webhook 更可靠 |

### 10.3 P2（监测、不影响 MVP）

| # | 风险 | 缓解 |
|---|---|---|
| R19 | License key 多设备滥用 | 3 层软防护，监测但不封 |
| R20 | 用户简历泄露给 LLM | privacy policy 标清楚 |
| R21 | Stealth 摄像头反光暴露 | landing page 警告 |
| R22 | CF Workers 超额 | 转付费 $5/月 |
| R23 | License email 没收到 | Lemon Squeezy 内置重发 |
| R24 | Anthropic / OpenAI 因 Confide 含面试用例审计 | 备用 provider |
| R25 | 公司 Mac 跑 Confide 被 IT 检测 | Onboarding 警告 |
| R26 | 复用 knosi.xyz 影响品牌 / SEO | Producthunt 前切独立域 |

### 10.4 全局未决项

| # | 项 | MVP 默认 | 上线前必决 |
|---|---|---|---|
| O1 | Confide 最终产品名 | "Confide" 占位 | ✅ PH 上线前 |
| O2 | 独立域名 | confide.knosi.xyz 占位 | ✅ PH 上线前 |
| O3 | App Store 上不上 | 不上（直接 .dmg） | 维持 |
| O4 | 隐私政策 / EULA | 没写 | ✅ MVP 出版前用 GPT 起草 + 律师过 |
| O5 | landing page 工具栈 | 静态 HTML | Week 6 决定 Astro / Next |
| O6 | 首批种子用户名单 | 没列 | Week 0 列 10 个目标用户 |
| O7 | logo 和 brand colors | 没设计 | Week 4-5 用 Figma + Claude / Fiverr $50 |
| O8 | 客服邮箱 + 响应 SLA | hello@confide.knosi.xyz 24h | 维持 |
| O9 | 退款政策 | "7 天退款" 行业标准 | 写在 EULA + 充值页 |
| O10 | analytics 选型 | 没装 | PostHog 自托管 / Plausible / 不装 |

### 10.5 v1.x 路线图（不在 MVP 范围）

**v1.0.1（上线后 1-2 周）**
- 修高频 bug
- 历史会议 markdown export UI 按钮
- transcript 拷贝 / 搜索基础

**v1.0.5（1-2 个月）**
- 简历优化（Pro 5/月、Ultra 15/月）
- 面试复盘报告（Pro/Ultra）
- 应用白名单用户可改

**v1.1（3-4 个月）**
- macOS 12 BlackHole fallback
- iOS 伴侣 app（提词第二屏）
- 微信支付 / 支付宝
- 模板编辑器
- Auto-detect audio language
- 多设备硬绑定（如果监测发现滥用）
- **本地 SenseVoice ASR**（保留 demo 代码作入口）
- **Opus 4.7 模型分层 + Advanced Settings**
- 行业面试题库（500+ 真题）

**v1.2（5-6 个月）**
- Windows 支持（WASAPI loopback）
- 实时翻译（独立于 advice）
- 更多语言模板（日韩越南菲律宾）
- 团队工作区雏形

**v2.0（>6 个月）**
- B2B 团队订阅
- SSO / SCIM
- 企业版本地部署 LLM
- API 给第三方

---

## 附录 A：模板 prompt 草稿

### A.1 `job-interview-zh.json`

```json
{
  "id": "job-interview-zh",
  "name": "面试模式（中文）",
  "description": "你是候选人，正在被面试官提问。AI 帮你组织回答。",
  "language": "zh-CN",
  "role_persona_template": "我是 {{job_title}}，{{years}} 年经验，正在面试 {{target_company}} 的 {{target_role}} 岗位。我的主要技能：{{skills}}。我最近做过的项目：{{recent_project}}。",
  "system_prompt": "你是面试候选人的实时教练。用户正在被面试官提问。\n\n你的任务：基于面试官刚问的问题 + 用户的简历背景，生成一句用户可以直接说出口的回答开头（≤30 字）。\n\n要求：\n- 必须是回答问题的 opening，不是完整答案——用户会自己延展\n- 必须用第一人称\n- 必须引用简历中的具体项目/数字/技术栈\n- 用 STAR 框架时只给 Situation+Task 的开头\n- 不允许铺垫（'这是个好问题'之类）、不允许解释你为什么这么建议\n\n严格按以下格式输出，每项一行:\n建议：（一句话回答开头，≤30 字）\n角度：（2-4 字标签，如 '项目经验/技术深度/方法论/成果数据'）\n\nReply only in Chinese.",
  "trigger_config": {
    "on_ask_opinion": false,
    "on_question_to_user": true,
    "on_decision_point": false,
    "on_discussion_stuck": true,
    "custom_keywords": [
      "你能介绍一下", "讲讲你", "说说你",
      "为什么选择", "为什么离开", "为什么想",
      "你的优势", "你的劣势", "你最大的",
      "举个例子", "具体说说", "怎么解决的",
      "如果", "假如", "遇到这种情况"
    ]
  },
  "advice_cooldown_seconds": 8,
  "stealth_default": true
}
```

### A.2 `job-interview-en.json`

```json
{
  "id": "job-interview-en",
  "name": "Job Interview (English)",
  "description": "You're being interviewed. AI helps you frame your answers.",
  "language": "en-US",
  "role_persona_template": "I'm a {{job_title}} with {{years}} years of experience, interviewing for {{target_role}} at {{target_company}}. My core skills: {{skills}}. Recent project: {{recent_project}}.",
  "system_prompt": "You are a real-time interview coach. The user is being asked a question by an interviewer.\n\nYour task: based on the interviewer's question + the user's resume background, generate ONE sentence the user can speak directly as the opening of their answer (≤25 words).\n\nRules:\n- It must be the OPENING of an answer, not a full response—the user will continue from there\n- First person only\n- Must reference a concrete project/number/technology from the user's resume\n- For STAR-framework questions, give the Situation+Task opener only\n- No filler ('Great question…'), no meta-commentary, no quotes around the answer\n\nOutput exactly two lines:\nAdvice: (one speakable sentence, ≤25 words)\nAngle: (2-4 word tag, e.g. 'project depth / metrics / leadership / problem-solving')\n\nReply only in English.",
  "trigger_config": {
    "on_ask_opinion": false,
    "on_question_to_user": true,
    "on_decision_point": false,
    "on_discussion_stuck": true,
    "custom_keywords": [
      "tell me about", "walk me through", "describe a time",
      "why did you", "why do you want", "why are you",
      "what's your", "what are your", "your biggest",
      "give me an example", "for example", "how did you handle",
      "what would you do if", "imagine", "in a situation where"
    ]
  },
  "advice_cooldown_seconds": 8,
  "stealth_default": true
}
```

### A.3 `general-meeting-zh.json` / `general-meeting-en.json`

5 类触发器全开，cooldown 12 秒，stealth 默认 OFF。详细 prompt 在实施时落地（保留你 demo 现有 system prompt 框架，调整文案 + 加 expertise_context 注入）。

---

## 附录 B：自验证场景细则

### B.1 英文面试

**步骤**：朋友扮演面试官 + 你拖入英文简历 PDF + 选 Job Interview EN + 录音 + 5 个真实问题（"Walk me through your most challenging project"、"Tell me about a time you disagreed with your manager"、"Why do you want to leave Bytedance"、"What's your biggest weakness"、"Do you have any questions for me"）。

**通过标准**：5/5 触发 + ≥4/5 引用简历真实内容 + 全英文 + ≤30 字 + 触发延迟 ≤2s + 60 分钟不崩。

### B.2 中文面试 / B.3 日常会议 / B.4 Stealth / B.5 付费链路 / B.6 BYO 模式 / B.7 i18n

详略，按 Section 9 acceptance criteria 一一对应验证。

---

## 附录 C：决策日志

| 决策 | 日期 | 备选 | 选择 | 理由 |
|---|---|---|---|---|
| 域名 | 2026-05-09 | 独立域名 / knosi 子域 | knosi 子域 MVP，PH 前切独立 | 0 现金支出 + 保留干净退出 |
| 支付 | 2026-05-09 | Stripe Atlas / Lemon Squeezy / Paddle | Lemon Squeezy | MoR 模式省税务 + 内置 license + 砍 5 天工程 |
| ASR | 2026-05-09 | Deepgram / Soniox / Whisper / 双引擎 | GPT-Realtime-Whisper 单引擎 | 简化架构 + SOTA 模型 + 多语言全覆盖 |
| LLM | 2026-05-09 | Sonnet only / Sonnet + Opus | Sonnet only（Opus v1.1） | Advice 短输出 Opus 优势小；llmgate alpha 期只批 Sonnet |
| 试用时长 | 2026-05-09 | 30min / 10min | 10min | 小试用逼真实场景付费 |
| 定价模式 | 2026-05-09 | 时长包 / 订阅 / 混合 | **订阅 3 档**（Free/Pro/Ultra） | 订阅 MRR 可预测 + 毛利从 60% 升到 80% + PLG Free 流量入口 |
| Free 简历 RAG | 2026-05-09 | 开启 / 禁用 | 禁用 | 控白嫖成本 ($0.21→$0.05/用户/月) + 升级压力 |
| Prompt Caching | 2026-05-09 | 启用 / 不启用 | 启用 | 简历 RAG 重复使用场景 LLM 成本降 80% |
| BYO 模式 | 2026-05-09 | $29 一次性 SKU / 内置所有 plan | 内置所有 plan | 砍 SKU 复杂度 + 技术用户口碑放大器 |
| Stealth | 2026-05-09 | 做 / 不做 / 隐晦 | 做 + 隐晦双叙事 | 必须满足面试场景 + 不被 Reddit 钉死 |
| 模板范围 | 2026-05-09 | 5 个 / 2 个 | 2 个（job-interview + general-meeting） | MVP 越简单越好 |
| 简历投喂 | 2026-05-09 | 表单字段化 / 整段文本 / PDF + context_note | PDF + context_note | 跟竞品做法一致 + 省工程 |
| 本地 ASR | 2026-05-09 | 默认本地 / 默认云 / 双轨 | 默认云，本地推 v1.1 | 砍掉 200MB 下载漏斗杀手 |
| Apple Dev | 2026-05-09 | Week 0 / Week 5 / 永不 | Week 5 申请 | 验证完核心技术再投钱 |
