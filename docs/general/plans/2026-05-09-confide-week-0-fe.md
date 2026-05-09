# Confide Week 0 — Pre-Code Setup

> **For agentic workers:** REQUIRED SUB-SKILL: Use gecc-dev:subagent-driven-development (recommended) or gecc-dev:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在动 code 前并行完成域名 / API keys / 第三方账号 / 工具验证，避免 Week 1+ 因外部依赖卡住。

**Domain:** general

**Architecture:** Week 0 不涉及代码改动，全部为外部账号注册 / DNS 配置 / API 验证。所有工作可在 1-2 天内并行完成（含等待时间）。

**Tech Stack:** Cloudflare DNS、Caddy、Lemon Squeezy、Resend、Anthropic / OpenAI / llmgate、`screencapturekit-rs` PoC

**Spec reference:** `docs/specs/2026-05-09-overseas-meeting-copilot-design.md` Section 8.2

---

### Task 1: 验证 knosi.xyz DNS 是否在 Cloudflare

**Files:** 无（外部配置）

- [ ] **Step 1: 检查 knosi.xyz 名称服务器**

Run:
```bash
dig knosi.xyz NS +short
```

Expected: 输出 `*.cloudflare.com` 表示 DNS 由 Cloudflare 托管（CF Workers 域绑定可零摩擦）。如果不是 Cloudflare（例如 Hetzner DNS / Namecheap），需要在那边加 CNAME 记录指向 Workers，或迁移到 CF。

- [ ] **Step 2: 记录决策**

如果在 CF：直接进 Task 2。
如果不在 CF：决定要不要把 knosi.xyz 整个迁到 CF（影响现有 knosi 主项目，需要谨慎），或保持现 DNS 提供商但加 CNAME 指向 `<worker-name>.workers.dev`。

---

### Task 2: 在 knosi 服务器添加 confide 子域 DNS + Caddy 反代

**Files:**
- Modify: `/etc/caddy/Caddyfile` on knosi server (via ssh knosi)

- [ ] **Step 1: 添加 DNS A 记录**

如果 DNS 在 Cloudflare：去 dash.cloudflare.com 给 `knosi.xyz` 添加 A 记录：
- Name: `confide`
- Value: `195.201.117.172`（knosi server IP，从你 MEMORY 里 knosi_server.md 取）
- Proxy status: DNS only（橙云关掉，否则 WebSocket 会被代理影响）

同时添加：
- Name: `api.confide`
- Value: 同上
- Proxy status: DNS only

如果 DNS 不在 CF：去对应 DNS 提供商加同样的 A 记录。

- [ ] **Step 2: 验证 DNS 解析**

Run:
```bash
dig confide.knosi.xyz +short
dig api.confide.knosi.xyz +short
```

Expected: 都返回 `195.201.117.172`。如未生效等 5 分钟（DNS 传播）。

- [ ] **Step 3: 在 knosi 服务器添加 Caddy 反代配置**

Run:
```bash
ssh knosi "cat >> /etc/caddy/Caddyfile" <<'EOF'

confide.knosi.xyz {
	encode gzip zstd

	header {
		Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
		X-Content-Type-Options "nosniff"
		Referrer-Policy "strict-origin-when-cross-origin"
		X-Frame-Options "SAMEORIGIN"
	}

	# MVP 阶段 confide.knosi.xyz 暂时只放 landing page 占位 + 充值跳转
	# Week 6 部署 landing page 时改成 reverse_proxy 或 file_server
	respond "Confide — coming soon" 200
}

api.confide.knosi.xyz {
	encode gzip zstd

	# Week 4 部署 CF Workers 时改成指向 *.workers.dev
	# MVP 阶段返回 503 表示未上线
	respond "API not deployed yet" 503
}
EOF
```

- [ ] **Step 4: Reload Caddy 让证书自动签发**

Run:
```bash
ssh knosi "caddy reload --config /etc/caddy/Caddyfile"
```

Expected: 无错误输出。Caddy 会自动从 Let's Encrypt 签 confide.knosi.xyz 和 api.confide.knosi.xyz 的 SSL 证书（首次 30 秒-2 分钟）。

