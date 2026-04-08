import { getCurrentWindow } from "@tauri-apps/api/window";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { AnimatePresence, motion } from "framer-motion";
import {
  AlertTriangle,
  ArrowLeft,
  ArrowRight,
  Check,
  CheckCheck,
  ChevronDown,
  Coffee,
  ExternalLink,
  FileDown,
  FileUp,
  FolderUp,
  LoaderCircle,
  Minimize2,
  MonitorSmartphone,
  RefreshCw,
  RotateCcw,
  ScanSearch,
  Settings2,
  ShieldCheck,
  Sparkles,
  Square,
  TerminalSquare,
  Wrench,
  X,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import type { CSSProperties, ReactNode } from "react";
import appIcon from "./assets/app-icon.png";
import supportQr from "./assets/support-qr.png";
import { presetCatalog, renderStyleCatalog } from "./lib/presets";
import {
  applyPreset,
  exportCurrentScheme,
  getApplySummary,
  importFontFiles,
  importSharedScheme,
  loadBootstrap,
  repairSystemFonts,
  restoreWindowsDefault,
  runRecoveryAction,
} from "./lib/tauri";
import type {
  ApplySummary,
  BootstrapPayload,
  CheckStatus,
  DisplayPreset,
  HealthItem,
  ImportedPresetPayload,
  PreviewMode,
  PreviewScale,
  PreviewTheme,
  RenderStyleId,
  RiskLevel,
} from "./types";

const CANVAS_WIDTH = 1440;
const DEFAULT_CANVAS_HEIGHT = 760;
const SYSTEM_PREVIEW_FONT = '"Segoe UI Variable", "Microsoft YaHei UI", sans-serif';
const previewScales: PreviewScale[] = [100, 125, 150, 200];

type ToastTone = "success" | "warning" | "info";

interface ToastState {
  tone: ToastTone;
  message: string;
}

interface KeepBannerState {
  secondsLeft: number;
}

interface PreviewScenario {
  eyebrow: string;
  title: string;
  body: string;
  sub: string;
}

function App() {
  const [bootstrap, setBootstrap] = useState<BootstrapPayload | null>(null);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [direction, setDirection] = useState(0);
  const [selectedRenderStyleId, setSelectedRenderStyleId] = useState<RenderStyleId>("balanced");
  const [previewMode, setPreviewMode] = useState<PreviewMode>("ui");
  const [previewTheme, setPreviewTheme] = useState<PreviewTheme>("light");
  const [previewScale, setPreviewScale] = useState<PreviewScale>(125);
  const [canvasScale, setCanvasScale] = useState(1);
  const [summary, setSummary] = useState<ApplySummary | null>(null);
  const [summaryOpen, setSummaryOpen] = useState(false);
  const [rescueOpen, setRescueOpen] = useState(false);
  const [supportOpen, setSupportOpen] = useState(false);
  const [supportNudge, setSupportNudge] = useState(false);
  const [expertMode, setExpertMode] = useState(false);
  const [isBusy, setIsBusy] = useState(false);
  const [isSliding, setIsSliding] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [keepBanner, setKeepBanner] = useState<KeepBannerState | null>(null);
  const [importedPreset, setImportedPreset] = useState<ImportedPresetPayload | null>(null);
  const stageRef = useRef<HTMLDivElement | null>(null);
  const canvasRef = useRef<HTMLDivElement | null>(null);
  const slideTimerRef = useRef<number | null>(null);

  useEffect(() => {
    void refresh(true);
  }, []);

  useEffect(() => {
    const stage = stageRef.current;
    const canvas = canvasRef.current;
    if (!stage || !canvas) {
      return;
    }

    let frameId: number | null = null;
    const updateScale = () => {
      const availableWidth = Math.max(stage.clientWidth - 16, 1);
      const availableHeight = Math.max(stage.clientHeight - 16, 1);
      const contentWidth = Math.max(canvas.scrollWidth, canvas.offsetWidth, CANVAS_WIDTH);
      const contentHeight = Math.max(canvas.scrollHeight, canvas.offsetHeight, DEFAULT_CANVAS_HEIGHT);
      const next = Math.min(availableWidth / contentWidth, availableHeight / contentHeight, 1);
      setCanvasScale(next);
    };

    const scheduleScale = () => {
      if (frameId !== null) {
        window.cancelAnimationFrame(frameId);
      }
      frameId = window.requestAnimationFrame(() => {
        frameId = null;
        updateScale();
      });
    };

    scheduleScale();
    const observer = new ResizeObserver(scheduleScale);
    observer.observe(stage);
    observer.observe(canvas);
    window.addEventListener("resize", scheduleScale);

    return () => {
      observer.disconnect();
      window.removeEventListener("resize", scheduleScale);
      if (frameId !== null) {
        window.cancelAnimationFrame(frameId);
      }
    };
  }, []);

  useEffect(() => {
    return () => {
      if (slideTimerRef.current !== null) {
        window.clearTimeout(slideTimerRef.current);
      }
    };
  }, []);

  useEffect(() => {
    if (!toast) {
      return;
    }

    const timer = window.setTimeout(() => setToast(null), 3200);
    return () => window.clearTimeout(timer);
  }, [toast]);

  useEffect(() => {
    if (!keepBanner) {
      return;
    }

    const timer = window.setInterval(() => {
      setKeepBanner((current) => {
        if (!current) {
          return null;
        }
        if (current.secondsLeft <= 1) {
          window.clearInterval(timer);
          return null;
        }
        return { secondsLeft: current.secondsLeft - 1 };
      });
    }, 1000);

    return () => window.clearInterval(timer);
  }, [keepBanner]);

  const presets = useMemo<DisplayPreset[]>(() => {
    const runtime = new Map((bootstrap?.presets ?? []).map((item) => [item.id, item]));
    return presetCatalog.map((preset) => {
      const state = runtime.get(preset.id);
      return {
        ...preset,
        status: state?.status ?? "ready",
        installMode: state?.installMode ?? "manualImport",
        available: state?.available ?? false,
        current: state?.current ?? false,
      };
    });
  }, [bootstrap]);

  const selectedPreset = presets[currentIndex] ?? presets[0] ?? null;

  const renderStyles = useMemo(
    () =>
      renderStyleCatalog.map((style) => {
        const runtime = bootstrap?.renderStyles.find((item) => item.id === style.id);
        return {
          ...style,
          current: runtime?.current ?? bootstrap?.activeRenderStyleId === style.id,
          recommended: runtime?.recommended ?? bootstrap?.recommendation.primaryRenderStyle === style.id,
        };
      }),
    [bootstrap],
  );

  const selectedRenderStyle =
    renderStyles.find((style) => style.id === selectedRenderStyleId) ??
    renderStyles[0] ??
    null;

  const recommendedPreset =
    presets.find((preset) => preset.id === bootstrap?.recommendation.primaryPresetId) ??
    selectedPreset;

  const recommendedRenderStyle =
    renderStyles.find((style) => style.id === bootstrap?.recommendation.primaryRenderStyle) ??
    selectedRenderStyle;

  const canGoPrev = currentIndex > 0;
  const canGoNext = currentIndex < presets.length - 1;
  const canSlide = !isSliding && !isBusy;
  const isCurrentMatch = Boolean(selectedPreset?.current && bootstrap?.activeRenderStyleId === selectedRenderStyleId);

  const scenario = useMemo(
    () => buildPreviewScenario(selectedPreset, previewMode),
    [previewMode, selectedPreset],
  );

  const healthItems = summary?.health.items.length ? summary.health.items : fallbackHealthItems(selectedPreset);
  const healthSummary =
    summary?.health.summary ??
    "应用前会自动检查字体可用性、Emoji 回退链和长期默认风险。";

  const primaryActionLabel = isCurrentMatch
    ? "当前正在使用"
    : !selectedPreset
      ? "正在读取方案"
      : !selectedPreset.available && selectedPreset.installMode === "manualImport"
        ? "导入本地字体"
        : !selectedPreset.available && selectedPreset.installMode === "autoDownload"
          ? "下载并应用"
          : "应用此风格";

  const statusTitle = isCurrentMatch
    ? "当前正在使用"
    : selectedPreset?.available
      ? "可以直接应用"
      : selectedPreset?.installMode === "autoDownload"
        ? "点击后自动准备"
        : "等字体包就位";

  const currentRenderStyleLabel =
    renderStyles.find((item) => item.id === bootstrap?.activeRenderStyleId)?.label ?? "平衡";

  const canvasStyle = {
    width: `${CANVAS_WIDTH}px`,
    transform: `scale(${canvasScale})`,
  } satisfies CSSProperties;

  useEffect(() => {
    if (!bootstrap || !selectedPreset) {
      return;
    }

    let cancelled = false;
    void getApplySummary(selectedPreset.id, selectedRenderStyleId)
      .then((payload) => {
        if (!cancelled) {
          setSummary(payload);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setSummary(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [bootstrap, selectedPreset, selectedRenderStyleId]);

  async function refresh(forceSelection = false) {
    try {
      const next = await loadBootstrap();
      setBootstrap(next);

      const targetPresetId = forceSelection
        ? next.activePresetId ?? next.recommendation.primaryPresetId ?? presetCatalog[0]?.id
        : presetCatalog[currentIndex]?.id ?? next.activePresetId ?? next.recommendation.primaryPresetId;

      const nextIndex = presetCatalog.findIndex((item) => item.id === targetPresetId);
      setCurrentIndex(nextIndex >= 0 ? nextIndex : 0);

      setSelectedRenderStyleId((current) => {
        if (!forceSelection && renderStyleCatalog.some((item) => item.id === current)) {
          return current;
        }
        return next.activeRenderStyleId ?? next.recommendation.primaryRenderStyle;
      });
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "启动时没能顺利读取当前系统状态。"),
      });
    }
  }

  async function handleOpenApplySummary() {
    if (!selectedPreset) {
      return;
    }

    try {
      const payload = await getApplySummary(selectedPreset.id, selectedRenderStyleId);
      setSummary(payload);
      setSummaryOpen(true);
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "没能生成这次应用的变更摘要。"),
      });
    }
  }

  async function handleApplyConfirmed() {
    if (!selectedPreset) {
      return;
    }

    setIsBusy(true);
    setSupportNudge(false);
    setSummaryOpen(false);

    try {
      await applyPreset(selectedPreset.id, selectedRenderStyleId);
      await refresh(true);
      setToast({
        tone: "success",
        message: "✨ 风格已应用，注销电脑后即可感受。",
      });
      setKeepBanner({ secondsLeft: 15 });
      window.setTimeout(() => setSupportNudge(true), 1500);
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "应用这套风格时出了点问题。"),
      });
    } finally {
      setIsBusy(false);
    }
  }

  async function handleRollbackLast() {
    setIsBusy(true);
    try {
      await runRecoveryAction("rollbackLast");
      await refresh(true);
      setKeepBanner(null);
      setToast({
        tone: "info",
        message: "已经回滚到这次修改前的状态。",
      });
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "回滚这次修改时没有成功。"),
      });
    } finally {
      setIsBusy(false);
    }
  }

  async function handleImportFonts() {
    const picked = await openFileDialog({
      multiple: true,
      filters: [{ name: "字体文件", extensions: ["ttf", "otf", "ttc", "otc"] }],
    });

    if (!picked) {
      return;
    }

    const paths = Array.isArray(picked) ? picked : [picked];
    if (!paths.length) {
      return;
    }

    setIsBusy(true);
    try {
      await importFontFiles(paths);
      await refresh();
      setToast({
        tone: "success",
        message: `已导入 ${paths.length} 个字体文件，现在可以继续应用喜欢的方案了。`,
      });
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "导入字体时出了点问题。"),
      });
    } finally {
      setIsBusy(false);
    }
  }

  async function handleImportScheme() {
    const picked = await openFileDialog({
      multiple: false,
      filters: [{ name: "方案文件", extensions: ["json", "wftpreset"] }],
    });

    if (!picked || Array.isArray(picked)) {
      return;
    }

    try {
      const payload = await importSharedScheme(picked);
      const presetIndex = presets.findIndex((item) => item.fontFamily === payload.cnFont);
      if (presetIndex >= 0) {
        setCurrentIndex(presetIndex);
      }
      setSelectedRenderStyleId(payload.renderStyleId);
      setImportedPreset(payload);
      setToast({
        tone: payload.warnings.length ? "warning" : "success",
        message: payload.warnings.length
          ? `已导入方案：${payload.name}。${payload.warnings[0]}`
          : `已导入方案：${payload.name}。`,
      });
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "导入分享方案时没有成功。"),
      });
    }
  }

  async function handleExportScheme() {
    if (!selectedPreset) {
      return;
    }

    try {
      const result = await exportCurrentScheme(selectedPreset.id, selectedRenderStyleId);
      setToast({
        tone: "success",
        message: result.exportPath ? `当前方案已导出到：${result.exportPath}` : "当前方案已经导出。",
      });
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "导出当前方案时没有成功。"),
      });
    }
  }

  async function handleRestoreDefaults() {
    setRescueOpen(false);
    setIsBusy(true);
    try {
      await restoreWindowsDefault();
      await refresh(true);
      setToast({
        tone: "info",
        message: "已经恢复到 Windows 原生设定。",
      });
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "恢复 Windows 原生设定时没有成功。"),
      });
    } finally {
      setIsBusy(false);
    }
  }

  async function handleRecoveryAction(action: string, successText: string) {
    setRescueOpen(false);
    setIsBusy(true);
    try {
      await runRecoveryAction(action);
      await refresh();
      setToast({
        tone: "info",
        message: successText,
      });
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "执行恢复动作时没有成功。"),
      });
    } finally {
      setIsBusy(false);
    }
  }

  async function handleRepairFonts() {
    setRescueOpen(false);
    try {
      await repairSystemFonts();
      setToast({
        tone: "info",
        message: "系统修复工具已经打开，接下来会自动运行 DISM 和 SFC。",
      });
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "系统修复工具没有顺利启动。"),
      });
    }
  }

  function shiftPreset(nextIndex: number) {
    if (!canSlide || nextIndex < 0 || nextIndex >= presets.length || nextIndex === currentIndex) {
      return;
    }

    setDirection(nextIndex > currentIndex ? 1 : -1);
    setCurrentIndex(nextIndex);
    setIsSliding(true);

    if (slideTimerRef.current !== null) {
      window.clearTimeout(slideTimerRef.current);
    }

    slideTimerRef.current = window.setTimeout(() => {
      setIsSliding(false);
      slideTimerRef.current = null;
    }, 320);
  }

  const primaryActionHandler = () => {
    if (!selectedPreset || isCurrentMatch || isBusy) {
      return;
    }

    if (!selectedPreset.available && selectedPreset.installMode === "manualImport") {
      void handleImportFonts();
      return;
    }

    void handleOpenApplySummary();
  };

  return (
    <div className="app-root">
      <header data-tauri-drag-region className="app-titlebar">
        <div className="app-titlebar__identity">
          <div className="app-logo">
            <img alt="" className="app-logo__image" src={appIcon} />
          </div>
          <div className="app-titlebar__text">
            <div className="app-titlebar__name">WindowsFontTuner 2.0</div>
            <div className="app-titlebar__sub">Windows 视觉调音师</div>
          </div>
        </div>

        <div className="app-titlebar__actions">
          <button
            className="app-link-pill"
            onClick={() => openUrl("https://github.com/soberbw-hash/WindowsFontTuner/releases")}
            type="button"
          >
            发布页
          </button>
          <WindowButton icon={<Minimize2 size={16} />} onClick={() => getCurrentWindow().minimize()} />
          <WindowButton icon={<Square size={14} />} onClick={() => getCurrentWindow().toggleMaximize()} />
          <WindowButton danger icon={<X size={16} />} onClick={() => getCurrentWindow().close()} />
        </div>
      </header>

      <main className="app-main">
          <div ref={stageRef} className="app-stage">
            <div ref={canvasRef} className="app-canvas" style={canvasStyle}>
            <section className="hero-card">
              <h1 className="hero-card__title">把系统字体换成你真正想看的样子</h1>
              <p className="hero-card__copy">
                先选一套你喜欢的方案，再叠一层更适合当前屏幕的渲染风格。备份、回滚、恢复默认这些脏活累活，都交给底层静默完成。
              </p>
              <div className="hero-card__meta">
                <StatusPill
                  icon={<MonitorSmartphone size={14} />}
                  label={bootstrap ? `${bootstrap.display.resolutionLabel} · ${bootstrap.display.scalePercent}%` : "正在识别屏幕"}
                />
                <StatusPill
                  icon={<Sparkles size={14} />}
                  label={
                    recommendedPreset && recommendedRenderStyle
                      ? `更适合你：${recommendedPreset.sceneName} / ${recommendedRenderStyle.label}`
                      : "正在生成推荐"
                  }
                />
                <StatusPill
                  icon={<ShieldCheck size={14} />}
                  label={bootstrap?.isAdmin ? "管理员已就绪" : "等待管理员权限"}
                  tone={bootstrap?.isAdmin ? "green" : "amber"}
                />
                <StatusPill icon={<RotateCcw size={14} />} label={`${bootstrap?.backupCount ?? 0} 份后悔药`} />
              </div>
            </section>

            <section className="workspace">
              <div className="workspace__flag">
                <Sparkles size={14} />
                当前正在看的方案
              </div>

              <button
                className="gallery-arrow gallery-arrow--left"
                disabled={!canGoPrev || !canSlide}
                onClick={() => shiftPreset(currentIndex - 1)}
                type="button"
              >
                <ArrowLeft size={18} />
              </button>

              <button
                className="gallery-arrow gallery-arrow--right"
                disabled={!canGoNext || !canSlide}
                onClick={() => shiftPreset(currentIndex + 1)}
                type="button"
              >
                <ArrowRight size={18} />
              </button>

              <div className="workspace__card-shell">
                {selectedPreset ? (
                  <AnimatePresence custom={direction} initial={false} mode="wait">
                    <motion.article
                      key={selectedPreset.id}
                      animate="center"
                      className="preview-card"
                      custom={direction}
                      exit="exit"
                      initial="enter"
                      transition={cardTransition}
                      variants={cardVariants}
                    >
                      <div className={`preview-card__hero bg-gradient-to-br ${selectedPreset.accentClass}`}>
                        <div className="preview-card__hero-main">
                          <div className="preview-card__chips">
                            <span className="preview-chip preview-chip--main">{selectedPreset.shortTag}</span>
                            <span className="preview-chip">{selectedPreset.mood}</span>
                            <span className={`preview-chip preview-chip--risk preview-chip--${selectedPreset.compatibility}`}>
                              {selectedPreset.compatibility}
                            </span>
                          </div>
                          <div className="preview-card__font-family" title={selectedPreset.fontFamily}>
                            {selectedPreset.fontFamily}
                          </div>
                          <p className="preview-card__intro">{selectedPreset.description}</p>
                        </div>

                        <section className="preview-card__status-card">
                          <div className="preview-card__status-eyebrow">当前状态</div>
                          <div className="preview-card__status-title">{statusTitle}</div>
                          <p className="preview-card__status-copy">{selectedPreset.recommendedFor}</p>
                          <MetaRow label="当前系统方案" value={bootstrap?.activeFontLabel ?? "Windows 默认"} />
                          <MetaRow label="当前渲染风格" value={currentRenderStyleLabel} />
                          <MetaRow label="屏幕匹配矩阵" value={bootstrap?.display.matrixProfile ?? "通用矩阵"} />
                          <MetaRow label="最近修改" value={bootstrap?.lastModifiedLabel ?? "刚刚读取"} />
                        </section>
                      </div>

                      <div className="preview-card__body">
                        <div className="preview-card__main">
                          <div className="preview-toolbar">
                            <div className="preview-toolbar__group">
                              <span className="preview-toolbar__label">预览场景</span>
                              <SegmentedButtons
                                activeValue={previewMode}
                                options={[
                                  { value: "ui", label: "UI" },
                                  { value: "reading", label: "阅读" },
                                  { value: "browser", label: "网页" },
                                  { value: "code", label: "代码" },
                                ]}
                                onChange={(value) => setPreviewMode(value as PreviewMode)}
                              />
                            </div>

                            <div className="preview-toolbar__group">
                              <span className="preview-toolbar__label">背景</span>
                              <SegmentedButtons
                                activeValue={previewTheme}
                                options={[
                                  { value: "light", label: "浅色" },
                                  { value: "dark", label: "深色" },
                                ]}
                                onChange={(value) => setPreviewTheme(value as PreviewTheme)}
                              />
                            </div>

                            <div className="preview-toolbar__group">
                              <span className="preview-toolbar__label">缩放</span>
                              <SegmentedButtons
                                activeValue={String(previewScale)}
                                options={previewScales.map((scale) => ({ value: String(scale), label: `${scale}%` }))}
                                onChange={(value) => setPreviewScale(Number(value) as PreviewScale)}
                              />
                            </div>
                          </div>

                          <div className="render-style-inline">
                            <div className="render-style-inline__label-wrap">
                              <span className="preview-toolbar__label">渲染风格</span>
                              <span className="render-style-inline__summary">
                                {selectedRenderStyle?.label ?? "平衡"} · {selectedRenderStyle?.summary ?? "默认推荐，清晰和舒适兼顾。"}
                              </span>
                            </div>
                            <div className="render-style-inline__list">
                              {renderStyles.map((style) => (
                                <button
                                  key={style.id}
                                  className={`render-style-pill ${selectedRenderStyleId === style.id ? "is-active" : ""} ${
                                    style.recommended ? "is-recommended" : ""
                                  }`}
                                  onClick={() => setSelectedRenderStyleId(style.id)}
                                  type="button"
                                >
                                  {style.label}
                                </button>
                              ))}
                            </div>
                          </div>

                          <div className="ab-grid">
                            <ABPreviewCard
                              footer={bootstrap?.activeFontLabel ?? "Windows 默认"}
                              label="当前系统"
                              mode={previewMode}
                              previewScale={previewScale}
                              sampleFont={SYSTEM_PREVIEW_FONT}
                              theme={previewTheme}
                              tone="current"
                            />
                            <ABPreviewCard
                              footer={`${selectedPreset.fontFamily} · ${selectedRenderStyle?.label ?? "平衡"}`}
                              label="应用后"
                              mode={previewMode}
                              previewScale={previewScale}
                              sampleFont={selectedPreset.previewFont}
                              theme={previewTheme}
                              tone="target"
                            />
                          </div>

                          <div className={`hero-preview hero-preview--${previewTheme}`}>
                            <div className="hero-preview__eyebrow">{scenario.eyebrow}</div>
                            <h2
                              className="hero-preview__title"
                              style={{
                                fontFamily: selectedPreset.previewFont,
                                fontSize: `${Math.round(40 * (previewScale / 100))}px`,
                              }}
                              title={scenario.title}
                            >
                              {scenario.title}
                            </h2>
                            <p className="hero-preview__body">{scenario.body}</p>
                            <p className="hero-preview__sub">{scenario.sub}</p>
                          </div>
                        </div>

                        <aside className="preview-card__sidebar">
                          <section className="side-panel side-panel--compact">
                            <div className="side-panel__eyebrow">文本观感体检</div>
                            <div className="health-summary">
                              <span className={`health-badge health-badge--${summary?.health.overallStatus ?? "warn"}`}>
                                {healthToneLabel(summary?.health.overallStatus ?? "warn")}
                              </span>
                              <p>{healthSummary}</p>
                            </div>
                            <div className="health-list">
                              {healthItems.slice(0, 2).map((item) => (
                                <HealthRow key={item.label} detail={item.detail} label={item.label} status={item.status} />
                              ))}
                            </div>
                          </section>
                        </aside>
                      </div>

                      <div className="preview-card__footerbar">
                        <div className="action-stack action-stack--footer">
                          <button
                            className="apply-button"
                            disabled={isBusy || isCurrentMatch}
                            onClick={primaryActionHandler}
                            type="button"
                          >
                            {isBusy ? <LoaderCircle className="spin" size={18} /> : <CheckCheck size={18} />}
                            {primaryActionLabel}
                          </button>

                          <div className="secondary-actions">
                            <SoftButton icon={<FolderUp size={16} />} onClick={handleImportFonts}>
                              导入字体
                            </SoftButton>
                            <SoftButton
                              icon={<ExternalLink size={16} />}
                              onClick={() => openUrl("https://github.com/soberbw-hash/WindowsFontTuner/releases")}
                            >
                              发布页
                            </SoftButton>
                          </div>
                        </div>
                      </div>
                    </motion.article>
                  </AnimatePresence>
                ) : null}
              </div>

              <div className="gallery-pagination">
                {presets.map((preset, index) => (
                  <button
                    key={preset.id}
                    aria-label={`切换到 ${preset.sceneName}`}
                    className={`gallery-pagination__dot ${index === currentIndex ? "is-active" : ""}`}
                    disabled={!canSlide}
                    onClick={() => shiftPreset(index)}
                    type="button"
                  />
                ))}
              </div>
            </section>
          </div>
        </div>
      </main>

      <div className="floating-tools">
        <AnimatePresence>
          {rescueOpen ? (
            <motion.div
              animate={{ opacity: 1, scale: 1, y: 0 }}
              className="rescue-panel"
              exit={{ opacity: 0, scale: 0.96, y: 12 }}
              initial={{ opacity: 0, scale: 0.96, y: 12 }}
              transition={{ duration: 0.18 }}
            >
              <RescueSection title="恢复中心">
                <RescueButton icon={<RotateCcw size={16} />} label="回滚上一次应用" onClick={handleRollbackLast} />
                <RescueButton
                  icon={<RefreshCw size={16} />}
                  label="恢复字体映射"
                  onClick={() => handleRecoveryAction("restoreFontMappings", "已经恢复字体映射。")}
                />
                <RescueButton
                  icon={<Sparkles size={16} />}
                  label="恢复 FontLink 回退链"
                  onClick={() => handleRecoveryAction("restoreFontLink", "已经恢复 FontLink 回退链。")}
                />
                <RescueButton
                  icon={<ScanSearch size={16} />}
                  label="恢复渲染参数"
                  onClick={() => handleRecoveryAction("restoreRendering", "已经恢复渲染参数。")}
                />
                <RescueButton icon={<ShieldCheck size={16} />} label="恢复 Windows 原生设定" onClick={handleRestoreDefaults} />
                <RescueButton icon={<Wrench size={16} />} label="修复系统字体文件" onClick={handleRepairFonts} />
                <RescueButton
                  icon={<RefreshCw size={16} />}
                  label="重新刷新 Explorer"
                  onClick={() => handleRecoveryAction("refreshExplorer", "资源管理器已经重新刷新。")}
                />
              </RescueSection>

              <RescueSection title="幕后入口">
                <RescueButton icon={<FolderUp size={16} />} label="导入本地字体" onClick={handleImportFonts} />
                <RescueButton icon={<FileUp size={16} />} label="导入分享方案" onClick={handleImportScheme} />
                <RescueButton icon={<FileDown size={16} />} label="导出当前方案" onClick={handleExportScheme} />
                <RescueButton
                  icon={<Coffee size={16} />}
                  label="支持这个项目"
                  onClick={() => {
                    setRescueOpen(false);
                    setSupportOpen(true);
                  }}
                />
              </RescueSection>

              <div className="expert-toggle">
                <button className="expert-toggle__button" onClick={() => setExpertMode((value) => !value)} type="button">
                  <TerminalSquare size={15} />
                  {expertMode ? "收起专家模式" : "展开专家模式"}
                  <ChevronDown size={15} className={expertMode ? "rotate-180" : ""} />
                </button>

                <AnimatePresence initial={false}>
                  {expertMode ? (
                    <motion.div
                      animate={{ opacity: 1, height: "auto" }}
                      className="expert-toggle__panel"
                      exit={{ opacity: 0, height: 0 }}
                      initial={{ opacity: 0, height: 0 }}
                    >
                      <div>当前屏幕：{bootstrap?.display.resolutionLabel ?? "--"} / {bootstrap?.display.scalePercent ?? 100}%</div>
                      <div>矩阵：{bootstrap?.display.matrixProfile ?? "通用矩阵"}</div>
                      <div>当前风格：{bootstrap?.activeFontLabel ?? "Windows 默认"} / {currentRenderStyleLabel}</div>
                      <div>后悔药目录：{bootstrap?.backupDir ?? "正在读取"}</div>
                      <div>急救说明：{bootstrap?.recovery.safeModeHint ?? "按住 Shift 启动即可静默恢复"}</div>
                      {importedPreset ? <div>最近导入：{importedPreset.name}</div> : null}
                    </motion.div>
                  ) : null}
                </AnimatePresence>
              </div>
            </motion.div>
          ) : null}
        </AnimatePresence>

        <button
          className={`rescue-trigger ${rescueOpen ? "is-active" : ""}`}
          onClick={() => setRescueOpen((value) => !value)}
          type="button"
        >
          <Settings2 size={18} />
        </button>
      </div>

      <AnimatePresence>
        {toast ? (
          <motion.div
            animate={{ opacity: 1, y: 0 }}
            className="toast-stack"
            exit={{ opacity: 0, y: 14 }}
            initial={{ opacity: 0, y: 14 }}
            transition={{ duration: 0.18 }}
          >
            <div className={`app-toast app-toast--${toast.tone}`}>
              <div className="app-toast__icon">
                {toast.tone === "success" ? <Check size={16} /> : toast.tone === "warning" ? <AlertTriangle size={16} /> : <Sparkles size={16} />}
              </div>
              <div className="app-toast__message">{toast.message}</div>
            </div>
            {supportNudge ? (
              <button
                className="support-nudge"
                onClick={() => {
                  setSupportNudge(false);
                  setSupportOpen(true);
                }}
                type="button"
              >
                <Coffee size={16} />
                请我喝杯咖啡
              </button>
            ) : null}
          </motion.div>
        ) : null}
      </AnimatePresence>

      <AnimatePresence>
        {keepBanner ? (
          <motion.div
            animate={{ opacity: 1, y: 0 }}
            className="keep-banner"
            exit={{ opacity: 0, y: -12 }}
            initial={{ opacity: 0, y: -12 }}
            transition={{ duration: 0.18 }}
          >
            <div className="keep-banner__copy">
              <div className="keep-banner__title">这次修改看起来正常吗？</div>
              <div className="keep-banner__body">还剩 {keepBanner.secondsLeft} 秒，你可以保留这次修改，也可以马上回滚。</div>
            </div>
            <div className="keep-banner__actions">
              <button className="keep-banner__ghost" onClick={() => setKeepBanner(null)} type="button">
                保留
              </button>
              <button className="keep-banner__danger" onClick={handleRollbackLast} type="button">
                立即回滚
              </button>
            </div>
          </motion.div>
        ) : null}
      </AnimatePresence>

      <AnimatePresence>
        {summaryOpen && summary ? (
          <motion.div animate={{ opacity: 1 }} className="modal-backdrop" exit={{ opacity: 0 }} initial={{ opacity: 0 }}>
            <motion.div
              animate={{ opacity: 1, scale: 1, y: 0 }}
              className="summary-modal"
              exit={{ opacity: 0, scale: 0.96, y: 10 }}
              initial={{ opacity: 0, scale: 0.96, y: 10 }}
              transition={{ duration: 0.2 }}
            >
              <div className="summary-modal__header">
                <div>
                  <div className="summary-modal__eyebrow">应用前摘要</div>
                  <h3>这次会怎么改你的系统观感</h3>
                </div>
                <button className="summary-modal__close" onClick={() => setSummaryOpen(false)} type="button">
                  <X size={16} />
                </button>
              </div>

              <div className="summary-modal__grid">
                <SummaryItem label="字体方案" value={summary.presetLabel} />
                <SummaryItem label="渲染风格" value={summary.renderStyleLabel} />
                <SummaryItem label="风险等级" value={riskLabel(summary.riskLevel)} />
                <SummaryItem label="文本体检" value={healthToneLabel(summary.health.overallStatus)} />
              </div>

              <div className="summary-modal__checks">
                <CheckLine ok={summary.willModifyFontSubstitutes}>会写入字体映射</CheckLine>
                <CheckLine ok={summary.willModifyFontLink}>会补 Emoji 与中文回退链</CheckLine>
                <CheckLine ok={summary.willWriteRendering}>会写入渲染参数</CheckLine>
                <CheckLine ok={summary.requiresExplorerRefresh}>应用后会刷新 Explorer</CheckLine>
                <CheckLine ok={summary.recommendSignOut}>建议注销后感受完整效果</CheckLine>
                <CheckLine ok={summary.willDownloadFonts}>若缺字体会先自动准备</CheckLine>
              </div>

              <div className="summary-modal__health">
                <div className="summary-modal__health-title">字体健康检测</div>
                <div className="health-list">
                  {summary.health.items.map((item) => (
                    <HealthRow key={item.label} detail={item.detail} label={item.label} status={item.status} />
                  ))}
                </div>
              </div>

              <div className="summary-modal__actions">
                <button className="soft-button soft-button--wide" onClick={() => setSummaryOpen(false)} type="button">
                  先不改
                </button>
                <button className="apply-button apply-button--inline" onClick={handleApplyConfirmed} type="button">
                  <CheckCheck size={18} />
                  就按这个来
                </button>
              </div>
            </motion.div>
          </motion.div>
        ) : null}
      </AnimatePresence>

      <AnimatePresence>
        {supportOpen ? (
          <motion.div animate={{ opacity: 1 }} className="modal-backdrop" exit={{ opacity: 0 }} initial={{ opacity: 0 }}>
            <motion.div
              animate={{ opacity: 1, scale: 1, y: 0 }}
              className="support-modal"
              exit={{ opacity: 0, scale: 0.96, y: 10 }}
              initial={{ opacity: 0, scale: 0.96, y: 10 }}
              transition={{ duration: 0.2 }}
            >
              <div className="summary-modal__header">
                <div>
                  <div className="summary-modal__eyebrow">支持这个项目</div>
                  <h3>如果这个小工具让你的屏幕更顺眼了</h3>
                </div>
                <button className="summary-modal__close" onClick={() => setSupportOpen(false)} type="button">
                  <X size={16} />
                </button>
              </div>

              <p className="support-modal__copy">不妨请我喝杯咖啡。谢谢你愿意支持这个项目继续变得更稳、更顺眼。</p>

              <div className="support-modal__qr-shell">
                <img alt="支持二维码" className="support-modal__qr" src={supportQr} />
              </div>

              <div className="support-modal__actions">
                <button className="soft-button soft-button--wide" onClick={() => setSupportOpen(false)} type="button">
                  稍后再说
                </button>
                <button
                  className="apply-button apply-button--inline"
                  onClick={() => openUrl("https://github.com/soberbw-hash/WindowsFontTuner")}
                  type="button"
                >
                  <ExternalLink size={18} />
                  看 GitHub
                </button>
              </div>
            </motion.div>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}

function WindowButton({
  icon,
  onClick,
  danger = false,
}: {
  icon: ReactNode;
  onClick: () => void;
  danger?: boolean;
}) {
  return (
    <button className={`window-button ${danger ? "window-button--danger" : ""}`} onClick={onClick} type="button">
      {icon}
    </button>
  );
}

function StatusPill({
  icon,
  label,
  tone = "default",
}: {
  icon: ReactNode;
  label: string;
  tone?: "default" | "green" | "amber";
}) {
  return (
    <div className={`status-pill status-pill--${tone}`}>
      {icon}
      <span>{label}</span>
    </div>
  );
}

function SegmentedButtons({
  activeValue,
  options,
  onChange,
}: {
  activeValue: string;
  options: Array<{ value: string; label: string }>;
  onChange: (value: string) => void;
}) {
  return (
    <div className="segment">
      {options.map((option) => (
        <button
          key={option.value}
          className={`segment__item ${activeValue === option.value ? "is-active" : ""}`}
          onClick={() => onChange(option.value)}
          type="button"
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

function ABPreviewCard({
  label,
  sampleFont,
  footer,
  theme,
  tone,
  previewScale,
  mode,
}: {
  label: string;
  sampleFont: string;
  footer: string;
  theme: PreviewTheme;
  tone: "current" | "target";
  previewScale: PreviewScale;
  mode: PreviewMode;
}) {
  const scaleFactor = previewScale / 100;

  return (
    <div className={`ab-card ab-card--${theme} ab-card--${tone}`}>
      <div className="ab-card__label">{label}</div>
      {mode === "ui" ? (
        <>
          <div className="ab-card__ui-title" style={{ fontFamily: sampleFont, fontSize: `${Math.round(22 * scaleFactor)}px` }}>
            敏捷的棕色狐狸
          </div>
          <div className="ab-card__ui-meta" style={{ fontFamily: sampleFont, fontSize: `${Math.round(16 * scaleFactor)}px` }}>
            Aa 0123456789
          </div>
          <div className="ab-card__ui-copy" style={{ fontFamily: sampleFont, fontSize: `${Math.round(13 * scaleFactor)}px` }}>
            The quick brown fox jumps over the lazy dog.
          </div>
        </>
      ) : mode === "reading" ? (
        <>
          <div className="ab-card__ui-title" style={{ fontFamily: sampleFont, fontSize: `${Math.round(18 * scaleFactor)}px` }}>
            Windows 字体观感调音台
          </div>
          <div className="ab-card__paragraph" style={{ fontFamily: sampleFont, fontSize: `${Math.round(12 * scaleFactor)}px` }}>
            这是一段中文正文，用来观察笔画、字腔和灰度层次。
          </div>
          <div className="ab-card__paragraph" style={{ fontFamily: sampleFont, fontSize: `${Math.round(11 * scaleFactor)}px` }}>
            USB4 40Gbps / PCIe 4.0 / 12.9"
          </div>
        </>
      ) : mode === "browser" ? (
        <>
          <div className="ab-card__ui-title" style={{ fontFamily: sampleFont, fontSize: `${Math.round(18 * scaleFactor)}px` }}>
            Typography Tuning for Windows
          </div>
          <div className="ab-card__paragraph" style={{ fontFamily: sampleFont, fontSize: `${Math.round(12 * scaleFactor)}px` }}>
            Settings / Display / Update / Explorer / Preview
          </div>
          <div className="ab-card__paragraph" style={{ fontFamily: sampleFont, fontSize: `${Math.round(11 * scaleFactor)}px` }}>
            % ￥ $ @ # &amp; * +
          </div>
        </>
      ) : (
        <>
          <pre className="ab-card__code" style={{ fontFamily: sampleFont, fontSize: `${Math.round(11 * scaleFactor)}px` }}>
            {`const fontMode = "balanced";\nfunction renderPreview() {\n  return "clear text";\n}`}
          </pre>
          <div className="ab-card__paragraph" style={{ fontFamily: sampleFont, fontSize: `${Math.round(11 * scaleFactor)}px` }}>
            资源管理器 · 设置页 · 终端
          </div>
        </>
      )}

      <div className="ab-card__footer">{footer}</div>
    </div>
  );
}

function HealthRow({
  label,
  detail,
  status,
}: {
  label: string;
  detail: string;
  status: CheckStatus;
}) {
  return (
    <div className="health-row">
      <span className={`health-row__dot health-row__dot--${status}`} />
      <div className="health-row__text">
        <div className="health-row__label">{label}</div>
        <div className="health-row__detail">{detail}</div>
      </div>
    </div>
  );
}

function MetaRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="meta-row">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function SoftButton({
  icon,
  onClick,
  children,
}: {
  icon: ReactNode;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button className="soft-button" onClick={onClick} type="button">
      {icon}
      {children}
    </button>
  );
}

function RescueSection({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="rescue-panel__section">
      <div className="rescue-panel__title">{title}</div>
      <div className="rescue-panel__group">{children}</div>
    </section>
  );
}

function RescueButton({
  icon,
  label,
  onClick,
}: {
  icon: ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button className="rescue-panel__button" onClick={onClick} type="button">
      {icon}
      <span>{label}</span>
    </button>
  );
}

function SummaryItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="summary-item">
      <div className="summary-item__label">{label}</div>
      <div className="summary-item__value">{value}</div>
    </div>
  );
}

function CheckLine({ children, ok }: { children: ReactNode; ok: boolean }) {
  return (
    <div className={`check-line ${ok ? "is-ok" : "is-muted"}`}>
      <span className="check-line__icon">{ok ? <Check size={14} /> : <AlertTriangle size={14} />}</span>
      <span>{children}</span>
    </div>
  );
}

function buildPreviewScenario(preset: DisplayPreset | null, mode: PreviewMode): PreviewScenario {
  if (!preset) {
    return {
      eyebrow: "正在读取",
      title: "正在准备你的字体方案",
      body: "先把当前系统状态、屏幕缩放和后悔药都核对清楚。",
      sub: "Everything is getting ready.",
    };
  }

  if (mode === "reading") {
    return {
      eyebrow: "阅读视角",
      title: preset.headline,
      body: "这段预览会更强调正文节奏、长文耐看度和句子之间的呼吸感。",
      sub: "This view is tuned for longer reading sessions.",
    };
  }

  if (mode === "browser") {
    return {
      eyebrow: "网页视角",
      title: preset.headline,
      body: "会模拟浏览器正文、设置页和混排内容，让你先看清楚日常最常碰到的那一类文本。",
      sub: "A browser-like preview with UI copy and mixed content.",
    };
  }

  if (mode === "code") {
    return {
      eyebrow: "代码视角",
      title: preset.headline,
      body: "这里会把小字号字母、数字、标点和列表边界一起摆出来，更容易判断符号辨识度。",
      sub: "Sharper symbols and steadier small text for dense interfaces.",
    };
  }

  return {
    eyebrow: "系统桌面",
    title: preset.headline,
    body: "这里会直接模拟资源管理器、设置页和常见 UI 文本，不用真正改完才知道顺不顺眼。",
    sub: preset.englishLine,
  };
}

function fallbackHealthItems(preset: DisplayPreset | null): HealthItem[] {
  if (!preset) {
    return [
      { label: "字体文件", status: "warn", detail: "正在确认这套方案的字体是否都已经就位。" },
      { label: "Emoji 回退", status: "pass", detail: "会自动追加 Segoe UI Emoji 兜底。" },
      { label: "长期默认", status: "warn", detail: "先看预览，再决定是否长期保留。" },
    ];
  }

  const stability: Record<DisplayPreset["compatibility"], CheckStatus> = {
    稳定: "pass",
    谨慎: "warn",
    实验: "risk",
  };

  return [
    { label: "中文字形覆盖", status: "pass", detail: `${preset.fontFamily} 的中文主字形已进入本次检查。` },
    { label: "Emoji 与 UI 回退", status: "pass", detail: "会自动补上 Segoe UI Emoji 和 Microsoft YaHei 回退链。" },
    {
      label: "长期默认风险",
      status: stability[preset.compatibility],
      detail:
        preset.compatibility === "稳定"
          ? "适合做长期默认。"
          : preset.compatibility === "谨慎"
            ? "建议先试一段时间再决定长期保留。"
            : "更适合体验和自定义导入场景。",
    },
  ];
}

function healthToneLabel(status: CheckStatus) {
  switch (status) {
    case "pass":
      return "通过";
    case "warn":
      return "留意";
    case "risk":
      return "谨慎";
    default:
      return "留意";
  }
}

function riskLabel(level: RiskLevel) {
  switch (level) {
    case "low":
      return "低";
    case "medium":
      return "中";
    case "high":
      return "高";
    default:
      return "中";
  }
}

function formatError(error: unknown, fallback: string) {
  if (typeof error === "string") {
    return error;
  }

  if (error && typeof error === "object" && "message" in error) {
    const message = (error as { message?: string }).message;
    if (message) {
      return message;
    }
  }

  return fallback;
}

const cardVariants = {
  enter: (direction: number) => ({
    opacity: 0,
    x: direction > 0 ? 46 : -46,
    scale: 0.985,
  }),
  center: {
    opacity: 1,
    x: 0,
    scale: 1,
  },
  exit: (direction: number) => ({
    opacity: 0,
    x: direction > 0 ? -46 : 46,
    scale: 0.985,
  }),
};

const cardTransition = {
  type: "spring" as const,
  stiffness: 210,
  damping: 24,
  mass: 0.95,
};

export default App;
