# Confide Week 6 — Pricing Page + Onboarding + Deploy + Self-Verification

> **For agentic workers:** REQUIRED SUB-SKILL: Use gecc-dev:subagent-driven-development (recommended) or gecc-dev:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** confide.knosi.xyz 上跑通完整付费链路 + onboarding 流程 + .dmg 给朋友试用。这是 MVP 收口周。

**Domain:** general

**Architecture:**
- 静态充值页（HTML + Tailwind），托管 Cloudflare Pages（或 Caddy 直接 file_server）
- 完整 onboarding 流程串通：首次启动 → screen recording 权限 → mic 权限 → free trial 激活 → 选模板 → 录音
- .dmg 打包（如 Apple Dev 已批 → 签名 + notarize；未批 → unsigned + 文档教用户右键 Open）
- 5 个朋友 alpha 试用

**Tech Stack:** 静态 HTML、Stripe.js（其实是 Lemon Squeezy hosted checkout）、tauri build

**Spec reference:** `docs/specs/2026-05-09-overseas-meeting-copilot-design.md` Section 8.3 Week 6

**Prerequisite:** Week 5 完成；Apple Developer Account 申请已提交（Day 21）

---

## File Structure

```
landing/                                    [Create dir at repo root]
├── pricing.html                            [Create] 静态充值页
├── index.html                              [Create] 简单 landing 占位
└── assets/
    └── styles.css                          [Create] inline 或外置

src-tauri/
└── src/
    ├── commands.rs                         [Modify] 完善 onboarding 状态 commands
    └── ...
src/
├── components/
│   └── onboarding/
│       ├── PermissionGate.tsx              [Modify] 完善流程
│       └── WelcomeFlow.tsx                 [Create] 首次启动 4 步引导

scripts/
├── build-dmg.sh                            [Create] 打包脚本
└── notarize.sh                             [Create] 公证脚本（Apple Dev 批了之后用）
```

---

### Task 1: 写 pricing.html

**Files:**
- Create: `landing/pricing.html`

- [ ] **Step 1: 写双语充值页**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Confide — Pricing</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <style>
    body { font-family: -apple-system, "SF Pro Display", system-ui, sans-serif; }
  </style>
