# Windows字体调谐器

`Windows字体调谐器` 是一个面向普通用户的小工具，用来备份、应用和恢复 Windows 字体调谐预设。

它适合这类场景：

- 备份 `FontSubstitutes`、`Desktop`、`Avalon.Graphics`、`WindowMetrics`
- 把 `Segoe UI` / `Segoe UI Variable` 替换成你喜欢的已安装字体
- 应用更接近灰阶抗锯齿的文字渲染方案
- 调整桌面图标、菜单和传统窗口字体
- 重建字体缓存并重启资源管理器

## 这个工具能做什么

- 从 `Presets` 文件夹加载预设
- 检查预设需要的字体是否已安装
- 自动把相关注册表导出到 `%LOCALAPPDATA%\\WindowsFontTuner\\Backups`
- 一键应用字体替换、渲染参数和窗口字体设置
- 一键恢复最近一次备份

## 这个工具不能做什么

- 不内置第三方字体文件
- 不修改 Explorer 私有资源或 XAML 私有样式
- 不保证 Windows 11 的每一个界面区域都会完全遵循同一套字体设置

## 当前内置预设

目前附带的预设是：

- `HarmonyOS 统一预设`

需要你提前安装这些字体：

- `HarmonyOS Sans SC`
- `HarmonyOS Sans SC Medium`

官方来源：

- [HarmonyOS Sans 官方仓库](https://github.com/huawei-fonts/HarmonyOS-Sans)

## 构建方法

本项目目标框架是 `.NET Framework 4.8`，使用 Windows 自带环境里的 `MSBuild`。

运行：

```bat
build.bat
```

生成文件位置：

```text
bin\Release\WindowsFontTuner.exe
```

## 使用方法

请以管理员身份运行程序。

因为写入 `HKLM\...\FontSubstitutes`、重建字体缓存、刷新资源管理器都需要管理员权限，所以程序清单里已经请求提升权限。

## 自定义预设

预设就是 `Presets` 目录里的 JSON 文件。

字段说明：

- `Name`：界面里显示的预设名称
- `Description`：预设说明
- `RequiredFonts`：这个预设依赖的已安装字体
- `FontSubstitutes`：写入 `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\FontSubstitutes` 的字体替换项
- `DesktopTextSettings`：写入 `HKCU\Control Panel\Desktop` 的字体平滑设置
- `Rendering`：写入 `HKCU\Software\Microsoft\Avalon.Graphics\DISPLAY*` 的渲染参数
- `WindowMetrics`：通过 `SystemParametersInfo` 应用的窗口、菜单、图标字体参数

## 发布到 GitHub

这个目录已经可以直接当作 Git 仓库使用。

如果你已经有一个空仓库，可以运行：

```bat
git remote add origin https://github.com/<your-name>/WindowsFontTuner.git
git push -u origin main
```

如果你要发布可下载版本，建议把 `bin\Release\` 里的可执行文件和必要资源一起打包，再作为 GitHub Release 附件上传。

## 说明

- Windows 11 的字体显示由多套机制共同决定，不同区域对这些设置的响应程度可能不同。
- 程序在应用预设之前，会先自动导出一份时间戳备份，方便你随时回退。
