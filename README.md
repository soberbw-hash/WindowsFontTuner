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

> [!WARNING]
> 本工具会安装字体并修改当前用户/系统的字体映射与文本渲染注册表。虽然每次应用前都会备份，仍建议先保存工作、关闭重要应用，并确保知道如何使用“恢复 Windows 默认”和 Shift 急救模式。公司或受管理电脑应先取得管理员授权。

<p align="center">
  <a href="#about">项目介绍</a> ·
  <a href="#download">下载</a> ·
  <a href="#features">功能</a> ·
  <a href="#presets">字体方案</a> ·
  <a href="#safety">安全恢复</a> ·
  <a href="#development">开发</a> ·
  <a href="#faq">FAQ</a> ·
  <a href="#license">许可</a>
</p>

## 📌 项目状态

- 当前正式版：`v2.2.4`；
- 当前主线：`WindowsFontTuner2/` 下的 Rust + Tauri 2 + React 版本；
- 支持平台：Windows 10 / 11 x64；
- 发布形式：NSIS 安装版，发布流程也保留便携包；
- 旧版实现：根目录 C# / WinForms 文件仅作历史参考，不再作为主线 UI 迭代。

<a id="about"></a>

## WindowsFontTuner 是什么？

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

## 为什么做 WindowsFontTuner？

Windows 字体美化教程经常要求用户下载若干字体、复制注册表、重启资源管理器，再凭肉眼判断有没有生效。真正出问题时，却很少有人记得原来的值是什么。

手工方案常见的问题包括：

- 只替换 `Segoe UI`，遗漏 `Segoe UI Variable` 或中文 UI 字体；
- 没有设置 FontLink，导致 Emoji、生僻字或部分语言变方框；
- 把一个字体文件名误当成字体内部 Family Name；
- 同一份渲染参数在 100% 和 150% DPI 上观感完全不同；
- 字体没有坏，只是应用没有刷新；
- 恢复教程只给“默认值”，却不适配用户修改前的真实状态；
- 把不能再分发的商业/系统字体直接打进安装包。

WindowsFontTuner 希望把这件事从“复制神秘注册表”变成一套看得懂、能预览、有备份、能急救的流程。

## 核心设计原则

### 👀 应用前先预览

用户应看到当前系统和目标方案的 A/B 差异、将写入的字体族和渲染风格，而不是点击后才猜变化。

### 💾 写入前一定备份

备份是应用动作的一部分，不是高级选项。备份失败时不应继续假装安全。

### 🧬 字体、回退链和渲染是一套方案

字体替换不能只改主字体。中文、英文、数字、Emoji、生僻字和 WPF/GDI 渲染需要一起考虑。

### 🖥️ 尊重屏幕差异

分辨率、DPI、面板和用户偏好都会影响结果。推荐是起点，不存在所有屏幕都最优的唯一数值。

### 🚑 最坏情况下也要能恢复

即使界面文字已经难以辨认，按住 Shift 启动仍应走无头恢复路径，不能要求用户先在坏掉的界面里找到按钮。

### 📜 代码许可和字体许可分开

项目代码采用 MIT，不代表仓库里的所有字体也采用 MIT。每套字体必须保留自己的来源和许可。

<a id="features"></a>

## ✨ 功能一览

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

### 🎛️ 1. 字体方案 + 渲染风格

先选字体族，再从六档渲染风格选择观感。字体和渲染不再混成一组难以理解的注册表数值。

### 🔍 2. 当前系统识别

启动时读取现有字体映射、FontLink、桌面平滑和 WPF 渲染设置，判断当前更接近哪个预设。

### 🆚 3. A/B 预览

同屏展示当前系统和即将应用的效果，检查中文、英文、数字、小字号和常见 UI 文本。

### 🖥️ 4. DPI 感知推荐

读取显示器和缩放信息，按不同 DPI 组合生成渲染建议，避免低 DPI 参数直接套到高 DPI 屏幕。

### 📦 5. 字体下载与安装

对允许按当前方式分发的字体使用固定资源清单下载；安装前解析字体内部信息，并写入用户字体存储和注册表。

### 📥 6. 自定义字体导入

支持 TTF、OTF 和 OTC。用户可以导入未随项目分发、但自己有权使用的字体，例如 OPPOSans。

### 🔗 7. FontLink 回退链