- [ ] **Step 5: 验证 HTTPS 工作**

Run:
```bash
curl -I https://confide.knosi.xyz
curl -I https://api.confide.knosi.xyz
```

Expected: 第一个返回 `HTTP/2 200`；第二个返回 `HTTP/2 503`（这是预期，Week 4 改）。**两个 SSL 证书都正常验证（Caddy 已签）**。

---

### Task 3: 写 100 行 PoC 验证 `screencapturekit-rs` 可用性

**Files:**
- Create: `/tmp/sckit-poc/Cargo.toml`
- Create: `/tmp/sckit-poc/src/main.rs`

- [ ] **Step 1: 初始化 PoC 项目**

Run:
```bash
mkdir -p /tmp/sckit-poc/src && cd /tmp/sckit-poc
```

写 `/tmp/sckit-poc/Cargo.toml`:
```toml
[package]
name = "sckit-poc"
version = "0.1.0"
edition = "2021"

[dependencies]
screencapturekit = "0.3"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

注：`screencapturekit-rs` 在 crates.io 实际名字是 `screencapturekit`。如果版本不对，去 https://crates.io/crates/screencapturekit 看最新。

- [ ] **Step 2: 写最小可运行的系统音频捕获 demo**

写 `/tmp/sckit-poc/src/main.rs`:
```rust
use anyhow::Result;
use screencapturekit::shareable_content::SCShareableContent;
use screencapturekit::stream::configuration::SCStreamConfiguration;
use screencapturekit::stream::content_filter::SCContentFilter;
use screencapturekit::stream::output_trait::SCStreamOutputTrait;
use screencapturekit::stream::output_type::SCStreamOutputType;
use screencapturekit::stream::SCStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct AudioCapture {
    sample_count: Arc<Mutex<usize>>,
}

