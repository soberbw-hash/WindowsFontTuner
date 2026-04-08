import { getCurrentWindow } from "@tauri-apps/api/window";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { AnimatePresence, motion } from "framer-motion";
import {
  ArrowLeft,
  ArrowRight,
  CheckCheck,
  Coffee,
  Crown,
  ExternalLink,
  FolderUp,
  HeartHandshake,
  Minimize2,
  MonitorSmartphone,
  RefreshCw,
  RotateCcw,
  Settings2,
  ShieldCheck,
  Square,
  WandSparkles,
  Wrench,
  X,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import type { CSSProperties, ReactNode } from "react";
import supportQr from "./assets/support-qr.png";
import appIcon from "./assets/app-icon.png";
import { presetCatalog } from "./lib/presets";
import { applyPreset, importFontFiles, loadBootstrap, repairSystemFonts, restoreWindowsDefault } from "./lib/tauri";
import type { BootstrapPayload, DisplayPreset } from "./types";

const CANVAS_WIDTH = 1440;
const CANVAS_HEIGHT = 784;
const SYSTEM_PREVIEW_FONT = '"Segoe UI Variable", "Microsoft YaHei UI", sans-serif';

type ToastTone = "success" | "warning" | "info";

interface ToastState {
  tone: ToastTone;
  message: string;
}

function App() {
  const [bootstrap, setBootstrap] = useState<BootstrapPayload | null>(null);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [direction, setDirection] = useState(0);
  const [canvasScale, setCanvasScale] = useState(1);
  const [rescueOpen, setRescueOpen] = useState(false);
  const [supportOpen, setSupportOpen] = useState(false);
  const [supportNudge, setSupportNudge] = useState(false);
  const [isBusy, setIsBusy] = useState(false);
  const [isSliding, setIsSliding] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);
  const slideTimerRef = useRef<number | null>(null);
  const stageRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    void refresh();
  }, []);

  useEffect(() => {
    const stage = stageRef.current;
    if (!stage) {
      return;
    }

    let frameId: number | null = null;
    const updateScale = () => {
      const availableWidth = Math.max(stage.clientWidth - 4, 1);
      const availableHeight = Math.max(stage.clientHeight - 4, 1);
      const next = Math.min(availableWidth / CANVAS_WIDTH, availableHeight / CANVAS_HEIGHT, 1);
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

    const observer = new ResizeObserver(() => {
      scheduleScale();
    });

    observer.observe(stage);
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
      if (slideTimerRef.current) {
        window.clearTimeout(slideTimerRef.current);
      }
    };
  }, []);

  useEffect(() => {
    if (!toast) {
      return;
    }

    const timer = window.setTimeout(() => setToast(null), 3400);
    return () => window.clearTimeout(timer);
  }, [toast]);

  const presets = useMemo<DisplayPreset[]>(() => {
    const runtime = new Map((bootstrap?.presets ?? []).map((item) => [item.id, item]));
    return presetCatalog.map((preset) => {
      const current = runtime.get(preset.id);
      return {
        ...preset,
        status: current?.status ?? "ready",
        installMode: current?.installMode ?? "manualImport",
        available: current?.available ?? false,
        current: current?.current ?? false,
      };
    });
  }, [bootstrap]);

  const currentPreset = presets[currentIndex] ?? presets[0];
  const canGoPrev = currentIndex > 0;
  const canGoNext = currentIndex < presets.length - 1;
  const canApply = !!currentPreset && !currentPreset.current && !!bootstrap?.isAdmin && !isBusy;
  const canSlide = !isSliding && !isBusy;
  const longName = currentPreset.name.length >= 15;
  const compactSummary = currentPreset.heroLine.length >= 12;

  const canvasStyle = {
    width: `${CANVAS_WIDTH}px`,
    height: `${CANVAS_HEIGHT}px`,
    transform: `scale(${canvasScale})`,
  } satisfies CSSProperties;

  async function refresh() {
    try {
      const next = await loadBootstrap();
      setBootstrap(next);

      const selectedId = next.activePresetId ?? presetCatalog[0]?.id;
      const selectedIndex = presetCatalog.findIndex((item) => item.id === selectedId);
      setCurrentIndex(selectedIndex >= 0 ? selectedIndex : 0);
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "启动时没能读取到当前系统状态。"),
      });
    }
  }

  async function handleApply() {
    if (!currentPreset || !canApply) {
      return;
    }

    setIsBusy(true);
    setSupportNudge(false);
    try {
      const result = await applyPreset(currentPreset.id);
      await refresh();
      setToast({
        tone: "success",
        message: result.message || "✨ 风格已应用，注销电脑后即可感受。",
      });

      window.setTimeout(() => {
        setSupportNudge(true);
      }, 1500);
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "应用风格时出了点问题。"),
      });
    } finally {
      setIsBusy(false);
    }
  }

  async function handleImport() {
    const picked = await openFileDialog({
      multiple: true,
      filters: [
        {
          name: "字体文件",
          extensions: ["ttf", "otf", "ttc", "otc"],
        },
      ],
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
      const result = await importFontFiles(paths);
      await refresh();
      setToast({
        tone: "success",
        message: result.message,
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

  async function handleRestoreDefaults() {
    setRescueOpen(false);
    setIsBusy(true);
    try {
      const result = await restoreWindowsDefault();
      await refresh();
      setToast({
        tone: "info",
        message: result.message,
      });
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "恢复原生设定时没有成功。"),
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
    if (nextIndex < 0 || nextIndex >= presets.length || nextIndex === currentIndex || !canSlide) {
      return;
    }

    setDirection(nextIndex > currentIndex ? 1 : -1);
    setCurrentIndex(nextIndex);
    setIsSliding(true);

    if (slideTimerRef.current) {
      window.clearTimeout(slideTimerRef.current);
    }

    slideTimerRef.current = window.setTimeout(() => {
      setIsSliding(false);
      slideTimerRef.current = null;
    }, 320);
  }

  return (
    <div className="relative z-10 flex h-full flex-col text-[var(--app-ink)]">
      <header data-tauri-drag-region className="app-titlebar">
        <div className="app-titlebar__identity">
          <div className="app-logo">
            <img alt="" className="app-logo__image" src={appIcon} />
          </div>
          <div className="leading-tight">
            <div className="text-[15px] font-semibold tracking-tight">WindowsFontTuner 2.0</div>
            <div className="text-[11px] text-[var(--app-muted)]">Windows 视觉调音师</div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button className="app-link-pill" onClick={() => openUrl("https://github.com/soberbw-hash/WindowsFontTuner/releases")} type="button">
            发布页
          </button>
          <WindowButton icon={<Minimize2 size={16} />} onClick={() => getCurrentWindow().minimize()} />
          <WindowButton icon={<Square size={14} />} onClick={() => getCurrentWindow().toggleMaximize()} />
          <WindowButton danger icon={<X size={16} />} onClick={() => getCurrentWindow().close()} />
        </div>
      </header>

      <main className="flex-1 overflow-hidden px-6 py-4">
        <div ref={stageRef} className="app-canvas-stage">
          <div className="app-canvas" style={canvasStyle}>
            <section className="app-hero">
              <div className="app-hero__title">把系统字体换成你真正想看的样子</div>
              <p className="app-hero__copy">
                不再堆技术名词，不再给你一坨下拉框。选一个你喜欢的风格，剩下的脏活累活交给底层静默完成。
              </p>
              <div className="app-hero__meta">
                <StatusPill icon={<MonitorSmartphone size={14} />} label={bootstrap?.display.resolutionLabel ?? "正在读取屏幕"} />
                <StatusPill icon={<WandSparkles size={14} />} label={bootstrap?.display.matrixProfile ?? "正在匹配矩阵"} />
                <StatusPill
                  icon={<ShieldCheck size={14} />}
                  label={bootstrap?.isAdmin ? "管理员已就绪" : "等待管理员权限"}
                  tone={bootstrap?.isAdmin ? "green" : "amber"}
                />
                <StatusPill icon={<RefreshCw size={14} />} label={`${bootstrap?.backupCount ?? 0} 份后悔药`} />
              </div>
            </section>

            <section className="gallery-shell">
              <div className="gallery-shell__flag">
                <Crown size={14} className="text-[var(--app-blue)]" />
                当前正在看的风格
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

              <div className="gallery-card-shell">
                <AnimatePresence custom={direction} initial={false} mode="wait">
                  <motion.article
                    key={currentPreset.id}
                    animate="center"
                    className="gallery-card"
                    custom={direction}
                    exit="exit"
                    initial="enter"
                    transition={cardTransition}
                    variants={cardVariants}
                  >
                    <div className={`gallery-card__top bg-gradient-to-br ${currentPreset.accentClass}`}>
                      <div className="gallery-card__chips">
                        <span className="gallery-card__chip gallery-card__chip--main">{currentPreset.tag}</span>
                        <span className="gallery-card__chip">{currentPreset.vibe}</span>
                        {currentPreset.installMode === "manualImport" ? (
                          <span className="gallery-card__chip gallery-card__chip--accent">先导入</span>
                        ) : null}
                      </div>

                      <div className="gallery-card__title-group">
                        <h2
                          className={`gallery-card__font-name ${longName ? "gallery-card__font-name--compact" : ""}`}
                          style={{ fontFamily: currentPreset.previewFont }}
                        >
                          {currentPreset.name}
                        </h2>
                        <p className="gallery-card__topline">{currentPreset.noteLine}</p>
                      </div>
                    </div>

                    <div className="gallery-card__bottom">
                      <div className="gallery-card__main">
                        <div className="gallery-card__preview-grid">
                          <PreviewCard footer={bootstrap?.activeFontLabel ?? "Windows 默认"} label="当前系统" sampleFont={SYSTEM_PREVIEW_FONT} tone="muted" />
                          <PreviewCard footer={currentPreset.name} label="应用后" sampleFont={currentPreset.previewFont} tone="primary" />
                        </div>

                        <div className="gallery-card__summary">
                          <h3 className={`gallery-card__summary-title ${compactSummary ? "gallery-card__summary-title--compact" : ""}`}>
                            {currentPreset.heroLine}
                          </h3>
                          <p className="gallery-card__summary-text">{currentPreset.description}</p>
                          <p className="gallery-card__summary-english">{currentPreset.englishLine}</p>
                        </div>
                      </div>

                      <aside className="gallery-card__status">
                        <div className="space-y-2.5">
                          <div className="text-[12px] font-semibold uppercase tracking-[0.24em] text-[var(--app-muted)]">当前状态</div>
                          <div className="text-[24px] font-semibold tracking-[-0.05em] text-slate-950">{resolveStatusTitle(currentPreset)}</div>
                          <p className="text-[13px] leading-6 text-[var(--app-muted)]">{currentPreset.recommendedFor}</p>
                        </div>

                        <div className="space-y-2">
                          <MetaRow label="当前系统风格" value={bootstrap?.activeFontLabel ?? "Windows 默认"} />
                          <MetaRow label="屏幕匹配矩阵" value={bootstrap?.display.matrixProfile ?? "正在识别"} />
                          <MetaRow
                            label="备份状态"
                            value={bootstrap?.backupCount ? `${bootstrap.backupCount} 份快照已就位` : "还没写入任何修改"}
                          />
                        </div>

                        <div className="gallery-card__actions">
                          <button className="gallery-card__apply" disabled={!canApply} onClick={handleApply} type="button">
                            {isBusy ? <RefreshCw size={18} className="animate-spin" /> : <CheckCheck size={18} />}
                            {resolveApplyLabel(currentPreset, bootstrap?.isAdmin ?? false)}
                          </button>

                          <div className="gallery-card__secondary">
                            <SoftButton icon={<FolderUp size={16} />} onClick={handleImport}>
                              导入本地字体
                            </SoftButton>
                            <SoftButton
                              icon={<ExternalLink size={16} />}
                              onClick={() => openUrl("https://github.com/soberbw-hash/WindowsFontTuner/releases")}
                            >
                              看发布页
                            </SoftButton>
                          </div>
                        </div>
                      </aside>
                    </div>
                  </motion.article>
                </AnimatePresence>
              </div>

              <div className="gallery-pagination">
                {presets.map((preset, index) => (
                  <button
                    key={preset.id}
                    aria-label={`切换到 ${preset.name}`}
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

      <div className="pointer-events-none absolute bottom-6 right-6 z-30">
        <div className="pointer-events-auto flex flex-col items-end gap-3">
          <AnimatePresence>
            {rescueOpen ? (
              <motion.div
                animate={{ opacity: 1, scale: 1, y: 0 }}
                className="rescue-panel"
                exit={{ opacity: 0, scale: 0.96, y: 12 }}
                initial={{ opacity: 0, scale: 0.96, y: 14 }}
                transition={{ duration: 0.18 }}
              >
                <RescueButton icon={<RotateCcw size={16} />} label="恢复 Windows 原生设定" onClick={handleRestoreDefaults} />
                <RescueButton icon={<Wrench size={16} />} label="修复系统字体文件" onClick={handleRepairFonts} />
                <RescueButton icon={<FolderUp size={16} />} label="高级自定义导入" onClick={handleImport} />
                <RescueButton
                  icon={<HeartHandshake size={16} />}
                  label="支持这个项目"
                  onClick={() => {
                    setRescueOpen(false);
                    setSupportOpen(true);
                  }}
                />
              </motion.div>
            ) : null}
          </AnimatePresence>

          <button className="rescue-trigger" onClick={() => setRescueOpen((value) => !value)} type="button">
            <Settings2 size={18} />
          </button>
        </div>
      </div>

      <AnimatePresence>
        {toast ? (
          <motion.div
            animate={{ opacity: 1, scale: 1, y: 0 }}
            className="toast-stack"
            exit={{ opacity: 0, scale: 0.98, y: -12 }}
            initial={{ opacity: 0, scale: 0.98, y: -18 }}
            transition={{ duration: 0.18 }}
          >
            <div className={`app-toast app-toast--${toast.tone}`}>
              <div>{toast.message}</div>
            </div>

            <AnimatePresence>
              {supportNudge ? (
                <motion.button
                  animate={{ opacity: 1, y: 0 }}
                  className="support-nudge"
                  exit={{ opacity: 0, y: -8 }}
                  initial={{ opacity: 0, y: -8 }}
                  onClick={() => setSupportOpen(true)}
                  transition={{ duration: 0.18 }}
                  type="button"
                >
                  <Coffee size={16} />
                  请我喝杯咖啡
                </motion.button>
              ) : null}
            </AnimatePresence>
          </motion.div>
        ) : null}
      </AnimatePresence>

      <AnimatePresence>
        {supportOpen ? (
          <motion.div
            animate={{ opacity: 1 }}
            className="support-modal-backdrop"
            exit={{ opacity: 0 }}
            initial={{ opacity: 0 }}
            onClick={() => setSupportOpen(false)}
            transition={{ duration: 0.18 }}
          >
            <motion.div
              animate={{ opacity: 1, scale: 1, y: 0 }}
              className="support-modal"
              exit={{ opacity: 0, scale: 0.96, y: 16 }}
              initial={{ opacity: 0, scale: 0.96, y: 20 }}
              onClick={(event) => event.stopPropagation()}
              transition={{ duration: 0.22, ease: [0.22, 1, 0.36, 1] }}
            >
              <div className="flex items-center gap-2 text-[13px] font-medium text-[var(--app-muted)]">
                <Coffee size={15} className="text-[var(--app-blue)]" />
                如果这个小工具让你的屏幕看起来更顺眼了，不妨请我喝杯咖啡。
              </div>
              <img alt="赞助二维码" className="support-modal__qr" src={supportQr} />
              <div className="text-center text-[14px] leading-7 text-[var(--app-muted)]">
                为你创造更舒适的 Windows 体验。感谢支持。
              </div>
              <button className="app-link-pill self-center" onClick={() => setSupportOpen(false)} type="button">
                收起
              </button>
            </motion.div>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}

const cardVariants = {
  enter: (direction: number) => ({
    opacity: 0,
    x: direction > 0 ? 56 : -56,
    scale: 0.992,
  }),
  center: {
    opacity: 1,
    x: 0,
    scale: 1,
  },
  exit: (direction: number) => ({
    opacity: 0,
    x: direction > 0 ? -56 : 56,
    scale: 0.992,
  }),
};

const cardTransition = {
  duration: 0.28,
  ease: [0.22, 1, 0.36, 1],
} as const;

function PreviewCard(props: {
  label: string;
  sampleFont: string;
  footer: string;
  tone: "muted" | "primary";
}) {
  return (
    <div className={`preview-card preview-card--${props.tone}`}>
      <div className="preview-card__label">{props.label}</div>
      <div className="preview-card__sample" style={{ fontFamily: props.sampleFont }}>
        <div className="preview-card__headline">敏捷的棕色狐狸</div>
        <div className="preview-card__digits">Aa 0123456789</div>
        <div className="preview-card__english">The quick brown fox jumps over the lazy dog.</div>
      </div>
      <div className="preview-card__footer">{props.footer}</div>
    </div>
  );
}

function WindowButton(props: {
  icon: ReactNode;
  danger?: boolean;
  onClick: () => Promise<void> | void;
}) {
  return (
    <button className={`window-button ${props.danger ? "window-button--danger" : ""}`} onClick={props.onClick} type="button">
      {props.icon}
    </button>
  );
}

function StatusPill(props: { icon: ReactNode; label: string; tone?: "green" | "amber" }) {
  return (
    <div className={`status-pill ${props.tone ? `status-pill--${props.tone}` : ""}`}>
      {props.icon}
      {props.label}
    </div>
  );
}

function SoftButton(props: {
  icon: ReactNode;
  children: ReactNode;
  onClick: () => void | Promise<void>;
}) {
  return (
    <button className="soft-button" onClick={props.onClick} type="button">
      {props.icon}
      {props.children}
    </button>
  );
}

function RescueButton(props: { icon: ReactNode; label: string; onClick: () => void | Promise<void> }) {
  return (
    <button className="rescue-panel__button" onClick={props.onClick} type="button">
      {props.icon}
      {props.label}
    </button>
  );
}

function MetaRow(props: { label: string; value: string }) {
  return (
    <div className="meta-row">
      <span>{props.label}</span>
      <span>{props.value}</span>
    </div>
  );
}

function resolveStatusTitle(preset: DisplayPreset) {
  if (preset.current) {
    return "当前正在使用";
  }

  if (preset.available) {
    return "可以直接应用";
  }

  if (preset.installMode === "manualImport") {
    return "先导入字体";
  }

  return "一键自动装好";
}

function resolveApplyLabel(preset: DisplayPreset, isAdmin: boolean) {
  if (!isAdmin) {
    return "请以管理员身份启动";
  }

  if (preset.current) {
    return "当前正在使用";
  }

  if (preset.installMode === "manualImport" && !preset.available) {
    return "先导入字体";
  }

  return "应用此风格";
}

function formatError(error: unknown, fallback: string) {
  if (typeof error === "string" && error.trim()) {
    return error;
  }

  if (error && typeof error === "object" && "message" in error) {
    const message = String((error as { message?: string }).message ?? "");
    if (message.trim()) {
      return message;
    }
  }

  return fallback;
}

export default App;
