# WindowsFontTuner 2.0

把 Windows 字体这件事，从“注册表体操”做成一个真正能用、能回退、能长期放在桌面的工具。

`WindowsFontTuner 2.0` 现在是一套重新做过的 `Rust + Tauri 2 + React` 桌面应用：

- 更像一个极简的视觉调音台，不像一堆危险开关
- 界面无边框、圆角、云母材质，默认就是沉浸式窗口
- 选风格、点应用，剩下的提权、备份、写注册表、广播刷新都交给底层
- 保留“后悔药”，也保留“出事时还能盲救”的安全感
- 默认同时提供安装版和绿色便携版，适合“装上就用”也适合“用完即走”

## 现在这版能做什么

- 一键应用预设字体风格
- 启动时自动识别当前系统正在使用的风格
- 按屏幕分辨率和 DPI 缩放下发不同渲染矩阵
- 自动备份 `FontSubstitutes`、`FontLink`、`Desktop`、`Avalon.Graphics`
- 一键恢复 Windows 原生设定
- 一键拉起 `DISM /RestoreHealth` 和 `sfc /scannow`
- 支持自定义导入字体文件
- 支持自动下载并安装可公开分发的字体资源
- 自动补上 `Segoe UI Emoji` 和 `Microsoft YaHei` 的 FontLink 回退链，降低 Emoji / 生僻字翻车概率

## 当前预设

- `HarmonyOS Sans SC`
  现代、干净、最稳妥的长期默认方案

- `更纱黑体`
  笔画更扎实，适合代码、文档、资源管理器

- `思源黑体 CN`
  中性、克制、跨平台一致性最好

- `霞鹜文楷`
  更温润，更有书卷气，适合写作和阅读

- `OPPOSans`
  需要先导入字体文件，再一键应用

- `Inter + HarmonyOS`
  把英文字母和数字的节奏拉得更像设计工具，中文保持稳定

## 安全感设计

- 每次真正写入前都会静默备份
- UI 右下角藏了一个很低调的急救箱菜单
- 支持“恢复 Windows 原生设定”
- 支持“修复系统字体文件”
- 支持隐藏的 `Shift` 急救模式

### Shift 急救模式

如果你真的把系统字体搞到看不清了：

1. 按住 `Shift`
2. 再双击启动 `WindowsFontTuner 2.0`

程序会跳过界面，直接在后台静默恢复它写过的字体映射，并尝试刷新资源管理器。

## 技术栈

- `Rust`
- `Tauri 2`
- `React`
- `TypeScript`
- `framer-motion`

## 目录说明

- [WindowsFontTuner2](./WindowsFontTuner2)
  当前主线项目，2.0 Tauri 版

- 根目录里的 `C# / WinForms` 文件
  旧版实现，保留作历史参考，不再作为主线 UI 继续迭代

## 本地开发

```powershell
cd .\WindowsFontTuner2
cmd /c npm install
cmd /c npm run tauri dev
```

## 构建发布

```powershell
cd .\WindowsFontTuner2
cmd /c npm run tauri build
```

默认会产出 `NSIS` 安装包。  
便携 `zip` 也会保留，因为这个项目本身就强调“绿色、用完即走”。

## 支持这个项目

这个工具如果真的让你的 Windows 看起来顺眼了，项目里也藏了一个很克制的支持入口。  
不打扰，但永远欢迎一杯咖啡。