为主字体补充中文、Emoji 和系统回退，降低只换主字体后出现方框、缺字和粗细不一致的概率。

### 💾 8. 自动注册表备份

每次写入前导出 `FontSubstitutes`、`FontLink`、`Desktop` 和 `Avalon.Graphics` 到时间戳目录。

### ↩️ 9. 整体或分项恢复

支持恢复上一次备份、Windows 默认，以及单独恢复字体映射、回退链或渲染参数。

### 🚑 10. Shift 无头急救

按住 Shift 启动时跳过常规 UI，直接尝试恢复最近记录的配置并刷新系统。

### 🩺 11. DISM + SFC 系统修复

字体文件真的损坏或系统组件异常时，打开 Windows 官方修复流程，而不是把微软字体打进项目重新分发。

### 📤 12. 方案导入与导出

把字体与渲染选择导出为结构化方案，便于在自己的设备之间记录和迁移；导入时仍需验证本机字体和许可。

### 🔔 13. 版本与发布信息

主线版本、安装包和便携包通过 GitHub Release 发布，开发目录与历史 C# 版本明确分开。

<a id="presets"></a>

## 🗂️ 字体方案

| 方案                           | 状态     | 适合场景                           | 安装方式                   |
| ------------------------------ | -------- | ---------------------------------- | -------------------------- |
| HarmonyOS Sans SC              | 内置支持 | 现代、干净、适合长期日用           | 自动下载/安装所需字重      |
| Sarasa UI SC（更纱黑体）       | 内置支持 | 代码、文档、资源管理器，笔画更扎实 | 自动下载/安装              |
| Source Han Sans CN（思源黑体） | 内置支持 | 中性、克制、跨平台一致             | 自动下载/安装              |
| LXGW WenKai（霞鹜文楷）        | 内置支持 | 写作和阅读，观感温润               | 从官方 Release 下载        |
| OPPOSans 3.0                   | 可用     | 现代中文 UI                        | 用户自行取得并导入字体文件 |
| Inter + HarmonyOS              | 内置支持 | 英文/数字节奏与中文稳定性兼顾      | 自动下载/安装组合字体      |

字体可出现在软件列表中，不代表本项目拥有该字体的全部权利。每套字体的分发、修改、嵌入和商用范围以原字体许可为准。

### 怎么选？

- 想要现代、稳妥、长期日用：先试 HarmonyOS Sans SC；
- 代码和资源管理器较多：试 Sarasa UI SC；
- 追求中性和跨平台一致：试 Source Han Sans CN；
- 写作和阅读为主：试 LXGW WenKai；
- 已合法取得 OPPOSans：通过自定义导入；
- 英文、数字和中文混排较多：试 Inter + HarmonyOS。

> [!TIP]
> 不要只在应用预览里选字体。至少检查资源管理器、浏览器标签、Office、代码编辑器、设置页、数字、Emoji 和包含生僻字的文本。

## 🖥️ 渲染风格与 DPI

字体文件决定字形，Windows 的平滑、Gamma、对比和灰度设置决定最终屏幕观感。同一个字体在 100%、125%、150% 缩放和不同面板上可能差异明显。

六档风格用于表达不同取向：

- **清晰**：强调边缘与识别度；
- **平衡**：日常默认，兼顾文字密度和柔和度；
- **柔和**：降低锐利感；
- **阅读**：面向长文本和较长使用时间；
- **代码**：强调英文、数字与细节分辨；
- **圆润观感**：偏向更饱满、亲和的 UI 效果。

推荐只是起点。最终应在常用显示器、缩放比例和软件中检查中文、英文、数字、Emoji、粗体、细体和小字号。

## 🧠 Windows 字体替换到底改了什么？

### 1. 字体文件

字体必须安装到 Windows 可识别的位置，并使用内部 Family Name 注册。文件存在不等于系统已经识别。

### 2. FontSubstitutes

把常见系统 UI 字体族映射到目标字体。不同 Windows 版本和应用可能使用 `Segoe UI`、Variable 系列或中文 UI 字体。

### 3. FontLink

主字体不包含某个字符时，Windows 需要回退链寻找 Emoji、中文或其它字形。错误回退会导致方框、错字形和粗细跳变。

### 4. Desktop / Avalon.Graphics