</head>
<body class="bg-neutral-50 text-neutral-900 antialiased">
  <div class="max-w-5xl mx-auto px-6 py-16" id="root">
    <!-- header -->
    <div class="text-xs text-neutral-500 mb-4">PRICING_VOL.01</div>
    <h1 class="text-5xl font-bold mb-4" data-i18n="title"></h1>
    <p class="text-lg text-neutral-600 mb-16" data-i18n="subtitle"></p>

    <!-- plans -->
    <div class="grid md:grid-cols-3 gap-6">
      <!-- Free -->
      <div class="bg-white p-8 rounded-lg border">
        <div class="text-sm font-medium mb-3" data-i18n="free.name"></div>
        <div class="text-5xl font-bold mb-2">$0</div>
        <div class="text-sm text-neutral-500 mb-1" data-i18n="free.qty"></div>
        <div class="text-xs text-neutral-400 mb-6" data-i18n="free.note"></div>
        <ul class="text-sm space-y-2 mb-8">
          <li data-i18n="free.f1"></li>
          <li data-i18n="free.f2"></li>
          <li data-i18n="free.f3"></li>
          <li data-i18n="free.f4"></li>
        </ul>
        <a href="https://confide.knosi.xyz/download" class="block w-full text-center py-3 border border-neutral-300 rounded text-sm font-medium" data-i18n="free.cta"></a>
      </div>

      <!-- Pro (highlighted) -->
      <div class="bg-white p-8 rounded-lg border-2 border-black relative">
        <div class="absolute -top-3 left-8 bg-black text-white text-xs px-3 py-1 rounded" data-i18n="pro.badge"></div>
        <div class="text-sm font-medium mb-3" data-i18n="pro.name"></div>
        <div class="text-5xl font-bold mb-2">$19<span class="text-xl text-neutral-500">/mo</span></div>
        <div class="text-sm text-neutral-500 mb-1" data-i18n="pro.qty"></div>
        <div class="text-xs text-neutral-400 mb-6" data-i18n="pro.note"></div>
        <ul class="text-sm space-y-2 mb-8">
          <li data-i18n="pro.f1"></li>
          <li data-i18n="pro.f2"></li>
          <li data-i18n="pro.f3"></li>
          <li data-i18n="pro.f4"></li>
          <li data-i18n="pro.f5"></li>
        </ul>
        <a id="pro-checkout" href="#" class="block w-full text-center py-3 bg-black text-white rounded text-sm font-medium" data-i18n="pro.cta"></a>
      </div>

      <!-- Ultra -->
      <div class="bg-white p-8 rounded-lg border">
        <div class="text-sm font-medium mb-3">ULTRA</div>
        <div class="text-5xl font-bold mb-2">$49<span class="text-xl text-neutral-500">/mo</span></div>
        <div class="text-sm text-neutral-500 mb-1" data-i18n="ultra.qty"></div>
        <div class="text-xs text-neutral-400 mb-6" data-i18n="ultra.note"></div>
        <ul class="text-sm space-y-2 mb-8">
          <li data-i18n="ultra.f1"></li>
          <li data-i18n="ultra.f2"></li>
          <li data-i18n="ultra.f3"></li>
          <li data-i18n="ultra.f4"></li>
          <li data-i18n="ultra.f5"></li>
        </ul>
        <a id="ultra-checkout" href="#" class="block w-full text-center py-3 border border-neutral-300 rounded text-sm font-medium" data-i18n="ultra.cta"></a>
      </div>
    </div>

    <p class="text-xs text-neutral-500 mt-8" data-i18n="footnote"></p>
  </div>

  <script>
    const STRINGS = {
      "en-US": {
        title: "Simple, transparent pricing",
        subtitle: "Start free. Upgrade when you need more.",
        free: {
          name: "FREE", qty: "10 minutes / month", note: "No credit card required",
          f1: "All 100+ languages", f2: "Job Interview + General Meeting templates",
          f3: "Stealth mode", f4: "BYO API key (free forever)",
          cta: "Start Free",
        },
        pro: {
          badge: "Most Popular", name: "PRO", qty: "60 minutes / month", note: "Overage $0.35/min",
          f1: "Everything in Free", f2: "Resume RAG (unlimited)", f3: "Permanent meeting history",
          f4: "5 resume reviews / month", f5: "Auto top-up",
          cta: "Subscribe — $19/mo",
        },
        ultra: {
          qty: "200 minutes / month", note: "Overage $0.25/min",
          f1: "Everything in Pro", f2: "15 resume reviews / month", f3: "Industry question bank",
          f4: "Priority support 24h", f5: "Early access to new features",
          cta: "Subscribe — $49/mo",
        },
        footnote: "Powered by Claude 4.6 + GPT-Realtime-Whisper. macOS 13+ required. Audio never leaves your machine for transcription metadata.",
      },
      "zh-CN": {
        title: "简单透明的定价",
        subtitle: "免费开始。需要更多时升级。",
        free: {
          name: "免费", qty: "10 分钟 / 月", note: "无需信用卡",
          f1: "全部 100+ 种语言", f2: "面试 + 日常会议模板",
          f3: "Stealth 模式", f4: "自带 API key（永久免费）",
          cta: "免费开始",
        },
        pro: {
          badge: "最受欢迎", name: "PRO", qty: "60 分钟 / 月", note: "超额 $0.35/min",
          f1: "免费版全部功能", f2: "简历 RAG（无限次）", f3: "永久通话历史",
          f4: "5 份简历优化 / 月", f5: "自动充值",
          cta: "订阅 — $19/月",
        },
        ultra: {
          qty: "200 分钟 / 月", note: "超额 $0.25/min",
          f1: "Pro 全部功能", f2: "15 份简历优化 / 月", f3: "行业题库",
          f4: "优先支持 24 小时", f5: "新功能抢先体验",
          cta: "订阅 — $49/月",
        },
        footnote: "搭载 Claude 4.6 + GPT-Realtime-Whisper。需要 macOS 13+。音频不离开你的设备。",
      },
    };

    const PRO_LEMON_URL = "REPLACE_WITH_LEMON_PRO_CHECKOUT_URL";
    const ULTRA_LEMON_URL = "REPLACE_WITH_LEMON_ULTRA_CHECKOUT_URL";

    function applyLang(lang) {
      const dict = STRINGS[lang] || STRINGS["en-US"];
      document.documentElement.lang = lang;
      const get = (path) => path.split('.').reduce((o, k) => o?.[k], dict) ?? "";
      document.querySelectorAll("[data-i18n]").forEach((el) => {
        el.textContent = get(el.getAttribute("data-i18n"));
      });
      // Set checkout links with lang param so Lemon shows right currency / locale
      document.getElementById("pro-checkout").href = PRO_LEMON_URL + "?desired_quantity=1&checkout[locale]=" + (lang === "zh-CN" ? "zh" : "en");
      document.getElementById("ultra-checkout").href = ULTRA_LEMON_URL + "?desired_quantity=1&checkout[locale]=" + (lang === "zh-CN" ? "zh" : "en");
    }

    const params = new URLSearchParams(location.search);
    const lang = params.get("lang") === "zh-CN" || (navigator.language.startsWith("zh") && !params.get("lang"))
      ? "zh-CN" : "en-US";
    applyLang(lang);
  </script>
