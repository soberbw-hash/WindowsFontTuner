<h1 align="center">WindowsFontTuner 2.0</h1>

<p align="center">
  <strong>把 Windows 全局字体与文本渲染从“注册表体操”变成可预览、可备份、可回退的视觉调音台。</strong><br />
  预设字体方案 · DPI 感知渲染 · 自动备份 · 一键恢复 · Shift 急救
</p>

<p align="center">
  <a href="https://github.com/soberbw-hash/WindowsFontTuner/releases/latest"><img alt="GitHub Release" src="https://img.shields.io/github/v/release/soberbw-hash/WindowsFontTuner?display_name=tag&sort=semver" /></a>
  <img alt="Windows 10 / 11" src="https://img.shields.io/badge/Windows-10%20%2F%2011-1675F2?logo=windows11&logoColor=white" />
  <img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white" />
  <img alt="Rust" src="https://img.shields.io/badge/Rust-native%20core-B7410E?logo=rust&logoColor=white" />
  <a href="LICENSE"><img alt="MIT License" src="https://img.shields.io/badge/license-MIT-2563EB" /></a>
</p>

<p align="center">
  <a href="#下载与安装">下载</a> ·
  <a href="#核心能力">功能</a> ·
  <a href="#字体方案">字体方案</a> ·
  <a href="#安全与恢复">安全恢复</a> ·
  <a href="#从源码开发">开发</a> ·
  <a href="#许可与字体版权">许可</a>
</p>

> [!WARNING]
> 本工具会安装字体并修改当前用户/系统的字体映射与文本渲染注册表。虽然每次应用前都会备份，仍建议先保存工作、关闭重要应用，并确保知道如何使用“恢复 Windows 默认”和 Shift 急救模式。公司或受管理电脑应先取得管理员授权。

## 项目状态

- 当前正式版：`v2.2.4`；
- 当前主线：`WindowsFontTuner2/` 下的 Rust + Tauri 2 + React 版本；
- 支持平台：Windows 10 / 11 x64；
- 发布形式：NSIS 安装版，发布流程也保留便携包；
- 旧版实现：根目录 C# / WinForms 文件仅作历史参考，不再作为主线 UI 迭代。

## 为什么需要 WindowsFontTuner

Windows 全局字体替换往往涉及字体安装、`FontSubstitutes`、`FontLink`、桌面平滑参数、WPF `Avalon.Graphics` 和 DPI 差异。手动修改注册表很难预览、容易漏项，也很难在出问题时确认应该恢复哪一组值。

WindowsFontTuner 把这一流程整理为：

1. 选择字体方案；
2. 选择渲染风格；
3. 查看当前系统与将要应用的 A/B 预览；
4. 查看本次变更摘要；
5. 自动备份；
6. 安装/确认字体并写入映射；
7. 广播系统刷新；
8. 必要时按组件或整体恢复。

它不只是“换字体”，而是把字体族、回退链和屏幕渲染参数作为一套可回滚方案管理。

## 核心能力

- 一键应用预设字体方案；
- `清晰 / 平衡 / 柔和 / 阅读 / 代码 / 圆润观感` 六档渲染风格；
- 当前系统与待应用方案的 A/B 预览；
- 启动时识别当前字体和渲染状态；
- 根据显示器分辨率与 DPI 调整渲染矩阵；
- 根据屏幕条件给出风格建议；
- 应用前备份 `FontSubstitutes`、`FontLink`、`Desktop` 和 `Avalon.Graphics`；
- 整体恢复 Windows 原生映射与渲染参数；
- 按字体映射、回退链或渲染参数分别恢复；
- 调用 `DISM /RestoreHealth` 和 `sfc /scannow` 修复系统字体文件；
- 导入本地 TTF/OTF/OTC 字体；
- 导入/导出可分享的方案文件；
- 为允许公开分发的预设字体自动下载并安装资源；
- 补齐 `Segoe UI Emoji` 与 `Microsoft YaHei` 回退链，降低 Emoji 和生僻字缺字概率；
- 无界面 Shift 急救恢复。

## 字体方案