impl SCStreamOutputTrait for AudioCapture {
    fn did_output_sample_buffer(
        &self,
        _sample_buffer: screencapturekit::output::CMSampleBuffer,
        of_type: SCStreamOutputType,
    ) {
        if of_type == SCStreamOutputType::Audio {
            let mut count = self.sample_count.lock().unwrap();
            *count += 1;
            if *count % 10 == 0 {
                eprintln!("[poc] audio samples received: {}", *count);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let content = SCShareableContent::get().map_err(|e| anyhow::anyhow!("get content: {:?}", e))?;
    let displays = content.displays();
    let display = displays.first().ok_or_else(|| anyhow::anyhow!("no display"))?;
    eprintln!("[poc] using display: {:?}", display.display_id());

    let filter = SCContentFilter::new().with_display_excluding_windows(display, &[]);
    let config = SCStreamConfiguration::new()
        .set_captures_audio(true)?
        .set_excludes_current_process_audio(true)?;

    let sample_count = Arc::new(Mutex::new(0_usize));
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(
        AudioCapture { sample_count: sample_count.clone() },
        SCStreamOutputType::Audio,
    );

    stream.start_capture().map_err(|e| anyhow::anyhow!("start: {:?}", e))?;
    eprintln!("[poc] capturing for 5 seconds, play any audio (Music / YouTube / etc)...");
    tokio::time::sleep(Duration::from_secs(5)).await;
    stream.stop_capture().map_err(|e| anyhow::anyhow!("stop: {:?}", e))?;

    let final_count = *sample_count.lock().unwrap();
    eprintln!("[poc] DONE. Total audio sample buffers: {}", final_count);
    if final_count == 0 {
        eprintln!("[poc] ⚠️  ZERO samples — possible: no audio playing, or permission denied");
        std::process::exit(1);
    }
    Ok(())
}
```

注：这份代码是**起步参考**——`screencapturekit` crate 的真实 API 可能略有差异，跑不通时去 https://docs.rs/screencapturekit 看当前版本的 trait 签名。

- [ ] **Step 3: 跑 PoC，开 Spotify / YouTube 后启动**

Run:
```bash
cd /tmp/sckit-poc
cargo build 2>&1 | tail -5
# 第一次编译需要 1-2 分钟
```

打开 macOS 系统设置 > 隐私与安全 > 屏幕录制 → 准备好授权（首次启动会弹）。

打开 Music / YouTube / Spotify，开始播放任何音频。

Run:
```bash
cargo run 2>&1
```

第一次跑会弹"Terminal 想录制屏幕" → 同意 → 重启 Terminal → 再 `cargo run`。

Expected: 输出大致为：
```
[poc] using display: 1
[poc] capturing for 5 seconds, play any audio...
[poc] audio samples received: 10
[poc] audio samples received: 20
...
[poc] DONE. Total audio sample buffers: 240
```

`240` 这个数字代表 5 秒收到了 240 个 sample buffer（约 48 buffers/second，48kHz @ 1024 samples/buffer 标准 macOS 速率）。**只要 > 0 就证明可用**。

- [ ] **Step 4: 评估结果，决定 Week 1 用哪个 crate**

如果 PoC 跑通：Week 1 直接用 `screencapturekit` crate。

如果跑不通（编译错 / 运行 panic / 0 samples）：
- 看 https://crates.io/crates/screencapturekit "Recently updated" 时间——超过 1 年没更新就放弃
- 后备：直接用 `objc2-screen-capture-kit` + `objc2` 写底层 binding（多 1-2 天工程量）

写决策到 `docs/general/plans/decision-log.md`：
```
2026-05-XX: screencapturekit-rs PoC verdict
- crate version tested: X.Y.Z
- result: pass / fail
- decision for Week 1: use screencapturekit / fallback to objc2 binding
```

---

### Task 4: 注册 Apple Developer 推迟到 Week 5（不是 Week 0）

**已按 design 决策**：Apple Dev 推到 Week 5 申请，Week 0 **不做**任何 Apple 相关事。

- [ ] **Step 1: 确认这个决策仍然合理**

回顾 design Section 8.2 + Section 10.1 R5。如果你重新决定 Week 0 申请，参考 design Section 8.2 修订路线图。

---

### Task 5: 注册 Anthropic API account（Week 5 切产用，但 Week 0 占座）

**Files:** 无（外部账号）

- [ ] **Step 1: 注册 Anthropic Console**

打开 https://console.anthropic.com/ → 注册账号 → 验证邮箱。

- [ ] **Step 2: 充 $5 试用额度**

Console > Plans & Billing → 充值 $5（送 $5 试用，账户里显示 $10）。**Week 5 才会真用，Week 1-4 alpha 走 llmgate 不动这个**。

- [ ] **Step 3: 创建一个 API key 标记为 "confide-alpha-test"**

Console > API Keys → Create key → name: `confide-alpha-test` → 复制 key。

- [ ] **Step 4: 验证 key 可用**

Run:
```bash
curl https://api.anthropic.com/v1/messages \
  -H "x-api-key: <key>" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{
    "model": "claude-sonnet-4-6",
    "max_tokens": 50,
    "messages": [{"role": "user", "content": "Say hello in 5 words."}]
  }'
```

Expected: 返回 `{"id":"msg_...","type":"message","content":[{"type":"text","text":"Hello! Nice to meet you."}],...}`

如果 401：key 不对。如果 400 model not found：换 `claude-3-5-sonnet-20241022` 或最新模型名（Anthropic 模型名 update 频繁，参考 console docs）。

- [ ] **Step 5: 把 key 写到 1Password / Bitwarden / 系统钥匙串**

**不要** commit 到 git、**不要** 写在任何 .env.example、**不要** 截图发任何地方。

---

### Task 6: 注册 OpenAI API + 验证 GPT-Realtime-Whisper 可用

**Files:** 无（外部账号）

- [ ] **Step 1: 注册 OpenAI Platform**

https://platform.openai.com/ → 注册 → 完成手机号验证 → 充 $10 入门额度。

- [ ] **Step 2: 创建 API key 标记 "confide-week1-asr"**

Settings > API keys → Create new secret key → name: `confide-week1-asr`。

- [ ] **Step 3: 验证 GPT-Realtime-Whisper 实际可用**

Run:
```bash
curl https://api.openai.com/v1/realtime/transcription_sessions \
  -H "Authorization: Bearer <key>" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-realtime-whisper"
  }'
```

Expected: 返回 `{"id":"...","object":"realtime.transcription_session","client_secret":{...},...}`。

如果 404 model not found：去 https://platform.openai.com/docs/models 确认 `gpt-realtime-whisper` 当前正确的 model ID（2026-05-07 发布时叫这个名，但 OpenAI 改名常见）。

如果 403：账号未启用 Realtime API access；可能要 join waitlist 或开通付费。

- [ ] **Step 4: 写 model ID 到决策日志**

记录确认的 model ID 字符串（`gpt-realtime-whisper` 或别的），Week 1 ASR provider 实现要用。

---

### Task 7: 注册 llmgate API access（已有则跳过）

**Files:** 无

- [ ] **Step 1: 确认 llmgate 中 Sonnet 4.6 模型 ID**

打开你字节内部 llmgate 控制台 → 找到 Anthropic Sonnet 4.6 对应的 model ID 字符串（可能叫 `claude-sonnet-4-6` 或 `bytedance-anthropic-sonnet-4.6` 或其他）。

- [ ] **Step 2: 验证 alpha 期 LLM 可用**

Run:
```bash
curl https://llmgate.io/v1/chat/completions \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "<llmgate sonnet model id>",
    "messages": [{"role": "user", "content": "ping"}],
    "max_tokens": 10
  }'
```

Expected: 返回 OpenAI-compatible chat completion JSON。如果失败，去字节内部 llmgate 文档确认 endpoint URL 和鉴权方式。

- [ ] **Step 3: 记录 llmgate config 到决策日志**

```
llmgate base_url: https://llmgate.io/v1
llmgate model_id: <verified id>
llmgate auth: Bearer <token in 1Password>
alpha 期使用：Week 1-4，Week 5 切 Anthropic 直连
```

---

### Task 8: 注册 Lemon Squeezy account + 创建 Confide store

**Files:** 无（外部 SaaS 配置）

- [ ] **Step 1: 注册 Lemon Squeezy**

https://app.lemonsqueezy.com/register → 注册 → 验证邮箱。

- [ ] **Step 2: 创建 Confide store**

Dashboard > Stores > New Store → name: `Confide` → currency: USD → tax mode: 让 Lemon 处理（默认）→ Statement descriptor: **VOICENOTE APP**（重要：要和 Section 5.2 stealth 进程伪装名一致）。

- [ ] **Step 3: 创建 3 个 subscription products**

每个 product 创建 1 个 monthly variant：

Product 1：
- Name: `Confide Pro`
- Price: $19.00 USD
- Billing: Subscription, monthly
- Variant ID 记下来 → 写到决策日志（design Section 6.7 PLAN_CATALOG.pro.lemonVariantId）

Product 2：
- Name: `Confide Ultra`
- Price: $49.00 USD
- Billing: Subscription, monthly
- Variant ID 记下来

Product 3（暂时不做，Free tier 在自己后端管，不在 Lemon 注册）

注：Free tier 用户在自己 KV 里管，不进 Lemon Squeezy。Lemon 只管 Pro / Ultra 订阅。

- [ ] **Step 4: 创建 Webhook signing secret**

Settings > Webhooks → 暂时填 placeholder URL `https://api.confide.knosi.xyz/lemonsqueezy-webhook`（Week 4 部署后改）→ 选事件：`subscription_created`, `subscription_payment_success`, `subscription_cancelled`, `subscription_expired` → 创建 → **复制 signing secret 到 1Password**。

- [ ] **Step 5: 验证 hosted checkout 工作**

去 Confide Pro product 页 → Share > Copy checkout link → 在浏览器打开 → 确认显示价格 $19/月、Pay With Card 按钮可点。**别真付（test mode 在 Settings 切，但 MVP 阶段直接生产环境也可，先不付）**。

---

### Task 9: 注册 Resend account + 验证 from 邮箱

**Files:** 无

- [ ] **Step 1: 注册 Resend**

https://resend.com/signup → 注册 → 验证邮箱。免费层 100 邮件/天，MVP 阶段够用。

- [ ] **Step 2: 添加 confide.knosi.xyz domain**

Dashboard > Domains > Add Domain → `confide.knosi.xyz` → Resend 给 4 条 DNS 记录（DKIM / SPF / DMARC / MX 或 return-path）。

- [ ] **Step 3: 把这些 DNS 记录加到 knosi.xyz 的 DNS**

如果 DNS 在 Cloudflare：dash.cloudflare.com > knosi.xyz DNS records → 全部添加。如果在别的 DNS provider：同步骤。

等 5-10 分钟传播。

- [ ] **Step 4: 在 Resend 触发 verification**

Dashboard > Domains > confide.knosi.xyz > Verify → 等待 4 条记录全部 ✅。

- [ ] **Step 5: 创建 API key 标记 "confide-week5"**

Dashboard > API Keys → Create → name: `confide-week5` → Full access → 复制到 1Password。

- [ ] **Step 6: 测试发一封邮件**

Run:
```bash
curl -X POST 'https://api.resend.com/emails' \
  -H 'Authorization: Bearer <resend key>' \
  -H 'Content-Type: application/json' \
  -d '{
    "from": "hello@confide.knosi.xyz",
    "to": ["zhousiyao03@gmail.com"],
    "subject": "Confide Week 0 — Resend Test",
    "html": "<p>Resend domain verified ✅</p>"
  }'
```

Expected: 返回 `{"id":"..."}`。1 分钟内 zhousiyao03@gmail.com 收到邮件。**如果进 spam**：检查 DKIM/DMARC 是否全 verified；如果仍进 spam，发件域信誉问题，发版前再调。

---

### Task 10: 创建 Cloudflare account + Workers 项目占位

**Files:** 无

- [ ] **Step 1: 注册 / 登录 Cloudflare**

如果已有 dash.cloudflare.com 账号（你有 knosi.xyz 在 CF 的话已经有），跳过注册。

- [ ] **Step 2: 安装 Wrangler CLI**

Run:
```bash
which wrangler || npm install -g wrangler
wrangler --version
```

Expected: 输出版本号 ≥ 4.0。如果未登录：
```bash
wrangler login
```

浏览器自动开 → 同意 → 回到 terminal 看到 "Successfully logged in"。

- [ ] **Step 3: 占位创建 KV namespace**

Run:
```bash
wrangler kv namespace create CONFIDE_LICENSES
wrangler kv namespace create CONFIDE_LICENSES --preview
```

Expected: 输出两个 namespace ID（一个 production、一个 preview）。**记录到决策日志**——Week 4 写 wrangler.toml 时用。

- [ ] **Step 4: 占位创建 D1 database**

Run:
```bash
wrangler d1 create confide-events
```

Expected: 输出 database id。**记录到决策日志**。

- [ ] **Step 5: 验证 CF account 可创建 Workers**

Run:
```bash
wrangler whoami
```

Expected: 输出 `Account ID: <id>` 和 email。**记录 Account ID 到决策日志**。

---

### Task 11: 决定 Confide 最终产品名（占座，可暂用 codename）

**Files:** 无

- [ ] **Step 1: 列出 5 个候选名 + 检查域名 / 商标**

写到决策日志：
```
候选名（最终产品名待定）：
1. Confide
2. <候选 2>
3. <候选 3>
4. <候选 4>
5. <候选 5>

每个候选检查：
- .com / .app / .io 域名是否被占
- USPTO trademark 搜索
- App Store 名称冲突（不上 App Store 但避免品牌混淆）
```

可用工具：https://domains.google / https://porkbun.com / https://www.namecheckr.com (社媒检查)。

- [ ] **Step 2: 决定 MVP 阶段对外名 + 上线时切名策略**

如果 Confide.com 不可用、Confide.app 可用：MVP 用 confide.app。
如果都被占：选第 2 候选，给 design 附录 C 改 codename。

写决策到日志，**MVP 期间所有 doc / UI 文案保持 codename "Confide"，Producthunt 上线前一周一次性切真名**。

---

### Task 12: 创建决策日志文件

**Files:**
- Create: `docs/general/plans/decision-log.md`

- [ ] **Step 1: 创建文件并初始化结构**

写 `docs/general/plans/decision-log.md`:
```markdown
# Confide Decision Log

> 所有 Week 0+ 的关键决策、外部 ID、API key 引用都记录在这里。**不写 secret 本身**，只写 secret 在 1Password 里的引用名。

## Week 0 验证结果

### DNS / 域名
- knosi.xyz DNS provider: <Cloudflare / 其他>
- confide.knosi.xyz A record: 195.201.117.172 (knosi server)
- api.confide.knosi.xyz A record: 195.201.117.172
- SSL 证书状态: ✅ 已签 / ⏳ 待签
- 最终产品名: <pending Week 0 Task 11>

### screencapturekit-rs PoC
- crate name: screencapturekit
- version tested: <X.Y.Z>
- PoC result: ✅ pass / ❌ fail
- decision for Week 1: <use screencapturekit | fallback to objc2 binding>

### API keys（1Password 引用名）
- Anthropic: 1P/confide-alpha-test
- OpenAI: 1P/confide-week1-asr
- llmgate: 1P/confide-llmgate
- Resend: 1P/confide-week5
- Lemon Squeezy webhook secret: 1P/confide-lemon-webhook

### Model IDs
- Anthropic Sonnet: <verified ID, e.g. claude-sonnet-4-6>
- OpenAI GPT-Realtime-Whisper: <verified ID, e.g. gpt-realtime-whisper>
- llmgate Sonnet: <verified ID>

### Lemon Squeezy
- Store name: Confide
- Statement descriptor: VOICENOTE APP
- Pro variant ID: <verified ID>
- Ultra variant ID: <verified ID>

### Cloudflare
- Account ID: <verified>
- KV namespace CONFIDE_LICENSES production: <id>
- KV namespace CONFIDE_LICENSES preview: <id>
- D1 database confide-events: <id>
```

- [ ] **Step 2: 在 Week 0 完成的每个 task 后回填这个文件**

Task 2/3/5/6/7/8/9/10/11 完成时把对应字段填上。Week 1+ 都会引用这个文件。

---

### Task 13: 完成 Week 0 sanity check

- [ ] **Step 1: 走一遍清单确认所有 task 已完成**

```
[ ] Task 1-2: DNS + Caddy 反代（confide.knosi.xyz / api.confide.knosi.xyz HTTPS 工作）
[ ] Task 3: screencapturekit PoC 跑通
[ ] Task 5: Anthropic API key 验证
[ ] Task 6: OpenAI GPT-Realtime-Whisper 可用确认
[ ] Task 7: llmgate Sonnet 可用确认
[ ] Task 8: Lemon Squeezy 3 SKU 创建
[ ] Task 9: Resend 域名验证 + 测试邮件收到
[ ] Task 10: CF account / KV / D1 占位
[ ] Task 11: 产品名候选决定
[ ] Task 12: decision-log.md 已写
```

如果有 ❌：解决再进 Week 1。**screencapturekit PoC 失败的话不要进 Week 1 主代码——先解决 ASR 选型**。

- [ ] **Step 2: 估算 Week 0 实际花费**

| 项 | 时间 | 钱 |
|---|---|---|
| DNS + Caddy | 30 分钟 | $0 |
| screencapturekit PoC | 1-2 小时 | $0 |
| API 账号注册 + 充值 | 1 小时 | $5 (Anthropic) + $10 (OpenAI) |
| Lemon Squeezy 配置 | 30 分钟 | $0 |
| Resend 域名 | 20 分钟（含 DNS 传播） | $0 |
| CF wrangler | 20 分钟 | $0 |

**总：~3 小时 active work + 等待时间 + $15 API 充值**。可以单天搞定。

---

## Week 0 完成标志

- ✅ 所有外部账号注册完毕、key 在 1Password
- ✅ confide.knosi.xyz HTTPS 可访问
- ✅ screencapturekit PoC 可跑（捕获到系统音频 sample buffer）
- ✅ Lemon Squeezy 3 SKU 可见、checkout link 工作
- ✅ Resend 测试邮件成功收到
- ✅ decision-log.md 所有字段已填
- ✅ Anthropic / OpenAI / llmgate 三个 LLM/ASR provider 都验证可调用

下一步：进 Week 1 — 音频管线 + GPT-Realtime-Whisper 集成。
