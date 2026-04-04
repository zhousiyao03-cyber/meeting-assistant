# Meeting Assistant

实时会议助手，基于 Tauri 2 + React 构建的桌面应用。通过本地 Whisper 模型进行语音转录，结合 LLM 提供会议摘要和发言建议。

## 核心功能

- **实时语音转录** — 基于 whisper.cpp，支持麦克风和系统音频双通道采集，本地运行无需联网
- **会议摘要** — 自动提取讨论要点和当前话题，每 30 秒更新
- **发言建议** — 根据会议模板和上下文，在合适时机给出发言建议
- **会议模板** — 内置头脑风暴、代码评审、技术评审、项目同步等场景模板，支持自定义
- **文档加载** — 支持加载参考文档作为 LLM 上下文
- **会议历史** — 本地 SQLite 存储，可回顾历史会议记录
- **双视图模式** — 窄视图（悬浮窗）和全视图，支持置顶显示

## 技术栈

| 层级 | 技术 |
|------|------|
| 框架 | Tauri 2 |
| 前端 | React 19 + TypeScript + Tailwind CSS 4 |
| 后端 | Rust + Tokio |
| 语音识别 | whisper-rs (whisper.cpp) |
| 音频采集 | cpal |
| 存储 | SQLite (rusqlite) |
| 构建工具 | Vite 6 |

## 项目结构

```
src/                    # 前端 (React)
├── components/
│   ├── full/           # 全视图组件
│   ├── narrow/         # 窄视图组件（悬浮窗）
│   ├── settings/       # 设置页面
│   └── shared/         # 公共组件
├── hooks/              # 自定义 Hooks
├── lib/                # 类型定义与 Tauri 桥接
└── styles/             # 全局样式

src-tauri/src/          # 后端 (Rust)
├── advisor/            # LLM 发言建议引擎
├── audio/              # 音频采集与缓冲
├── documents/          # 文档加载
├── storage/            # 配置与历史记录（SQLite）
├── transcript/         # 转录文本管理
└── whisper/            # Whisper 模型下载与推理

templates/              # 会议场景模板 (JSON)
```

## 快速开始

### 前置条件

- [Node.js](https://nodejs.org/) (>= 18)
- [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/) (stable)
- Tauri 2 系统依赖 — 参考 [Tauri 官方文档](https://v2.tauri.app/start/prerequisites/)

### 开发

```bash
# 安装前端依赖
pnpm install

# 启动开发模式（同时编译 Rust 后端）
pnpm tauri dev
```

首次启动时应用会引导下载 Whisper 模型。

### 构建

```bash
pnpm tauri build
```

## 配置

启动应用后进入 **设置页面** 进行配置：

- **音频设备** — 选择麦克风和系统音频捕获设备
- **LLM 服务** — 配置 API 地址、密钥和模型（兼容 OpenAI API 格式）
- **语言偏好** — 设置转录和建议的语言

## 许可证

MIT