| 方案 | 状态 | 适合场景 | 安装方式 |
| --- | --- | --- | --- |
| HarmonyOS Sans SC | 内置支持 | 现代、干净、适合长期日用 | 自动下载/安装所需字重 |
| Sarasa UI SC（更纱黑体） | 内置支持 | 代码、文档、资源管理器，笔画更扎实 | 自动下载/安装 |
| Source Han Sans CN（思源黑体） | 内置支持 | 中性、克制、跨平台一致 | 自动下载/安装 |
| LXGW WenKai（霞鹜文楷） | 内置支持 | 写作和阅读，观感温润 | 从官方 Release 下载 |
| OPPOSans 3.0 | 可用 | 现代中文 UI | 用户自行取得并导入字体文件 |
| Inter + HarmonyOS | 内置支持 | 英文/数字节奏与中文稳定性兼顾 | 自动下载/安装组合字体 |

字体可出现在软件列表中，不代表本项目拥有该字体的全部权利。每套字体的分发、修改、嵌入和商用范围以原字体许可为准。

## 渲染风格与 DPI

字体文件决定字形，Windows 的平滑、Gamma、对比和灰度设置决定最终屏幕观感。同一个字体在 100%、125%、150% 缩放和不同面板上可能差异明显。

六档风格用于表达不同取向：

- **清晰**：强调边缘与识别度；
- **平衡**：日常默认，兼顾文字密度和柔和度；
- **柔和**：降低锐利感；
- **阅读**：面向长文本和较长使用时间；
- **代码**：强调英文、数字与细节分辨；
- **圆润观感**：偏向更饱满、亲和的 UI 效果。

推荐只是起点。最终应在常用显示器、缩放比例和软件中检查中文、英文、数字、Emoji、粗体、细体和小字号。

## 下载与安装