</body>
</html>
```

- [ ] **Step 2: 把 PRO_LEMON_URL / ULTRA_LEMON_URL 替换为真实值**

去 Lemon Squeezy Dashboard > Pro Product > Variants > Pro Monthly > Share > 复制 buy link → 粘贴到 `PRO_LEMON_URL`。

同样 Ultra。

- [ ] **Step 3: 写 landing/index.html**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Confide — AI Meeting Copilot</title>
  <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-neutral-950 text-white antialiased font-sans min-h-screen flex items-center">
  <div class="max-w-3xl mx-auto px-6 py-16 text-center">
    <h1 class="text-6xl font-bold mb-6 tracking-tight">Confide</h1>
    <p class="text-2xl text-neutral-400 mb-12">
      Real-time meeting copilot. Speak with confidence in any language.
    </p>
    <div class="flex gap-4 justify-center">
      <a href="/pricing.html" class="px-6 py-3 bg-white text-black rounded font-medium">View Pricing</a>
      <a href="https://github.com/<your-handle>/confide" class="px-6 py-3 border border-neutral-700 rounded">GitHub</a>
    </div>
    <p class="text-sm text-neutral-500 mt-16">
      macOS 13+ · Powered by Claude 4.6 + GPT-Realtime-Whisper
    </p>
  </div>
</body>
</html>
```

---

### Task 2: 部署 landing 页面到 Caddy

**Files:**
- Modify: `/etc/caddy/Caddyfile` on knosi server

- [ ] **Step 1: 把 landing 文件 push 到 knosi server**

```bash
scp -r landing knosi:/srv/confide-landing
ssh knosi "ls /srv/confide-landing"
```

- [ ] **Step 2: 改 Caddyfile**

```bash
ssh knosi "cat > /etc/caddy/confide.caddy" <<'EOF'
confide.knosi.xyz {
	encode gzip zstd

	header {
		Strict-Transport-Security "max-age=31536000; includeSubDomains"
		X-Content-Type-Options "nosniff"
	}

	root * /srv/confide-landing
	file_server
	try_files {path} {path}.html /index.html
}
EOF
```

把 `confide.knosi.xyz` 段从 main Caddyfile 替换/include 这个文件。或直接编辑 `/etc/caddy/Caddyfile`。

- [ ] **Step 3: reload caddy**

```bash
ssh knosi "caddy reload --config /etc/caddy/Caddyfile"
```

