# Confide Decision Log

> Week 0+ 的关键决策、外部 ID、API key 引用都在这里。**不写 secret 本身**，只写在 1Password 里的引用名。

## DNS / 域名

- knosi.xyz DNS provider: **Cloudflare**（已确认 NS = john.ns.cloudflare.com / molly.ns.cloudflare.com）
- knosi server IP（IPv4）: 195.201.117.172
- knosi server IP（IPv6）: 2a01:4f8:1c0c:7d81::1
- confide.knosi.xyz A record: ⏳ 待加（Cloudflare DNS only，不开橙云）
- api.confide.knosi.xyz A record: ⏳ 待加
- SSL 证书状态: ⏳ Caddy 自动签
- 最终产品名: ⏳ 候选 codename "Confide"

## screencapturekit PoC

- crate name: `screencapturekit`
- version tested: **0.3.6**（编译通过 ✅，2026-05-09）
- PoC result: ✅ pass（编译 + API 匹配；运行验证需要授予 Screen Recording 权限给 Terminal，由用户完成）
- decision for Week 1: 用 `screencapturekit = "0.3.6"`，extract_pcm 用 Pattern A（CMSampleBuffer.get_audio_buffer_list）
- PoC 路径: `/tmp/sckit-poc/`

## API keys（1Password 引用名 — 实际 key 不写在这里）

- Anthropic: `1P/confide-alpha-test`
- OpenAI: `1P/confide-week1-asr`
- llmgate: `1P/confide-llmgate`
- Resend: `1P/confide-week5`
- Lemon Squeezy webhook secret: `1P/confide-lemon-webhook`
- Lemon Squeezy API: `1P/confide-lemon-api`

## Model IDs

- Anthropic Sonnet 直连: ⏳ 待确认（候选 `claude-sonnet-4-6` / `claude-3-5-sonnet-20241022`）
- OpenAI GPT-Realtime-Whisper: ⏳ 待确认（候选 `gpt-realtime-whisper`）
- llmgate Sonnet: ⏳

## Lemon Squeezy

- Store name: Confide
- Statement descriptor: **VOICENOTE APP**（与 stealth 进程伪装名一致）
- Pro variant ID: ⏳
- Ultra variant ID: ⏳
- Pro checkout link: ⏳
- Ultra checkout link: ⏳

## Cloudflare

- Account ID: ⏳
- KV namespace `CONFIDE_LICENSES` (production): ⏳
- KV namespace `CONFIDE_LICENSES` (preview): ⏳
- D1 database `confide-events`: ⏳

## Apple Developer

- 申请日期: ⏳ Week 5 Day 21 提交
- 状态: ⏳ pending
- Team ID: ⏳
- Apple ID（用于 notarytool）: ⏳
- App-specific password（appleid.apple.com 创建）: `1P/confide-apple-app-password`

## Resend

- Domain `confide.knosi.xyz`:
  - DKIM: ⏳ 待加 DNS 记录
  - SPF: ⏳
  - DMARC: ⏳
  - Verified: ⏳
- API key: `1P/confide-week5`

## 完成节点

### Week 0
- 日期: ⏳
- 13 个 task 状态: ⏳

### Week 1
- 日期: ⏳
- 验收: ⏳

### Week 2
- 日期: ⏳

### Week 3
- 日期: ⏳

### Week 4
- 日期: ⏳

### Week 5
- 日期: ⏳

### Week 6 / MVP 完成
- 日期: ⏳
- 整体结论: ⏳ MVP ready / ready with caveats / blocked

## v1.0.5 / v1.1 待办池（实施过程中陆续追加）

- Resend locale 跟随 license 创建时的 user locale（MVP 只发 en-US）
- pending_usage 离线队列持久化到磁盘
- Auto top-up 实际向 Lemon Squeezy 发起 charge
- 多设备 license lease 5 分钟
- 透明度调节真改 opacity（MVP 仅 log）
- 菜单栏 "Stealth Mode: ON/OFF" label 动态更新
- Speaker diarization（mic / system 双 ASR session）
- PDF OCR fallback（tesseract-rs）
- 模板编辑器 UI（用户改 system_prompt）
- macOS 12 BlackHole fallback
- iOS 伴侣 app
- 用户可改 app 白名单
- 用户可自定义快捷键
- Opus 4.7 模型分层 + Advanced Settings
- 行业题库（Ultra 专属）
- 简历优化 + 面试复盘报告（Pro/Ultra credits）
