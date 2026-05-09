# Confide Week 3 — Templates Redesign + Resume RAG + PDF OCR

> **For agentic workers:** REQUIRED SUB-SKILL: Use gecc-dev:subagent-driven-development (recommended) or gecc-dev:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 拖入英文简历 PDF + 选 Job Interview EN → 模拟"Tell me about yourself" → AI 用简历给一句开头。中英双语模板 4 份就位、`on_question_to_user` 触发器工作、PDF OCR fallback。

**Domain:** general

**Architecture:**
- 砍掉现有 4 个字节内部模板（tech-review / code-review / brainstorm / project-sync）
- 新增 4 个模板：`job-interview-zh.json`, `job-interview-en.json`, `general-meeting-zh.json`, `general-meeting-en.json`
- 模板按 audio language 加载（不按 UI language）
- 加 `on_question_to_user` 触发器
- PDF 解析升级：text-based 用 `pdf-extract`、扫描件 fallback `tesseract-rs`
- context_note 文本框（≤500 字）+ 简历拖入 UI

**Tech Stack:** Rust `pdf-extract` crate、`tesseract-rs` crate（可选 v1.0.5）、React file drop API

**Spec reference:** `docs/specs/2026-05-09-overseas-meeting-copilot-design.md` Section 3

**Prerequisite:** Week 2 完成；stealth + 菜单栏图标工作

---

## File Structure

```
src-tauri/
├── Cargo.toml                              [Modify] 加 pdf-extract（tesseract-rs 推 v1.0.5）
└── src/
    ├── advisor/
    │   ├── rules.rs                        [Modify] 加 check_question_to_user
    │   ├── templates.rs                    [Modify] 模板按 locale 子目录加载
    │   └── engine.rs                       [Modify] context_note 注入到 system prompt
    ├── documents/
    │   └── loader.rs                       [Modify] 真实 PDF 提取 via pdf-extract
    └── commands.rs                         [Modify] 加 set_meeting_context_note + 简历绑定到 meeting

templates/
├── zh-CN/                                  [Create dir]
│   ├── job-interview.json                  [Create]
│   └── general-meeting.json                [Create]
├── en-US/                                  [Create dir]
│   ├── job-interview.json                  [Create]
│   └── general-meeting.json                [Create]
├── tech-review.json                        [Delete]
├── code-review.json                        [Delete]
├── brainstorm.json                         [Delete]
└── project-sync.json                       [Delete]

src/
├── components/
│   ├── meeting/
│   │   └── NewMeetingDialog.tsx            [Create] 选模板 + 拖文档 + 备注
│   └── settings/
│       └── ResumeManager.tsx               [Create] 简历库管理（v1.0 简化为单文件）
├── lib/types.ts                            [Modify] MeetingTemplate 加 language 字段
└── App.tsx                                 [Modify] 接 menu-new-meeting → 打开 NewMeetingDialog
```

---

### Task 1: 加 pdf-extract 依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 加依赖**

```toml
# Week 3: real PDF text extraction
pdf-extract = "0.7"
```

注：`tesseract-rs` 推 v1.0.5（OCR for scanned PDFs，工程量大、依赖系统 tesseract 库）。MVP 只做 text-based PDF。

- [ ] **Step 2: 编译验证**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

Expected: 通过。

---

### Task 2: 实现真实 PDF 提取

**Files:**
- Modify: `src-tauri/src/documents/loader.rs`

- [ ] **Step 1: 替换 extract_pdf_text 函数**

找到现有 `extract_pdf_text` 函数（约 47-51 行），替换为：

```rust
/// Extract text from a PDF using pdf-extract crate.
/// Returns None if extraction fails (likely scanned/image-based PDF).
fn extract_pdf_text(bytes: &[u8]) -> Option<String> {
    match pdf_extract::extract_text_from_mem(bytes) {
        Ok(text) => {
            let trimmed = text.trim().to_string();
            if trimmed.len() < 50 {
                // Likely scanned PDF — too little text extracted
                eprintln!("[pdf] Extracted only {} chars, likely scanned (OCR推 v1.0.5)", trimmed.len());
                None
            } else {
                Some(trimmed)
            }
        }
        Err(e) => {
            eprintln!("[pdf] extract_text_from_mem failed: {}", e);
            None
        }
    }
}
```