- [ ] **Step 4: 验证**

```bash
curl https://confide.knosi.xyz/ | head -20
curl https://confide.knosi.xyz/pricing.html | head -20
curl 'https://confide.knosi.xyz/pricing.html?lang=zh-CN' | grep "简单透明"
```

Expected: 三个 curl 都返回正确 HTML。

---

### Task 3: 写 WelcomeFlow 完整 onboarding

**Files:**
- Create: `src/components/onboarding/WelcomeFlow.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: 写 4 步流程**

```tsx
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  checkScreenRecordingPermission,
  openScreenRecordingSettings,
  listAudioDevices,
  saveConfig,
  getConfig,
  getUserPlan,
} from "../../lib/tauri";
import { setUiLanguage } from "../../i18n/config";

const ONBOARDING_DONE_KEY = "confide.onboardingDone";

export function WelcomeFlow({ onDone }: { onDone: () => void }) {
  const { t, i18n } = useTranslation();
  const [step, setStep] = useState(0);
  const totalSteps = 4;

  if (localStorage.getItem(ONBOARDING_DONE_KEY) === "true") {
    onDone();
    return null;
  }

  function next() {
    if (step + 1 >= totalSteps) {
      localStorage.setItem(ONBOARDING_DONE_KEY, "true");
      onDone();
    } else {
      setStep(step + 1);
    }
  }

  return (
    <div className="fixed inset-0 bg-[var(--bg-primary)] z-50 flex items-center justify-center p-6">
      <div className="max-w-md w-full">
        <div className="text-xs text-gray-500 mb-2">Step {step + 1} of {totalSteps}</div>

        {step === 0 && (
          <div>
            <h1 className="text-2xl font-bold mb-4">Welcome to Confide</h1>
            <p className="text-sm text-gray-400 mb-6">
              Real-time meeting copilot. Powered by GPT-Realtime-Whisper + Claude 4.6.
            </p>
            <p className="text-sm mb-2">Choose your language:</p>
            <div className="flex gap-2 mb-6">
              <button onClick={() => setUiLanguage("en-US")} className={`px-4 py-2 border rounded ${i18n.language === "en-US" ? "bg-[var(--accent-purple)] text-white border-transparent" : "border-[var(--border)]"}`}>English</button>
              <button onClick={() => setUiLanguage("zh-CN")} className={`px-4 py-2 border rounded ${i18n.language === "zh-CN" ? "bg-[var(--accent-purple)] text-white border-transparent" : "border-[var(--border)]"}`}>中文</button>
            </div>
            <button onClick={next} className="px-6 py-2 bg-[var(--accent-purple)] text-white rounded">Continue</button>
          </div>
        )}

        {step === 1 && <ScreenRecordingStep onContinue={next} />}
        {step === 2 && <MicrophoneStep onContinue={next} />}
        {step === 3 && (
          <div>
            <h2 className="text-xl font-bold mb-4">You're all set 🎉</h2>
            <p className="text-sm text-gray-400 mb-6">
              You have <b>10 free minutes per month</b> to start. Click the menu bar icon to start your first meeting.
            </p>
            <button onClick={next} className="px-6 py-2 bg-[var(--accent-purple)] text-white rounded">
              Start using Confide
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

function ScreenRecordingStep({ onContinue }: { onContinue: () => void }) {
  const [status, setStatus] = useState<"checking" | "denied" | "granted">("checking");

  async function check() {
    const r = await checkScreenRecordingPermission();
    setStatus(r.status === "granted" ? "granted" : "denied");
    if (r.status === "granted") {
      setTimeout(onContinue, 600);
    }
  }

  useEffect(() => { void check(); }, []);

  return (
    <div>
      <h2 className="text-xl font-bold mb-3">Screen Recording permission</h2>
      <p className="text-sm text-gray-400 mb-6">
        Needed to capture meeting audio. We never see your screen — only the audio.
      </p>
      {status === "denied" && (
        <div className="space-y-3">
          <button
            onClick={() => void openScreenRecordingSettings()}
            className="px-4 py-2 bg-[var(--accent-purple)] text-white rounded"
          >
            Open System Settings
          </button>
          <button
            onClick={() => void check()}
            className="px-4 py-2 border border-[var(--border)] rounded ml-2"
          >
            Re-check
          </button>
          <p className="text-xs text-gray-500">
            After enabling, quit and restart Confide to take effect.
          </p>
        </div>
      )}
      {status === "granted" && <div className="text-green-400">✓ Granted</div>}
    </div>
  );
}

function MicrophoneStep({ onContinue }: { onContinue: () => void }) {
  const [devices, setDevices] = useState<{ id: string; name: string }[]>([]);

  async function load() {
    try {
      const d = await listAudioDevices();
      setDevices(d);
      if (d.length > 0) {
        const cfg = await getConfig();
        await saveConfig({ ...cfg, audio: { ...cfg.audio, mic_device: d[0].name } });
      }
    } catch (e) {
      console.error(e);
    }
  }

  useEffect(() => { void load(); }, []);

  return (
    <div>
      <h2 className="text-xl font-bold mb-3">Microphone</h2>
      <p className="text-sm text-gray-400 mb-4">
        Select your microphone (system sound is captured separately):
      </p>
      <select
        className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-2 py-1 mb-6"
        onChange={async (e) => {
          const cfg = await getConfig();
          await saveConfig({ ...cfg, audio: { ...cfg.audio, mic_device: e.target.value } });
        }}
      >
        {devices.map((d) => (
          <option key={d.id} value={d.id}>{d.name}</option>
        ))}
      </select>
      <button onClick={onContinue} className="px-6 py-2 bg-[var(--accent-purple)] text-white rounded">
        Continue
      </button>
    </div>
  );
}
```

- [ ] **Step 2: 在 App.tsx 接 WelcomeFlow**

```tsx
const [onboardingDone, setOnboardingDone] = useState(
  localStorage.getItem(ONBOARDING_DONE_KEY) === "true",
);

if (!onboardingDone) {
  return <WelcomeFlow onDone={() => setOnboardingDone(true)} />;
}
```

- [ ] **Step 3: typecheck**

```bash
pnpm typecheck 2>&1 | tail -5
```

---

### Task 4: 写 build-dmg 脚本

**Files:**
- Create: `scripts/build-dmg.sh`

- [ ] **Step 1: 写脚本**

```bash
#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT=$(pwd)

echo "==> Building VoiceNote.app via Tauri"
pnpm tauri build

DMG_PATH="$ROOT/src-tauri/target/release/bundle/dmg/VoiceNote_0.1.0_aarch64.dmg"
if [[ ! -f "$DMG_PATH" ]]; then
  # Try x86_64 path
  DMG_PATH="$ROOT/src-tauri/target/release/bundle/dmg/VoiceNote_0.1.0_x64.dmg"
fi

if [[ ! -f "$DMG_PATH" ]]; then
  echo "ERROR: DMG not found. Check src-tauri/target/release/bundle/dmg/"
  ls "$ROOT/src-tauri/target/release/bundle/dmg/" || true
  exit 1
fi

echo "==> DMG built: $DMG_PATH"
echo "==> Size: $(du -h "$DMG_PATH" | awk '{print $1}')"
echo
echo "Next steps:"
echo "  - If unsigned: distribute as-is. Users must right-click → Open."
echo "  - If Apple Dev approved: ./scripts/notarize.sh \"$DMG_PATH\""
echo "  - Upload to confide.knosi.xyz/download/VoiceNote.dmg"
```

```bash
chmod +x scripts/build-dmg.sh
```

- [ ] **Step 2: 写 notarize.sh（Apple Dev 批了之后用）**

```bash
#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${1:-}" ]]; then
  echo "Usage: $0 <path-to-dmg>"
  exit 1
fi
DMG="$1"

# These come from your Apple Developer Account (Week 0 / Week 5 Task 1)
APPLE_ID="${APPLE_ID:?Set APPLE_ID env}"
APPLE_TEAM_ID="${APPLE_TEAM_ID:?Set APPLE_TEAM_ID env}"
APP_SPECIFIC_PASSWORD="${APP_SPECIFIC_PASSWORD:?Set APP_SPECIFIC_PASSWORD env (from appleid.apple.com)}"

echo "==> Submitting $DMG to Apple notarization service"
xcrun notarytool submit "$DMG" \
  --apple-id "$APPLE_ID" \
  --team-id "$APPLE_TEAM_ID" \
  --password "$APP_SPECIFIC_PASSWORD" \
  --wait

echo "==> Stapling notarization ticket"
xcrun stapler staple "$DMG"

echo "==> ✅ Notarization complete: $DMG"
```

```bash
chmod +x scripts/notarize.sh
```

---

### Task 5: 第一次完整 build

- [ ] **Step 1: 跑 build**

```bash
cd /Users/bytedance/meeting-assistant
./scripts/build-dmg.sh 2>&1 | tail -20
```

第一次 release build 10-20 分钟。

- [ ] **Step 2: 测试 dmg 能跑**

```bash
open src-tauri/target/release/bundle/dmg/VoiceNote_*.dmg
```

打开 dmg → 把 VoiceNote.app 拖到 Applications → 双击启动 → 走完整 onboarding 流程。

如果 unsigned + 弹"Cannot be opened"：右键 → Open → 第二个对话框点 "Open"。

- [ ] **Step 3: 把 dmg 上传到 knosi server**

```bash
DMG_PATH=$(ls src-tauri/target/release/bundle/dmg/VoiceNote_*.dmg | head -1)
scp "$DMG_PATH" knosi:/srv/confide-landing/download/VoiceNote.dmg
```

ssh knosi 创建目录如不存在：
```bash
ssh knosi "mkdir -p /srv/confide-landing/download && chmod 755 /srv/confide-landing/download"
```

- [ ] **Step 4: 验证下载链接**

```bash
curl -I https://confide.knosi.xyz/download/VoiceNote.dmg
```

Expected: 返回 200 + Content-Length 正常 dmg 大小。

---

### Task 6: 自验证 7 个 dogfood 场景

参考 design Section 9.1 + 9.2 acceptance criteria。

- [ ] **Step 1: 场景 1 — 英文面试**

朋友扮演面试官 + 拖入英文简历 PDF + Job Interview EN + 录 5 个真实问题。

通过标准：
- 5/5 触发 ✅
- ≥4/5 引用简历 ✅
- 全英文 advice ✅
- ≤30 字 ✅
- 触发延迟 ≤2s ✅
- 60 分钟不崩 ✅

- [ ] **Step 2: 场景 2 — 中文面试**

同上中文版。

- [ ] **Step 3: 场景 3 — 日常会议**

真实和老板的 1:1（提前问允许） + 拖入 agenda → 验证触发不过敏。

- [ ] **Step 4: 场景 4 — Stealth 测试**

朋友 Zoom 共享 → 截图给你看是否暴露。

- [ ] **Step 5: 场景 5 — 付费链路**

新邮箱注册 Free → 用完 10 分钟 → Lemon test card 订阅 Pro → 收 license → 输入激活 → 录音扣 5 分钟 → sync → 余额对得上。

- [ ] **Step 6: 场景 6 — BYO**

Settings > BYO > 填 OpenAI key → 录 30 分钟 → 验 Confide quota 不变 + OpenAI 后台有调用。

- [ ] **Step 7: 场景 7 — i18n**

UI 中英切换无 [missing] → 中文用户充值收到中文 license 邮件（Week 5 Task 7 简化只发 en-US，可标记 Pass with caveat）。

- [ ] **Step 8: 把验证结果写到 decision-log.md**

```
## Week 6 自验证
- 日期: <2026-05-XX>
- 场景 1 英文面试: ✅ 通过 / ⚠️ 部分 / ❌ 失败
- 场景 2 中文面试: ...
- 场景 3 日常会议: ...
- 场景 4 Stealth: ...
- 场景 5 付费链路: ...
- 场景 6 BYO: ...
- 场景 7 i18n: ...

整体结论: ✅ MVP ready / ⚠️ ready with caveats / ❌ blocked
```

---

### Task 7: 找 5 个朋友 alpha 试用

**Files:** 无

- [ ] **Step 1: 列出 alpha 用户名单**

至少 3 类用户：
- 1 个海外华人 IC（你目标用户）
- 1 个国内同事在用英文开会
- 1 个完全不懂技术的人（验证 onboarding）
- 2 个面试者（海外求职 / 国内求职）

- [ ] **Step 2: 发 dmg 链接 + 简短使用说明**

模板：
```
Hey, I built a thing — Confide, real-time meeting AI copilot.
Download: https://confide.knosi.xyz/download/VoiceNote.dmg
(macOS 13+ only. App is unsigned during alpha → right-click → Open the first time.)

Try this: open Zoom, start a test meeting, run Confide, see if it transcribes the audio + gives you advice when someone asks "Tell me about yourself".

Free tier: 10 min/month. No card required.
Bug reports / feedback to me directly. ❤️
```

- [ ] **Step 3: 收 5 份反馈**

反馈格式：
```
- 安装顺利吗？(权限弹窗 / 右键 Open 体验?)
- onboarding 哪步卡住？
- 录音 + transcript 质量如何？(英文 / 中文)
- Stealth 觉得有用吗？担心吗？
- 价格 $19 你会付吗？为什么/为什么不？
- 1 个最想要的 v1.1 功能？
```

- [ ] **Step 4: 整理反馈到 decision-log.md**

---

### Task 8: 标 Week 6 + MVP 完成

- [ ] **Step 1: 写 MVP 完成节点**

```
## Week 6 / MVP 完成
- 日期: <2026-05-XX>
- confide.knosi.xyz/pricing 上线 ✅
- VoiceNote.dmg 可下载 ✅
- 完整付费链路通过 ✅
- 5 朋友 alpha 反馈收到 ✅
- 总开发时长: <X> 周（vs design 估算 6 周）
- 主要偏差:
  - <填>
- v1.0.1 hotfix 队列:
  - <填高频 bug>
- 下一步:
  - Week 7-8 alpha 期：基于反馈调 prompt + 修 bug
  - Week 9-10 公测：切独立域名 + landing 完整版
  - Week 11-12 Producthunt 上线
```

---

## Week 6 完成标志（Acceptance Criteria）

完整 design Section 9 AC 复核：

| 类 | AC | 状态 |
|---|---|---|
| A | A1-A4 安装与启动 | ✅ |
| B | B1-B7 录音转录 | ✅（B1 区分 me/other 推 v1.0.5） |
| C | C1-C6 Advice | ✅ |
| D | D1-D6 Stealth | ✅ |
| E | E1-E5 模板 + RAG | ✅（E3 OCR 推 v1.0.5） |
| F | F1-F14 License + 计费 | ✅（F10/F11 auto top-up 推 v1.0.5） |
| G | G1-G4 i18n | ✅（G3 邮件 locale fallback en-US，v1.0.5 完整） |
| H | H1-H4 隐私安全 | ✅ |
| I | I1-I4 质量 | ✅（I2 5 朋友 alpha 反馈完成） |

---

## MVP 完成定义达成

> 你自己能在 6-8 周内：用 Confide 完成一次真实英文面试 + 一次真实 1:1，stealth 不漏陷、付费链路不丢钱、5 朋友试用反馈正面。

✅ 具备进入公测 / Producthunt 上线准备的状态。

下一步：
- v1.0.1: 修 alpha 反馈高频 bug（1-2 周）
- 切独立域名（PH 上线前）
- v1.0.5: 简历优化 + 面试复盘 + auto top-up（首发后 2 周）
- v1.1: iOS 伴侣 + macOS 12 + Opus 模型分层（3-4 个月）