前往 [Releases](https://github.com/soberbw-hash/WindowsFontTuner/releases/latest) 下载最新版。

建议安装流程：

1. 只从本仓库 Release 下载；
2. 关闭正在编辑重要文档的应用；
3. 启动后先查看当前系统检测结果；
4. 选择字体方案与渲染风格，阅读应用摘要；
5. 确认备份成功后应用；
6. 注销或重启相关应用，必要时注销 Windows；
7. 检查资源管理器、浏览器、Office、代码编辑器和 Emoji；
8. 不满意时恢复上一次备份或 Windows 默认。

系统策略、杀毒软件或 SmartScreen 可能拦截未签名程序。遇到提示时先核对下载来源，不要使用第三方重新打包版本。

## 安全与恢复

### 自动备份

每次真正写入前，程序会在以下位置创建时间戳备份目录：

```text
%LOCALAPPDATA%\WindowsFontTuner\Backups\backup-*
```

备份包含：

```text
FontSubstitutes.reg
FontLink.reg
Desktop.reg
Avalon.Graphics.reg
```

自动下载的字体临时内容位于：

```text
%LOCALAPPDATA%\WindowsFontTuner\Downloads
```

导出的方案位于本地应用数据下的 `WindowsFontTuner\Exports`。

### 恢复方式

- **恢复上一次备份**：恢复最近一次写入前的注册表状态；
- **按组件恢复**：分别恢复字体映射、FontLink 回退链或渲染参数；
- **恢复 Windows 默认**：重置本工具管理的映射与渲染设置；
- **修复系统字体文件**：依次运行 DISM 和 SFC；
- **Shift 急救模式**：界面字体已经看不清时，无界面恢复本工具记录的设置。

### Shift 急救模式

如果应用字体后系统文字严重异常：

1. 按住 `Shift`；
2. 双击启动 WindowsFontTuner；
3. 程序跳过常规界面，执行无头恢复；
4. 等待刷新完成；
5. 如仍异常，注销 Windows 或重启；
6. 必要时以管理员终端运行 DISM 和 SFC。

急救模式只能恢复本工具记录和管理的范围，不能保证修复其它软件或手工注册表修改造成的问题。

## 应用后没有变化怎么办

Windows 不同应用使用 GDI、DirectWrite、WPF、Chromium 或自有渲染器，刷新时机不同。可以依次尝试：

1. 完全退出并重开目标应用；
2. 重启资源管理器；
3. 注销当前 Windows 用户；
4. 重启系统；
5. 检查应用是否硬编码自己的字体；
6. 检查字体文件是否安装完整、名称是否匹配；
7. 使用 A/B 预览确认不是缩放或渲染差异。

部分应用不会遵循系统字体替换，这是应用自身渲染策略，不应通过无限扩大注册表替换范围强行解决。

## 导入自定义字体

支持导入 TTF、OTF 和 OTC 文件。导入前请确认：

- 字体文件来自可信来源；
- 许可允许你安装和使用；
- 字体包含所需中文、英文和符号字形；
- 字体内部 Family Name 与文件名可能不同；
- 不把未知字体作为唯一系统字体；
- 保留至少一套可靠回退字体和最近备份。

OPPOSans 因分发条件不适合直接打入公开 Release，需要用户自行取得合法字体文件后导入。

## 技术架构

```text
React / TypeScript UI
├─ 字体和渲染风格选择
├─ A/B 预览、应用摘要和恢复界面
└─ 通过 Tauri IPC 提交结构化方案 ID
              │
              ▼
Rust / Tauri Core
├─ 当前系统和显示器/DPI 检测
├─ 字体文件解析、下载与安装
├─ 注册表备份、写入与恢复
├─ 方案导入/导出
├─ 系统刷新与无头急救
└─ DISM / SFC 修复入口
              │
              ▼
Windows 字体存储、注册表、GDI / WPF 渲染设置
```

主线技术：Rust、Tauri 2、React 19、TypeScript、Vite、Windows API、`winreg`、`ttf-parser` 和 `reqwest`。

## 从源码开发

### 环境要求

- Windows 10 / 11；
- Node.js 20+（建议使用当前 LTS）；
- npm；
- Rust stable；
- Visual Studio 2022 C++ Build Tools；
- WebView2 Runtime。

### 启动主线项目

```powershell
git clone https://github.com/soberbw-hash/WindowsFontTuner.git
cd WindowsFontTuner\WindowsFontTuner2
npm install
npm run tauri dev
```

### 前端构建与 Rust 检查

```powershell
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

### 生成安装包

```powershell
npm run tauri build
```

Tauri 默认生成 NSIS 安装包。仓库根目录还保留旧版 C# 工程和历史发布脚本；开发新功能时应以 `WindowsFontTuner2/` 为准。

## 变更验证

涉及字体或注册表的改动，建议在可恢复测试机中覆盖：

- 100%、125%、150% 和高 DPI 显示；
- 单屏、多屏和不同缩放组合；
- 字体已安装、缺失、损坏和下载失败；
- TTF、OTF、OTC 导入；
- 普通权限和管理员权限；
- 应用前备份失败；
- 整体恢复与按组件恢复；
- Shift 无头急救；
- 中文、英文、数字、粗体、Emoji 和生僻字；
- 资源管理器、浏览器、Office、WPF 和常用编辑器。

不要只验证应用内预览；系统字体工具必须在真实 Windows 应用中检查。

## 项目结构

```text
WindowsFontTuner2/                当前 Tauri 2 主线
├─ src/                           React / TypeScript 界面
├─ src-tauri/src/system.rs        Windows 字体、注册表、备份与恢复
├─ src-tauri/src/preset_data.rs   字体方案和下载清单
└─ src-tauri/tauri.conf.json      窗口与打包配置
FontPackages/                     可分发字体包、来源和原许可
Presets/                          旧版/兼容字体方案数据
installer/                        安装与卸载脚本
scripts/                          发布和图标脚本
*.cs                              历史 C# / WinForms 实现
字体授权说明.md                   字体来源与分发边界
LICENSE                           项目代码 MIT 许可
```

## 许可与字体版权

WindowsFontTuner 自有代码按 [MIT License](LICENSE) 开放。MIT 允许任何人使用、复制、修改、分发、再许可和销售软件副本，但必须保留版权和许可声明；软件按现状提供，不附带担保。

代码许可不等于字体许可。仓库和 Release 中的字体继续适用各自条款：

- HarmonyOS Sans SC：保留其原始许可与来源；
- Source Han Sans CN：保留其原始开源字体许可；
- Sarasa UI SC：保留其原始许可；
- LXGW WenKai、Inter 等下载资源：以对应上游版本许可为准；
- Windows 自带字体（Segoe UI、Microsoft YaHei 等）不随项目重新分发；
- OPPOSans、MiSans 等受额外分发限制的字体不应未经许可打入公开包。

详细来源和边界见 [字体授权说明.md](字体授权说明.md) 以及各 `FontPackages/*/LICENSE.txt`、`SOURCE.txt`。发布新字体包前必须同时核对字体文件、原许可证、来源 URL、版本和是否允许再分发。

> MIT 许可保护版权声明，但不会阻止别人合法复用代码或实现相似功能。如果项目未来需要限制商用、闭源改版或重新分发，必须在发布相应代码前重新设计授权策略；已按 MIT 获得的历史版本权利通常不能被追溯撤回。

