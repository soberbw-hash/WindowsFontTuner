# Windows全局字体替换器

> 如果你也受够了微软那套又细、又灰、又发虚，看久了像在拿眼睛上刑的默认字体，这个工具就是给你准备的。  
> 一键换字体，一键备份，一键恢复，尽量少折腾系统，尽量多提升观感。

`Windows全局字体替换器` 是一个给普通用户准备的 Windows 字体调校工具。  
它的目标很直接：把系统字体从“能用”拉到“顺眼”，而且别把电脑搞炸。

## ✨ 这玩意能干嘛

- 一键替换 `Segoe UI` / `Segoe UI Variable` 相关系统字体映射
- 一键调整桌面图标、菜单、传统窗口字体
- 一键应用更舒服的文字渲染参数
- 自动备份 `FontSubstitutes`、`Desktop`、`Avalon.Graphics`、`WindowMetrics`
- 一键恢复最近一次备份，翻车了也能撤回
- 一键恢复 `Windows 默认` 字体映射和渲染参数
- 一键启动 `DISM + SFC` 官方修复流程，处理系统字体文件缺失
- 内置字体包，不用再自己满网找字体
- 支持检查更新，后续发新版本软件里能直接提示

## 📦 现在有两种下载方式

- `Setup.exe` 安装版：适合小白，双击安装，自动创建桌面和开始菜单快捷方式
- `zip` 便携版：适合喜欢自己解压、自己掌控目录的人

## 🔤 内置三套字体方案

- `HarmonyOS Sans SC`
  风格现代、干净，整体观感最接近这台机器现在调好的效果。

- `Source Han Sans CN`
  最稳、最中性，跨机器一致性最好，适合长期默认使用。

- `Sarasa UI SC`
  笔画更扎实，更有存在感，适合嫌 Windows 默认太细的人。

## 🎯 适合谁

- 看不惯 Windows 默认字体那股“又细又虚”的味道
- 想把桌面、资源管理器、传统界面整体调顺眼一点
- 不想手动翻注册表，但又想保留可回退能力
- 想直接给朋友或网友发一个能用的字体替换工具

## 🚀 怎么用

1. 下载你想要的版本。
2. 如果你下的是 `Setup.exe`，直接双击安装。
3. 如果你下的是 `zip`，解压后右键运行 `WindowsFontTuner.exe`，选择“以管理员身份运行”。
4. 在下拉框里选一个预设。
5. 点击“安装所需字体”。
6. 点击“应用当前预设”。
7. 如果效果不满意，点击“恢复最近备份”。
8. 如果想回到微软原生方案，点击“恢复 Windows 默认”。
9. 如果系统自带字体文件被误删，再点“修复系统字体”。

## 🧠 软件里实际做了什么

- 从 `Presets` 目录读取多个字体预设
- 从 `FontPackages` 目录读取内置字体包
- 检查当前预设所需字体是否已经安装
- 自动安装当前预设所需字体
- 自动导出一份注册表备份到 `%LOCALAPPDATA%\\WindowsFontTuner\\Backups`
- 写入字体替换、渲染参数和窗口字体设置
- 支持清除本工具写入过的字体映射，恢复 Windows 默认状态
- 支持调用微软官方的 `DISM /RestoreHealth` 和 `sfc /scannow`
- 重建字体缓存并按需重启资源管理器

## ⚠️ 先把边界说清楚

- Windows 11 的字体显示不是一条线说了算，不同区域响应程度可能不一样
- 这个工具不保证系统里每一个角落都会 100% 跟着同一套字体走
- 它不会自动卸载已经安装到系统里的字体文件
- 它不会去改 Explorer 的私有资源，也不会碰 WinUI / XAML 私有样式
- 公开 Release 不会附带 `Segoe UI`、`Segoe UI Variable` 这类 Windows 自带字体文件

为什么不把微软默认字体一起打包？

- 微软官方字体 FAQ 明确写了：除文档嵌入等特殊情况外，`Windows 自带字体不能被重新分发`
- `Segoe UI` 这类字体在官方列表里也标的是 `Download N/A`，只随微软产品提供
- 所以这个项目里“恢复默认”的做法是：恢复注册表和渲染参数；如果系统字体文件本身坏了或被删了，再调用 Windows 官方修复

## ❓为什么没有 MiSans

不是我不想放，是它的授权不适合直接塞进公开 Release。

简单说就是：

- `MiSans` 官方授权明确限制再次分发字体软件或其副本
- 所以它不适合被直接打进一个公开下载的工具里
- 现在内置的三套字体，都是更适合公开分发的方案

## 📝 字体授权与来源

软件附带字体包时，都会一起带上原始授权文件和来源说明：

- `FontPackages\\harmonyos-sc\\LICENSE.txt`
- `FontPackages\\source-han-sans-cn\\LICENSE.txt`
- `FontPackages\\sarasa-ui-sc\\LICENSE.txt`

对应来源：

- [HarmonyOS Sans 官方仓库](https://github.com/huawei-fonts/HarmonyOS-Sans)
- [Source Han Sans 官方仓库](https://github.com/adobe-fonts/source-han-sans)
- [Sarasa Gothic 官方仓库](https://github.com/be5invis/Sarasa-Gothic)
- [Microsoft Font redistribution FAQ](https://learn.microsoft.com/en-my/typography/fonts/font-faq)
- [Segoe UI font family](https://learn.microsoft.com/en-us/typography/font-list/segoe-ui)

## 🛠️ 自己构建

项目目标框架是 `.NET Framework 4.8`，使用 Windows 自带环境里的 `MSBuild`。

普通构建：

```bat
build.bat
```

生成位置：

```text
bin\Release\WindowsFontTuner.exe
```

如果你想一次性打出 `zip` 便携包和 `Setup.exe` 安装包：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\Build-Packages.ps1 -Version 0.5.2
```

## 🧩 自定义预设

预设就是 `Presets` 目录里的 JSON 文件。

主要字段说明：

- `Name`：界面里显示的预设名
- `Description`：预设说明
- `FontPackageId`：当前预设对应的内置字体包 ID
- `RequiredFonts`：这个预设依赖的已安装字体
- `FontSubstitutes`：写入 `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\FontSubstitutes` 的字体替换项
- `DesktopTextSettings`：写入 `HKCU\Control Panel\Desktop` 的字体平滑设置
- `Rendering`：写入 `HKCU\Software\Microsoft\Avalon.Graphics\DISPLAY*` 的渲染参数
- `WindowMetrics`：通过 `SystemParametersInfo` 应用的窗口、菜单、图标字体参数

## 🚢 发布到 GitHub

这个目录已经可以直接当作 Git 仓库使用。

如果你已经有一个空仓库：

```bat
git remote add origin https://github.com/<your-name>/WindowsFontTuner.git
git push -u origin main
```

如果你已经配置好了 GitHub 凭据，可以直接用内置脚本更新 Release：

```powershell
$assets = @(
  '.\dist\WindowsFontTuner-v0.5.2-win64.zip',
  '.\dist\WindowsFontTuner-Setup-v0.5.2.exe'
)

powershell -ExecutionPolicy Bypass -File .\scripts\Publish-Release.ps1 `
  -Version 0.5.2 `
  -ReleaseName 'Windows全局字体替换器 v0.5.2' `
  -AssetPaths $assets
```

## 🪄 最后一句

这个工具不是为了把 Windows 变成 macOS。  
它只是很认真地解决一个很多人都在忍、但懒得自己折腾的问题：

**让 Windows 字体，终于看起来像是给人看的。**