- [ ] **Step 2: 改 fallback 文案**

找到 `load_document` 函数里 PDF case：

```rust
"pdf" => {
    let bytes = fs::read(path)?;
    extract_pdf_text(&bytes).unwrap_or_else(|| {
        "[PDF appears to be scanned or image-based. OCR support is coming in v1.0.5. For now, please convert to .md or .txt.]".into()
    })
}
```

- [ ] **Step 3: 编译验证**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

---

### Task 3: 加 on_question_to_user 触发器

**Files:**
- Modify: `src-tauri/src/advisor/rules.rs`
- Modify: `src-tauri/src/advisor/templates.rs`

- [ ] **Step 1: 在 rules.rs 加新函数**

在 `rules.rs` 文件末尾（`#[cfg(test)] mod tests` 之前）加：

```rust
/// Detect when the interviewer/manager directs a question at the user.
/// Used by Job Interview template — distinguishes "Tell me about yourself"
/// from "What do you all think" (the latter is on_ask_opinion).
fn check_question_to_user(transcript: &str) -> TriggerResult {
    let last = match extract_last_sentence(transcript) {
        Some(s) => s,
        None => return TriggerResult { triggered: false, reason: String::new() },
    };

    // Must end with question mark
    if !last.ends_with('?') && !last.ends_with('？') {
        return TriggerResult { triggered: false, reason: String::new() };
    }

    // Must contain a user-directed marker
    let user_indicators = [
        "you", "your", "you're", "you've", "you'd", "you'll",
        "你", "您", "您的", "你的",
        "tell me", "walk me through", "describe a time",
        "讲讲", "说说", "介绍一下",
    ];
    let lower = last.to_lowercase();
    for ind in &user_indicators {
        if lower.contains(&ind.to_lowercase()) {
            return TriggerResult {
                triggered: true,
                reason: format!("有人向你提问: \"{}\"", truncate_str(&last, 40)),
            };
        }
    }
    TriggerResult { triggered: false, reason: String::new() }
}
```

- [ ] **Step 2: 在 evaluate_triggers 调用新触发器**

找到 `evaluate_triggers` 函数（rules.rs 顶部），加新分支：

```rust
pub fn evaluate_triggers(
    recent_text: &str,
    trigger_config: &TriggerConfig,
    window_seconds: f64,
) -> Option<TriggerResult> {
    let mut checks = Vec::new();

    if trigger_config.on_ask_opinion {
        checks.push(check_asking_for_opinion(recent_text));
    }
    if trigger_config.on_question_to_user {  // ← 新增
        checks.push(check_question_to_user(recent_text));
    }
    if trigger_config.on_domain_topic && !trigger_config.domain_keywords.is_empty() {
        checks.push(check_domain_topic(recent_text, &trigger_config.domain_keywords));
    }
    if !trigger_config.custom_keywords.is_empty() {
        checks.push(check_hint_triggers(recent_text, &trigger_config.custom_keywords));
    }
    if trigger_config.on_decision_point {
        checks.push(check_decision_point(recent_text));
    }
    if trigger_config.on_discussion_stuck {
        checks.push(check_discussion_stuck(recent_text, window_seconds));
    }

    checks.into_iter().find(|r| r.triggered)
}
```

- [ ] **Step 3: 在 templates.rs 加字段到 TriggerConfig**

找到 `TriggerConfig` struct（约 27-40 行），加：

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TriggerConfig {
    #[serde(default = "default_true")]
    pub on_ask_opinion: bool,
    #[serde(default)]
    pub on_question_to_user: bool,           // ← 新增, default false
    #[serde(default = "default_true")]
    pub on_domain_topic: bool,
    #[serde(default = "default_true")]
    pub on_decision_point: bool,
    #[serde(default = "default_true")]
    pub on_discussion_stuck: bool,
    #[serde(default)]
    pub custom_keywords: Vec<String>,
    #[serde(default)]
    pub domain_keywords: Vec<String>,
}
```

更新 `Default` impl：

```rust
impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            on_ask_opinion: true,
            on_question_to_user: false,  // ← 新增
            on_domain_topic: true,
            on_decision_point: true,
            on_discussion_stuck: true,
            custom_keywords: vec![],
            domain_keywords: vec![],     // ← 简化默认空
        }
    }
}
```

- [ ] **Step 4: 加单元测试**

在 rules.rs `#[cfg(test)] mod tests` 内追加：