桌面字体平滑与 WPF 渲染参数影响边缘、对比和 Gamma。不同渲染技术不会百分之百使用同一套设置。

### 5. 应用刷新

程序广播系统设置变化，但部分应用只在启动、登录或系统重启时重新读取字体。没有立即变化不等于写入失败。

<a id="download"></a>

## 📥 下载与安装

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

## 🚀 3 分钟完成第一次调音

1. 启动后先查看当前字体、DPI 和备份状态；
2. 选择 HarmonyOS Sans SC 或另一套已就绪预设；
3. 先选“平衡”渲染风格；
4. 在 A/B 预览中检查中文、英文、数字和小字号；
5. 阅读应用摘要，确认要安装和映射的字体；
6. 点击应用并确认备份成功；
7. 完全退出并重开资源管理器/浏览器等常用应用；
8. 不满意就恢复上一次备份，不要连续叠加多个方案。

> [!IMPORTANT]
> 第一次应用后先观察一段时间。不要为了比较方案连续快速写入六次注册表；每次切换都应知道当前备份对应哪个状态。

<a id="safety"></a>

## 🛡️ 安全与恢复

### 💾 自动备份

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

### ↩️ 恢复方式

- **恢复上一次备份**：恢复最近一次写入前的注册表状态；
- **按组件恢复**：分别恢复字体映射、FontLink 回退链或渲染参数；
- **恢复 Windows 默认**：重置本工具管理的映射与渲染设置；
- **修复系统字体文件**：依次运行 DISM 和 SFC；
- **Shift 急救模式**：界面字体已经看不清时，无界面恢复本工具记录的设置。

### 🚑 Shift 急救模式

如果应用字体后系统文字严重异常：

1. 按住 `Shift`；
2. 双击启动 WindowsFontTuner；
3. 程序跳过常规界面，执行无头恢复；
4. 等待刷新完成；
5. 如仍异常，注销 Windows 或重启；
6. 必要时以管理员终端运行 DISM 和 SFC。

急救模式只能恢复本工具记录和管理的范围，不能保证修复其它软件或手工注册表修改造成的问题。

### 出问题时的恢复顺序

1. 先关闭刚才未刷新的应用并重新打开；
2. 使用“恢复上一次备份”；
3. 注销当前 Windows 用户并重新登录；
4. 仍异常时用“恢复 Windows 默认”；
5. UI 无法操作时使用 Shift 急救；
6. 字体文件损坏时再运行 DISM 和 SFC；
7. 其它工具或手工修改造成的问题需用对应备份恢复。

## 🔄 应用后没有变化怎么办？

Windows 不同应用使用 GDI、DirectWrite、WPF、Chromium 或自有渲染器，刷新时机不同。可以依次尝试：

1. 完全退出并重开目标应用；
2. 重启资源管理器；
3. 注销当前 Windows 用户；
4. 重启系统；
5. 检查应用是否硬编码自己的字体；
6. 检查字体文件是否安装完整、名称是否匹配；
7. 使用 A/B 预览确认不是缩放或渲染差异。

部分应用不会遵循系统字体替换，这是应用自身渲染策略，不应通过无限扩大注册表替换范围强行解决。

## 🧭 常见使用场景

### 新电脑想统一中文观感

先选择稳定预设和“平衡”，检查高频应用；确认无缺字后再尝试更个性化的渲染风格。

### 高分屏觉得系统文字太细

不要只换更粗字体。先核对 DPI 和渲染风格，比较“清晰”“阅读”和“平衡”的边缘与对比。

### 代码编辑器英文好看，系统中文不好看

可以尝试 Inter + HarmonyOS 组合，或只在编辑器内部设置代码字体。系统全局替换不一定是解决单个应用的最佳方法。

### Emoji 或生僻字变方框

优先检查 FontLink 和目标字体覆盖范围，不要盲目重复安装同一个字体文件。

### 想用未打包的字体

确认许可和来源后通过自定义导入。不要把只能个人使用或禁止再分发的字体提交到仓库或公开方案包。

## 📥 导入自定义字体

支持导入 TTF、OTF 和 OTC 文件。导入前请确认：

- 字体文件来自可信来源；
- 许可允许你安装和使用；
- 字体包含所需中文、英文和符号字形；
- 字体内部 Family Name 与文件名可能不同；
- 不把未知字体作为唯一系统字体；
- 保留至少一套可靠回退字体和最近备份。

