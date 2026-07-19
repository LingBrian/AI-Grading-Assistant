# AI 阅卷助手 (AI Grading Assistant)

基于 Tauri v2 + Vue 3 的桌面端 AI 辅助阅卷工具，支持截图批改、评分标准管理、历史记录等功能。

## 技术栈

| 层面     | 技术                                          |
| -------- | --------------------------------------------- |
| 前端     | Vue 3 + TypeScript + Pinia + Vue Router       |
| UI       | Element Plus                                  |
| 桌面框架 | Tauri v2 (Rust)                               |
| 后端     | Rust (rusqlite, reqwest, xcap, image, base64) |
| 数据库   | SQLite (bundled via rusqlite)                  |

## 前置条件

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) (stable)
- Windows: [Microsoft Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 或 Visual Studio (C++ 工作负载)
- 系统依赖参考 [Tauri v2 前置条件文档](https://v2.tauri.app/start/prerequisites/)

## 开发

```bash
# 1. 安装前端依赖
npm install

# 2. 启动开发模式（自动启动 Vite HMR + Tauri 窗口）
npm run tauri dev
```

Tauri 开发模式会先启动 Vite 开发服务器（端口 5173），然后打开 Tauri 应用窗口，支持热更新。

## 构建

```bash
# 构建前端 + Rust 二进制 + 安装包（MSI + NSIS）
npm run tauri build

# 或者一键构建并汇集产物到版本目录
npm run release
```

产物位于 `src-tauri/target/release/v{版本号}/`：

| 文件 | 说明 |
|------|------|
| `AI阅卷助手_v{版本}_x64.exe` | 便携版（免安装，直接运行） |
| `AI阅卷助手_{版本}_x64_zh-CN.msi` | MSI 安装包 |
| `AI阅卷助手_{版本}_x64-setup.exe` | NSIS 安装程序 |

## 项目结构

```
AIGA-Tauri/
├── src/                    # Vue 前端源码
│   ├── components/         # 组件（FloatingToolbar, ResultDrawer, 等）
│   ├── pages/              # 页面级组件（HistoryDialog, TemplateDialog）
│   ├── stores/             # Pinia 状态管理
│   ├── styles/             # 全局样式
│   ├── types/              # TypeScript 类型定义
│   ├── utils/              # 工具函数
│   ├── App.vue             # 根组件
│   └── main.ts             # 入口
├── src-tauri/              # Rust 后端源码
│   ├── src/
│   │   ├── commands/       # Tauri IPC 命令
│   │   ├── ai.rs           # AI 评分逻辑
│   │   ├── db.rs           # SQLite 数据库
│   │   └── lib.rs          # Tauri 应用入口、窗口管理
│   ├── icons/              # 应用图标
│   ├── Cargo.toml
│   └── tauri.conf.json
├── scripts/              # 构建工具脚本
├── package.json
└── README.md
```

## 功能

- **悬浮工具栏**：全局置顶，支持拖拽，快捷键截图
- **截图批改**：框选区域后调用 AI 模型自动评分
- **评分标准管理**：多题目、评分细则、参考答案（含图片）
- **模板管理**：保存/导入/导出评分模板
- **设置**：API Key / Base URL / 模型 / 快捷键 / 代理等
- **历史记录**：评分记录查询、详情查看、删除
- **系统托盘**：关闭窗口后后台运行，托盘菜单唤出

## 许可

MIT