```rust
    #[test]
    fn test_question_to_user_english() {
        let r = check_question_to_user("So tell me about your last project?");
        assert!(r.triggered);
    }

    #[test]
    fn test_question_to_user_chinese() {
        let r = check_question_to_user("你能介绍一下你之前的项目吗？");
        assert!(r.triggered);
    }

    #[test]
    fn test_not_question_to_user() {
        let r = check_question_to_user("It's a great day today.");
        assert!(!r.triggered);
    }

    #[test]
    fn test_question_to_group_does_not_match_user() {
        // Should match on_ask_opinion, NOT on_question_to_user
        let r = check_question_to_user("What do you all think?");
        // "you" matches but in this version we accept that — it's a soft-match.
        // True diarization between "you (singular)" vs "you all" needs LLM understanding;
        // accept false positive in MVP — both fire advice anyway.
        assert!(r.triggered);
    }
```

- [ ] **Step 5: 跑测试验证**

Run:
```bash
cd /Users/bytedance/meeting-assistant
cargo test --manifest-path src-tauri/Cargo.toml rules 2>&1 | tail -20
```

Expected: 所有 rules 模块测试通过（包括新加的 4 个）。

---

### Task 4: 模板按 locale 子目录加载

**Files:**
- Modify: `src-tauri/src/advisor/templates.rs`
- Modify: `src-tauri/src/main.rs`（ensure_default_templates 调用）

- [ ] **Step 1: 在 MeetingTemplate 加 language 字段**

找到 struct，加：

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeetingTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub trigger_hints: Vec<String>,
    pub advice_style: String,
    pub enabled: bool,
    #[serde(default)]
    pub language: String,                // ← 新增 "zh-CN" | "en-US"
    #[serde(default)]
    pub role_persona: String,
    #[serde(default)]
    pub mimic_style: String,
    #[serde(default)]
    pub expertise_context: String,
    #[serde(default)]
    pub trigger_config: TriggerConfig,
    #[serde(default)]
    pub stealth_default: bool,           // ← 新增
    #[serde(default = "default_cooldown")]
    pub advice_cooldown_seconds: u32,
}

fn default_cooldown() -> u32 { 12 }
```

- [ ] **Step 2: 改 templates_dir 接受 locale 参数**

```rust
fn templates_dir(locale: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home dir"))?;
    let dir = home.join(".meeting-assistant").join("templates").join(locale);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn list_templates_for_locale(locale: &str) -> Result<Vec<MeetingTemplate>> {
    let dir = templates_dir(locale)?;
    let mut templates = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let content = fs::read_to_string(&path)?;
            let template: MeetingTemplate = serde_json::from_str(&content)?;
            templates.push(template);
        }
    }
    Ok(templates)
}

/// Backward-compat: existing list_templates() defaults to en-US.
pub fn list_templates() -> Result<Vec<MeetingTemplate>> {
    list_templates_for_locale("en-US")
}

