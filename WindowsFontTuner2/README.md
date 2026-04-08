# WindowsFontTuner 2.0 App

这是 `WindowsFontTuner 2.0` 的主工程目录。

## 技术栈

- Rust
- Tauri 2
- React
- TypeScript
- framer-motion

## 启动开发环境

```powershell
cmd /c npm install
cmd /c npm run tauri dev
```

## 生产构建

```powershell
cmd /c npm run tauri build
```

## 这版重点

- 无边框窗口 + Mica 质感
- 单卡片轮播，不再堆下拉框
- 自动 DPI 嗅探和渲染矩阵分发
- FontLink 回退链兜底
- Shift 急救模式
- 自定义导入字体
- 应用成功后的低打扰支持入口
- 默认发布 `NSIS` 安装版，同时保留绿色便携包
