# Meeting Copilot

macOS 实时会议助手。自动转写音频、生成会议摘要、智能提示发言时机和内容 —— 语音识别完全本地运行。

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue) ![React](https://img.shields.io/badge/React-19-blue) ![Rust](https://img.shields.io/badge/Rust-2021-orange)

**[English](README.md)**

---

## 功能特性

- **实时转写** — 基于 SenseVoice 模型（sherpa-onnx）+ Silero VAD，本地运行，支持中/英/日/韩/粤语
- **说话人分离** — 双通道独立识别：麦克风（"我"）与系统音频（"对方"）分开转写，自动标注说话人，内置回声抑制和跨通道去重
- **实时摘要** — LLM 每 30 秒自动滚动生成讨论要点
- **发言建议** — AI 检测适合发言的时机（提问、停顿、关键词命中），建议你说什么
- **双通道采集** — 同时录制麦克风和系统音频（通过 BlackHole 虚拟设备采集远端参会者声音）
- **会议模板** — 内置技术评审、代码评审、头脑风暴、项目同步等场景预设，配套完整模板编辑器
- **参考文档** — 加载会议议程或相关文档，让 AI 建议更贴合上下文
- **会议历史** — 自动保存转写、摘要和行动项到本地 SQLite，支持 Markdown 导出
- **会议纪要** — 会议结束时自动生成标题、要点和行动项
- **隐私优先** — 语音识别 100% 本地运行，音频不离开你的设备，仅转写文本发送至 LLM API

## 截图

应用以紧凑的置顶窗口（420×840）形式运行，贴靠在视频会议软件旁边：

- **窄视图** — 紧凑面板，显示实时转写、摘要和发言建议
- **全视图** — 展开布局，包含模板选择、完整转写记录和可调大小的 AI 助手面板

## 环境要求

- **macOS**（通过 cpal 调用 CoreAudio）
- **Rust**（最新稳定版）
- **Node.js** 18+ 和 **pnpm**
- **BlackHole 2ch** — 虚拟音频驱动，用于采集系统音频（[下载](https://existential.audio/blackhole/)）

### BlackHole 配置

1. 安装 BlackHole 2ch
2. 打开 **音频 MIDI 设置**（macOS 自带）
3. 创建 **多输出设备**，组合你的耳机 + BlackHole 2ch
4. 将多输出设备设为系统音频输出
5. 在 Meeting Copilot 设置中，选择 BlackHole 2ch 作为采集设备

## 快速开始

```bash
# 克隆并安装依赖
git clone <repo-url>
cd meeting-assistant
pnpm install

# 开发模式运行（首次编译 Rust 约 5 分钟）
pnpm tauri dev
```

首次启动：
1. 应用会自动下载 SenseVoice 模型（约 200 MB）+ Silero VAD
2. 在设置中配置音频设备（麦克风 + BlackHole）
3. 在设置中填入 LLM API Key
4. 点击 **+ 新建会议** 开始录制

## 配置说明

设置保存在 `~/.meeting-assistant/config.json`：

```json
{
  "llm": {
    "base_url": "https://llmgate.io/v1",
    "api_key": "your-api-key",
    "model": "gpt-5.4"
  },
  "audio": {
    "mic_device": "MacBook Pro Microphone",
    "capture_device": "BlackHole 2ch",
    "noise_reduction": true
  },
  "language_preference": "auto",
  "analysis_mode": "balanced"
}
```

LLM 接口需兼容 OpenAI 格式（`/v1/chat/completions`），任何兼容的服务商均可使用。

## 会议模板

模板用于定制不同会议场景下 AI 的行为方式，位于 `templates/` 目录：

| 模板 | 适用场景 | 触发示例 |
|------|---------|---------|
| tech-review | 架构与设计评审 | 征求意见、提及方案 |
| code-review | 代码评审讨论 | 评审提问、反馈提示 |
| brainstorm | 头脑风暴 | 征集想法、开放讨论 |
| project-sync | 项目状态同步 | 进度确认、阻塞提及 |

每个模板包含 `system_prompt`（塑造 AI 建议风格）和 `trigger_hints`（触发关键词，转写中检测到时激活发言建议）。

## 语音识别架构

基于 **sherpa-onnx** 框架，核心组件：

| 组件 | 模型 | 说明 |
|------|------|------|
| ASR | **SenseVoice**（int8 量化） | 阿里达摩院多语言模型，支持中/英/日/韩/粤，约 200 MB |
| VAD | **Silero VAD** | 轻量语音活动检测，512 采样点（32 ms）窗口 |
| 推理 | **ONNX Runtime** | 通过 sherpa-onnx 调用，纯 CPU 推理，无需 GPU |

### 处理流程

1. **双通道采集** — 麦克风和系统音频分别采集为 16 kHz 单声道 f32 PCM 流
2. **独立 VAD + ASR** — 每个通道运行独立的 Silero VAD → SenseVoice 流水线
3. **语音分段** — VAD 在连续语音 ≥250 ms 后激活，静音 ≥250 ms 后切段，单段最长 8 秒
4. **回声抑制** — 系统音频通道活跃时，抑制麦克风通道的输出（消除扬声器回声串入麦克风）
5. **跨通道去重** — 3 秒窗口内字符相似度 >50% 的重复段自动过滤

模型存储在 `~/.meeting-assistant/models/`，首次启动时自动下载。

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2 |
| 前端 | React 19、TypeScript、Tailwind CSS 4 |
| 构建工具 | Vite 6 |
| 后端 | Rust（2021 edition） |
| 音频采集 | cpal + BlackHole 2ch |
| 语音转文字 | sherpa-onnx（SenseVoice + Silero VAD） |
| LLM | OpenAI 兼容 API |
| 数据库 | SQLite（rusqlite） |
| 包管理 | pnpm |

## 开发

```bash
# 仅前端热重载（:1420 端口）
pnpm dev

# 完整应用（Tauri + 前端）
pnpm tauri dev

# 类型检查
pnpm typecheck              # 前端
cd src-tauri && cargo check  # 后端

# 测试
cd src-tauri && cargo test

# 生产构建
pnpm tauri build
```

## 项目结构

```
meeting-assistant/
├── src/                          # React 前端
│   ├── components/
│   │   ├── narrow/               # 紧凑面板视图
│   │   ├── full/                 # 展开视图
│   │   ├── settings/             # 设置页
│   │   ├── history/              # 会议历史
│   │   └── shared/               # 公共组件
│   ├── hooks/                    # useRecording, useTauriEvents
│   └── lib/                      # Tauri API 封装、类型定义
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── audio/                # 音频采集与缓冲
│   │   ├── whisper/              # SenseVoice + Silero VAD 引擎
│   │   ├── advisor/              # LLM 集成与触发规则
│   │   ├── transcript/           # 转写存储
│   │   ├── storage/              # 配置与历史（SQLite）
│   │   ├── documents/            # 参考文档加载
│   │   └── commands.rs           # 所有 Tauri 命令
│   └── icons/                    # 应用图标
└── templates/                    # 会议模板（JSON）
```

## 数据存储

所有数据保存在本地：

| 数据 | 位置 |
|------|------|
| 应用配置 | `~/.meeting-assistant/config.json` |
| ASR 模型 | `~/.meeting-assistant/models/` |
| 会议历史 | `~/.meeting-assistant/history.db` |

## 许可证

私有项目。
