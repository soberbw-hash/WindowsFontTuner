# Windows全局字体替换器

`Windows全局字体替换器` 是一个面向普通用户的小工具，用来备份、应用和恢复 Windows 全局字体替换预设。

这一版开始，软件里直接附带了三套可安装字体包，下载后不需要再自己上网找字体：

- `HarmonyOS Sans SC`
- `Source Han Sans CN`
- `Sarasa UI SC`

你可以先安装字体，再一键切换预设。

## 适合做什么

- 备份 `FontSubstitutes`、`Desktop`、`Avalon.Graphics`、`WindowMetrics`
- 把 `Segoe UI` / `Segoe UI Variable` 替换成更顺眼的中文黑体
- 应用更接近灰阶抗锯齿的文字渲染方案
- 调整桌面图标、菜单和传统窗口字体
- 重建字体缓存并重启资源管理器

## 当前内置的三个选项

### 1. HarmonyOS 统一预设

- 风格最接近这台机器当前调好的效果
- 现代、干净、整体感强
- 适合想继续保持 HarmonyOS 风格的人

### 2. 思源黑体 CN 统一预设

- 最稳妥、最中性
- 跨机器一致性最好
- 适合想要稳定、标准、耐看的系统字体风格的人

### 3. 更纱 UI SC 统一预设

- 三套里最有存在感
- 笔画更扎实，内容区更容易看清
- 适合觉得 Windows 默认太细的人

## 软件现在能做什么

- 从 `Presets` 文件夹加载多个预设
- 从 `FontPackages` 文件夹读取内置字体包
- 检查预设需要的字体是否已经安装
- 一键安装当前预设所需字体
- 自动把相关注册表导出到 `%LOCALAPPDATA%\\WindowsFontTuner\\Backups`
- 一键应用字体替换、渲染参数和窗口字体设置
- 一键恢复最近一次备份

## 软件不能做什么

- 不保证 Windows 11 的每一个界面区域都会完全遵循同一套字体设置
- 不会自动卸载已经安装到系统里的字体文件
- 不修改 Explorer 私有资源或 XAML 私有样式

## 为什么没有 MiSans

我没有把 `MiSans` 打进发布包。

原因很简单：它的官方授权明确写了，不能单独再分发字体软件或其副本，所以不适合直接塞进公开 Release 里。

## 字体授权

软件内置字体包时，都会一起附带原始授权文件和来源说明：

- `FontPackages\\harmonyos-sc\\LICENSE.txt`
- `FontPackages\\source-han-sans-cn\\LICENSE.txt`
- `FontPackages\\sarasa-ui-sc\\LICENSE.txt`

对应来源：

- [HarmonyOS Sans 官方仓库](https://github.com/huawei-fonts/HarmonyOS-Sans)
- [Source Han Sans 官方仓库](https://github.com/adobe-fonts/source-han-sans)
- [Sarasa Gothic 官方仓库](https://github.com/be5invis/Sarasa-Gothic)

## 使用方法

1. 右键运行 `WindowsFontTuner.exe`，选择“以管理员身份运行”。
2. 在下拉框里选择一个预设。
3. 先点“安装预设字体”。
4. 再点“应用当前预设”。
5. 如果不满意，可以点“恢复最近备份”。

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

## 自定义预设

预设就是 `Presets` 目录里的 JSON 文件。

字段说明：

- `Name`：界面里显示的预设名称
- `Description`：预设说明
- `FontPackageId`：当前预设对应的内置字体包 ID
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

## 说明

- Windows 11 的字体显示由多套机制共同决定，不同区域对这些设置的响应程度可能不同。
- 程序在应用预设之前，会先自动导出一份时间戳备份，方便你随时回退。
- 恢复最近备份主要回退字体映射和渲染设置，不会自动删除已经安装到系统里的字体文件。