OPPOSans 因分发条件不适合直接打入公开 Release，需要用户自行取得合法字体文件后导入。

> [!CAUTION]
> 来历不明的字体文件可能损坏、命名异常或包含不完整字形。不要从未知网盘下载所谓“系统字体整合包”，更不要删除 Windows 自带字体来强制生效。

## 🏗️ 它是怎么工作的？

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

### React / TypeScript

负责预设选择、当前状态、A/B 预览、应用摘要、恢复入口和设置。前端只提交方案 ID、渲染风格 ID 和用户选择的文件路径。

### Tauri 命令边界

暴露 `load_bootstrap`、`get_apply_summary`、`apply_preset`、导入字体、恢复、方案导入导出和系统字体修复等结构化操作。

### Rust 系统层

负责显示器/DPI 读取、字体解析、文件下载、用户字体安装、注册表导出/导入、回退链、系统广播和无头恢复。

### 预设数据

`preset_data.rs` 集中声明字体族、所需文件、下载 URL、安装方式、替换映射和回退列表，避免 UI 与底层各维护一份不同清单。

<a id="development"></a>

## 🛠️ 从源码开发

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

## ✅ 变更验证

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

## 📁 项目结构

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

<a id="faq"></a>

## ❓ 常见问题

### 修改字体会影响所有 Windows 用户吗？

项目同时涉及当前用户和部分系统范围设置，具体取决于字体安装和注册表项。应用摘要与 UAC 提示应作为判断依据。

### 为什么有的应用变了，有的没变？

不同应用使用 GDI、DirectWrite、WPF、Chromium 或自有字体，读取系统映射的范围和刷新时机不同。

### 应用后必须重启吗？

先重开目标应用和资源管理器；部分场景需要注销或重启才能让所有进程重新读取设置。

### 会删除 Windows 原生字体吗？

不会以删除微软字体作为替换手段。恢复默认也主要恢复映射和渲染；系统文件损坏交给 DISM/SFC。

### 为什么不能直接打包 Segoe UI？

它是 Windows 随附字体，重新分发受微软条款限制。项目只引用系统已有字体，不把文件放进 Release。

### 为什么 OPPOSans 需要自己导入？

项目不在没有明确再分发许可的情况下替用户重新发布字体文件。

### MiSans 为什么没有内置？

其授权包含再分发限制，不适合默认打入公开 Release。

### Shift 急救什么时候用？

常规 UI 已经难以阅读或无法可靠操作时使用。正常情况下优先在界面中恢复上一次备份。

### 备份可以复制到其它电脑吗？

注册表备份反映原设备状态，不应作为跨设备字体方案。跨设备使用导出方案，并在目标机重新检查字体和 DPI。

### 自定义字体导入后出现方框怎么办？

检查字体是否覆盖中文/符号，并确认 FontLink 回退。单个拉丁字体不能承担完整 Windows UI 字符集。

### 支持 Windows 7/8 吗？

当前目标是 Windows 10/11，Tauri、WebView2、字体和 Variable 系列逻辑都按现代 Windows 设计。

### MIT 允许别人做商业版本吗？

允许。MIT 允许使用、修改、分发、再许可和销售，但必须保留版权与许可声明；字体和品牌不一定随代码授权。

## 🤝 参与项目

欢迎提交：

- 特定 Windows 版本、DPI 和多显示器组合问题；
- 字体 Family Name、字重和回退链修正；
- 可合法再分发字体的来源/许可更新；
- 应用刷新、备份和恢复可靠性改进；
- 不删除系统字体的安全方案；
- 主线 Tauri UI 的可访问性和窄窗口问题。

新增字体预设必须同时提交来源、版本、许可、文件哈希/清单、所需字重、替换映射、回退链和真实应用验证结果。

## 📚 更新记录

- **v2.2.4**：当前正式版，Tauri 2 主线持续完善预设、视觉和系统应用流程；
- **v2.2.x**：集中迭代高 DPI、窗口适配、字体方案与安装/便携发布；
- **v0.5.x**：历史 C# / WinForms 版本，保留作实现参考，不再作为主线界面开发。

正式版本与安装文件以 [Releases](https://github.com/soberbw-hash/WindowsFontTuner/releases) 为准。

<a id="license"></a>

## ⚖️ 许可与字体版权

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