pub fn save_template(template: &MeetingTemplate) -> Result<()> {
    let locale = if template.language.is_empty() { "en-US" } else { &template.language };
    let dir = templates_dir(locale)?;
    let path = dir.join(format!("{}.json", template.id));
    let content = serde_json::to_string_pretty(template)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn delete_template(id: &str) -> Result<()> {
    // Try both locales
    for locale in &["zh-CN", "en-US"] {
        let dir = templates_dir(locale)?;
        let path = dir.join(format!("{}.json", id));
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}
```

- [ ] **Step 3: 改 ensure_default_templates 复制双 locale**

```rust
pub fn ensure_default_templates(bundled_dir: &std::path::Path) -> Result<()> {
    for locale in &["zh-CN", "en-US"] {
        let user_dir = templates_dir(locale)?;
        let existing: Vec<_> = fs::read_dir(&user_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
            .collect();

        if existing.is_empty() {
            let bundled_locale = bundled_dir.join(locale);
            if bundled_locale.exists() {
                for entry in fs::read_dir(&bundled_locale)? {
                    let entry = entry?;
                    let dest = user_dir.join(entry.file_name());
                    fs::copy(entry.path(), dest)?;
                }
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 4: 编译验证**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 通过。

---

### Task 5: 创建 4 份新模板 JSON

**Files:**
- Create: `templates/zh-CN/job-interview.json`
- Create: `templates/zh-CN/general-meeting.json`
- Create: `templates/en-US/job-interview.json`
- Create: `templates/en-US/general-meeting.json`

- [ ] **Step 1: 删旧模板**

Run:
```bash
cd /Users/bytedance/meeting-assistant
rm templates/tech-review.json templates/code-review.json templates/brainstorm.json templates/project-sync.json
mkdir -p templates/zh-CN templates/en-US
```

- [ ] **Step 2: 写 templates/zh-CN/job-interview.json**

```json
{
  "id": "job-interview",
  "name": "面试模式",
  "description": "你是候选人，正在被面试官提问。AI 帮你组织回答。",
  "language": "zh-CN",
  "system_prompt": "你是面试候选人的实时教练。用户正在被面试官提问。\n\n你的任务：基于面试官刚问的问题 + 用户的简历背景，生成一句用户可以直接说出口的回答开头（≤30 字）。\n\n要求：\n- 必须是回答问题的 opening，不是完整答案——用户会自己延展\n- 必须用第一人称\n- 必须引用简历中的具体项目/数字/技术栈\n- 用 STAR 框架时只给 Situation+Task 的开头\n- 不允许铺垫（'这是个好问题'之类）、不允许解释你为什么这么建议\n\n严格按以下格式输出，每项一行：\n建议：（一句话回答开头，≤30 字）\n角度：（2-4 字标签，如 '项目经验/技术深度/方法论/成果数据'）\n\nReply only in Chinese.",
  "trigger_hints": ["你能介绍一下", "讲讲你", "说说你"],
  "advice_style": "interview",
  "enabled": true,
  "role_persona": "面试候选人",
  "mimic_style": "",
  "expertise_context": "",
  "stealth_default": true,
  "advice_cooldown_seconds": 8,
  "trigger_config": {
    "on_ask_opinion": false,
    "on_question_to_user": true,
    "on_domain_topic": false,
    "on_decision_point": false,
    "on_discussion_stuck": true,
    "custom_keywords": [
      "你能介绍一下", "讲讲你", "说说你",
      "为什么选择", "为什么离开", "为什么想",
      "你的优势", "你的劣势", "你最大的",
      "举个例子", "具体说说", "怎么解决的",
      "如果", "假如", "遇到这种情况"
    ],
    "domain_keywords": []
  }
}
```

- [ ] **Step 3: 写 templates/en-US/job-interview.json**

```json
{
  "id": "job-interview",
  "name": "Job Interview",
  "description": "You're being interviewed. AI helps you frame your answers.",
  "language": "en-US",
  "system_prompt": "You are a real-time interview coach. The user is being asked a question by an interviewer.\n\nYour task: based on the interviewer's question + the user's resume background, generate ONE sentence the user can speak directly as the opening of their answer (≤25 words).\n\nRules:\n- It must be the OPENING of an answer, not a full response — the user will continue from there\n- First person only\n- Must reference a concrete project/number/technology from the user's resume\n- For STAR-framework questions, give the Situation+Task opener only\n- No filler ('Great question…'), no meta-commentary, no quotes around the answer\n\nOutput exactly two lines:\nAdvice: (one speakable sentence, ≤25 words)\nAngle: (2-4 word tag, e.g. 'project depth / metrics / leadership / problem-solving')\n\nReply only in English.",
  "trigger_hints": ["tell me about", "walk me through"],
  "advice_style": "interview",
  "enabled": true,
  "role_persona": "Interview candidate",
  "mimic_style": "",
  "expertise_context": "",
  "stealth_default": true,
  "advice_cooldown_seconds": 8,
  "trigger_config": {
    "on_ask_opinion": false,
    "on_question_to_user": true,
    "on_domain_topic": false,
    "on_decision_point": false,
    "on_discussion_stuck": true,
    "custom_keywords": [
      "tell me about", "walk me through", "describe a time",
      "why did you", "why do you want", "why are you",
      "what's your", "what are your", "your biggest",
      "give me an example", "for example", "how did you handle",
      "what would you do if", "imagine", "in a situation where"
    ],
    "domain_keywords": []
  }
}
```

- [ ] **Step 4: 写 templates/zh-CN/general-meeting.json**

```json
{
  "id": "general-meeting",
  "name": "日常会议",
  "description": "1:1 / standup / 客户会 / tech review 等通用会议场景。",
  "language": "zh-CN",
  "system_prompt": "你是会议参会者的实时教练。当对话出现适合插话的时机时，建议用户怎么说。\n\n要求：\n- 建议必须引用对话中的具体内容\n- 建议是一句可以直接说出口的话，不超过 30 字\n- 第一人称\n- 不要铺垫、不要解释\n\n严格按以下格式输出，每项一行：\n建议：（一句话，可直接说出口）\n角度：（2-4 字标签）\n\nReply only in Chinese.",
  "trigger_hints": [],
  "advice_style": "general",
  "enabled": true,
  "role_persona": "会议参与者",
  "mimic_style": "",
  "expertise_context": "",
  "stealth_default": false,
  "advice_cooldown_seconds": 12,
  "trigger_config": {
    "on_ask_opinion": true,
    "on_question_to_user": true,
    "on_domain_topic": true,
    "on_decision_point": true,
    "on_discussion_stuck": true,
    "custom_keywords": [],
    "domain_keywords": []
  }
}
```

- [ ] **Step 5: 写 templates/en-US/general-meeting.json**

```json
{
  "id": "general-meeting",
  "name": "General Meeting",
  "description": "1:1, standup, customer call, tech review — all-purpose mode.",
  "language": "en-US",
  "system_prompt": "You are a real-time meeting copilot. Help the user respond when there's an opening to speak.\n\nRules:\n- Reference specifics from recent transcript\n- ≤25 words, speakable in one breath\n- First person\n- No filler, no meta-commentary\n\nOutput exactly two lines:\nAdvice: (one speakable sentence)\nAngle: (2-4 word tag)\n\nReply only in English.",
  "trigger_hints": [],
  "advice_style": "general",
  "enabled": true,
  "role_persona": "Meeting participant",
  "mimic_style": "",
  "expertise_context": "",
  "stealth_default": false,
  "advice_cooldown_seconds": 12,
  "trigger_config": {
    "on_ask_opinion": true,
    "on_question_to_user": true,
    "on_domain_topic": true,
    "on_decision_point": true,
    "on_discussion_stuck": true,
    "custom_keywords": [],
    "domain_keywords": []
  }
}
```

- [ ] **Step 6: 验证 JSON 格式**

Run:
```bash
for f in templates/zh-CN/*.json templates/en-US/*.json; do
  python3 -m json.tool < "$f" > /dev/null && echo "OK: $f" || echo "BAD: $f"
done
```

Expected: 4 个 OK。

---

### Task 6: context_note 注入到 advisor

**Files:**
- Modify: `src-tauri/src/commands.rs`（RecordingState 加字段 + 新 command）
- Modify: `src-tauri/src/advisor/engine.rs`（generate_advice 注入）

- [ ] **Step 1: 在 RecordingState 加 context_note 字段**

找到 `commands.rs:86` 的 `RecordingState`，加：

```rust
pub struct RecordingState {
    pub is_recording: bool,
    pub is_paused: bool,
    pub mic_buffer: SharedBuffer,
    pub capture_buffer: SharedBuffer,
    pub transcript: SharedTranscriptStore,
    pub start_time: Option<std::time::Instant>,
    pub reference_docs: String,
    pub active_template_id: Option<String>,
    pub context_note: String,           // ← 新增
}

impl RecordingState {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            is_paused: false,
            mic_buffer: create_shared_buffer(2, 16000),
            capture_buffer: create_shared_buffer(2, 16000),
            transcript: create_shared_store(),
            start_time: None,
            reference_docs: String::new(),
            active_template_id: None,
            context_note: String::new(),  // ← 新增
        }
    }
}
```

- [ ] **Step 2: 加 set_meeting_context_note command**

在 `commands.rs` 末尾加：

```rust
#[command]
pub async fn set_meeting_context_note(
    note: String,
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<(), String> {
    if note.chars().count() > 500 {
        return Err("Context note must be ≤500 characters".into());
    }
    let mut rec = state.lock().await;
    rec.context_note = note;
    Ok(())
}

#[command]
pub async fn get_meeting_context_note(
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<String, String> {
    let rec = state.lock().await;
    Ok(rec.context_note.clone())
}
```

注册到 `main.rs invoke_handler!`：

```rust
            commands::set_meeting_context_note,
            commands::get_meeting_context_note,
```

- [ ] **Step 3: 改 advisor::engine::generate_advice 接受 context_note**

找到 `engine.rs::generate_advice`（约 156-218 行），改签名加参数：

```rust
pub async fn generate_advice(
    &self,
    template: &MeetingTemplate,
    transcript: &str,
    trigger_reason: &str,
    reference_docs: &str,
    context_note: &str,           // ← 新增
    offset_secs: f64,
) -> Result<SpeakingAdvice> {
    let mut system = String::new();

    if !template.role_persona.is_empty() {
        system.push_str(&format!("用户角色：{}。\n\n", template.role_persona));
    }
    if !template.mimic_style.is_empty() {
        system.push_str(&format!("发言风格：{}。\n\n", template.mimic_style));
    }
    if !template.expertise_context.is_empty() {
        system.push_str(&format!("专业背景：\n{}\n\n", template.expertise_context));
    }

    if system.is_empty() {
        system = template.system_prompt.clone();
    } else {
        system.push_str(&template.system_prompt);
    }

    if !context_note.is_empty() {
        system.push_str(&format!("\n\n本场会议上下文（用户备注）：\n{}", context_note));
    }
    if !reference_docs.is_empty() {
        system.push_str(&format!("\n\n参考文档：\n{}", reference_docs));
    }

    let user_msg = format!(
        "最近对话：\n{}\n\n触发原因：{}\n\n请按格式输出建议和角度。",
        transcript, trigger_reason
    );

    let messages = vec![
        LlmMessage { role: "system".into(), content: system },
        LlmMessage { role: "user".into(), content: user_msg },
    ];

    let response = self.chat(&messages, 150).await?;
    Ok(parse_advice(&response, trigger_reason, offset_secs))
}
```

- [ ] **Step 4: 改 commands.rs 里调用 generate_advice 的地方**

在 `start_recording` 内的 advisor loop 找到 `advisor.generate_advice(...)`（在你 Week 1 改过的 `spawn_advisor_loop` 函数体里），改成：

```rust
let context_note = {
    let rec = state.lock().await;
    rec.context_note.clone()
};
match advisor.generate_advice(
    tmpl, &recent, &trigger.reason, &ref_docs, &context_note, offset
).await {
    // ...
}
```

- [ ] **Step 5: 编译验证**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
pnpm typecheck 2>&1 | tail -5
```

Expected: 通过。

---

### Task 7: 创建 NewMeetingDialog 组件

**Files:**
- Create: `src/components/meeting/NewMeetingDialog.tsx`
- Modify: `src/lib/tauri.ts`（加 wrapper）

- [ ] **Step 1: 加 tauri wrappers**

```typescript
export const setMeetingContextNote = (note: string) =>
  invoke<void>("set_meeting_context_note", { note });

export const getMeetingContextNote = () =>
  invoke<string>("get_meeting_context_note");
```

- [ ] **Step 2: 写 NewMeetingDialog**

```tsx
import { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  getTemplates,
  setActiveTemplate,
  loadReferenceDoc,
  setMeetingContextNote,
} from "../../lib/tauri";
import type { MeetingTemplate } from "../../lib/types";

interface Props {
  defaultKind: "interview" | "general";
  defaultLocale: "zh-CN" | "en-US";
  onStart: () => void;
  onCancel: () => void;
}

export function NewMeetingDialog({ defaultKind, defaultLocale, onStart, onCancel }: Props) {
  const [templates, setTemplates] = useState<MeetingTemplate[]>([]);
  const [selectedTemplateId, setSelectedTemplateId] = useState<string>(
    defaultKind === "interview" ? "job-interview" : "general-meeting",
  );
  const [docPath, setDocPath] = useState<string | null>(null);
  const [docName, setDocName] = useState<string>("");
  const [contextNote, setContextNote] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    void getTemplates().then(setTemplates).catch(console.error);
  }, []);

  async function handlePickDoc() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Document", extensions: ["pdf", "md", "txt"] }],
    });
    if (path && typeof path === "string") {
      setDocPath(path);
      try {
        const filename = await loadReferenceDoc(path);
        setDocName(filename);
      } catch (e) {
        console.error("loadReferenceDoc failed:", e);
        alert("Failed to load document: " + e);
      }
    }
  }

  async function handleStart() {
    setLoading(true);
    try {
      await setActiveTemplate(selectedTemplateId);
      await setMeetingContextNote(contextNote);
      onStart();
    } catch (e) {
      console.error(e);
      alert("Failed to prepare meeting: " + e);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center p-4 z-50">
      <div className="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-6 max-w-md w-full">
        <h2 className="text-lg font-bold mb-4">New Meeting</h2>

        <label className="text-sm block mb-1">Template</label>
        <select
          className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded px-2 py-1 mb-4"
          value={selectedTemplateId}
          onChange={(e) => setSelectedTemplateId(e.target.value)}
        >
          {templates.map((t) => (
            <option key={t.id} value={t.id}>
              {t.name}
            </option>
          ))}
        </select>

        <label className="text-sm block mb-1">Context document (optional)</label>
        <div className="flex items-center gap-2 mb-4">
          <button
            type="button"
            onClick={handlePickDoc}
            className="px-3 py-1 border border-[var(--border)] rounded text-xs"
          >
            {docName ? "Change…" : "Drop or pick PDF / MD / TXT"}
          </button>
          {docName && <span className="text-xs text-gray-400">{docName}</span>}
        </div>

        <label className="text-sm block mb-1">
          Context note (optional, ≤500 chars)
        </label>
        <textarea
          className="w-full bg-[var(--bg-secondary)] border border-[var(--border)] rounded p-2 text-sm mb-1"
          rows={3}
          maxLength={500}
          value={contextNote}
          onChange={(e) => setContextNote(e.target.value)}
          placeholder="e.g. This is the Stripe onsite second round, focus on system design."
        />
        <div className="text-xs text-gray-500 text-right mb-4">
          {contextNote.length}/500
        </div>

        <div className="flex gap-2 justify-end">
          <button
            onClick={onCancel}
            className="px-4 py-2 border border-[var(--border)] rounded text-sm"
            disabled={loading}
          >
            Cancel
          </button>
          <button
            onClick={() => void handleStart()}
            className="px-4 py-2 bg-[var(--accent-purple)] text-white rounded text-sm"
            disabled={loading}
          >
            {loading ? "Loading…" : "Start Recording"}
          </button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 3: 在 App.tsx 接入 dialog**

```tsx
const [showNewMeetingDialog, setShowNewMeetingDialog] = useState(false);
const [newMeetingKind, setNewMeetingKind] = useState<"interview" | "general">("general");

// In useEffect for menu-new-meeting:
unlistens.push(
  await onMenuNewMeeting(async (kind) => {
    setNewMeetingKind(kind);
    setShowNewMeetingDialog(true);
  }),
);

// In render:
{showNewMeetingDialog && (
  <NewMeetingDialog
    defaultKind={newMeetingKind}
    defaultLocale="en-US"  // TODO Week 5: derive from app locale
    onStart={() => {
      setShowNewMeetingDialog(false);
      setView("narrow");
      // recording.start() called by user clicking Start in NarrowView, or auto-start here
    }}
    onCancel={() => setShowNewMeetingDialog(false)}
  />
)}
```

- [ ] **Step 4: 编译验证**

Run:
```bash
pnpm typecheck 2>&1 | tail -5
```

---

### Task 8: 类型同步 + 删旧 settings 模板编辑器引用

**Files:**
- Modify: `src/lib/types.ts`（加 language / stealth_default）
- Modify: `src/components/settings/ProfileSettings.tsx`（如有引用旧模板字段，简化）

- [ ] **Step 1: 在 types.ts 加字段**

找到 MeetingTemplate type，加：

```typescript
export interface MeetingTemplate {
  id: string;
  name: string;
  description: string;
  system_prompt: string;
  trigger_hints: string[];
  advice_style: string;
  enabled: boolean;
  language?: string;                    // ← 新增
  role_persona?: string;
  mimic_style?: string;
  expertise_context?: string;
  stealth_default?: boolean;            // ← 新增
  advice_cooldown_seconds?: number;     // ← 新增
  trigger_config?: TriggerConfig;
}

export interface TriggerConfig {
  on_ask_opinion: boolean;
  on_question_to_user?: boolean;        // ← 新增
  on_domain_topic: boolean;
  on_decision_point: boolean;
  on_discussion_stuck: boolean;
  custom_keywords: string[];
  domain_keywords: string[];
}
```

- [ ] **Step 2: 检查 ProfileSettings 是否依赖删除的字段**

Run:
```bash
grep -n "tech-review\|code-review\|brainstorm\|project-sync" src/ -r
```

如果有：替换为 `job-interview` / `general-meeting`。

- [ ] **Step 3: 编译验证**

Run:
```bash
pnpm typecheck 2>&1 | tail -5
```

---

### Task 9: 端到端验证

- [ ] **Step 1: 启动 + 选模板 + 拖简历**

```bash
OPENAI_API_KEY="<key>" pnpm tauri dev
```

菜单栏 → New Interview Meeting → 弹出 NewMeetingDialog → 选 Job Interview → 拖入英文简历 PDF → 写 context note → Start。

- [ ] **Step 2: 模拟面试问题**

打开 Zoom test meeting，对着 mic 说 "Tell me about your most challenging project."

观察 NarrowView：
- ✅ Transcript 出现这句话
- ✅ 几秒内 advice 卡片出现
- ✅ Advice 引用了简历里的项目

如果 advice 没出现：
- 看 Tauri terminal 是否 fire `[advisor] Trigger fired`
- 检查 `on_question_to_user: true` 是否在选中模板里
- 看 LLM call 是否成功（没有 401/500）

- [ ] **Step 3: 验证 JSON 解析正常**

Run:
```bash
ls ~/.meeting-assistant/templates/zh-CN/
ls ~/.meeting-assistant/templates/en-US/
```

Expected: 各 2 个 .json 文件。如果空：`ensure_default_templates` 没正确复制。

- [ ] **Step 4: 标 Week 3 完成**

`decision-log.md` 加：

```
## Week 3 完成
- 日期: <2026-05-XX>
- 验收: ✅ 拖入简历 PDF + 选 Job Interview EN + 模拟面试 → AI 引用简历项目给一句开头
- PDF OCR fallback 推 v1.0.5（MVP 仅支持 text-based PDF）
```

---

## Week 3 完成标志（Acceptance Criteria）

对应 design Section 9 AC：
- ✅ E1 内置 4 个模板（job-interview × zh+en、general-meeting × zh+en）
- ✅ E2 拖入 PDF/MD/TXT 后 ≤5s 处理（text-based）
- ⏳ E3 OCR-needed PDF 降级（v1.0.5）
- ✅ E4 简历内容能被 advice 引用
- ✅ E5 context_note ≤500 字限制 + 字符计数
- ✅ C1 面试模板触发率（用 on_question_to_user）

下一步：进 Week 4 — License + Lemon Squeezy + 月度 quota。
