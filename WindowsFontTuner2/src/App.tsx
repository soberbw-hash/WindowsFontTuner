import { AnimatePresence, motion } from "framer-motion";
import { openUrl } from "@tauri-apps/plugin-opener";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  ArrowLeft,
  ArrowRight,
  CheckCheck,
  Crown,
  DownloadCloud,
  Minimize2,
  MonitorSmartphone,
  RefreshCw,
  RotateCcw,
  Settings2,
  ShieldCheck,
  Sparkles,
  Square,
  WandSparkles,
  Wrench,
  X,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { applyPreset, loadBootstrap, repairSystemFonts, restoreWindowsDefault } from "./lib/tauri";
import { presetCatalog } from "./lib/presets";
import type { BootstrapPayload, DisplayPreset } from "./types";

type ToastTone = "success" | "warning" | "info";

interface ToastState {
  tone: ToastTone;
  message: string;
}

const cardMotion = {
  initial: { opacity: 0, y: 28, scale: 0.985 },
  animate: { opacity: 1, y: 0, scale: 1 },
  exit: { opacity: 0, y: -24, scale: 0.98 },
  transition: { duration: 0.32, ease: [0.22, 1, 0.36, 1] as const },
};

function App() {
  const [bootstrap, setBootstrap] = useState<BootstrapPayload | null>(null);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [rescueOpen, setRescueOpen] = useState(false);
  const [isBusy, setIsBusy] = useState(false);
  const [toast, setToast] = useState<ToastState | null>(null);
  const [addons, setAddons] = useState({
    cursor: false,
    terminal: false,
    emoji: false,
  });

  useEffect(() => {
    void refresh();
  }, []);

  useEffect(() => {
    if (!toast) {
      return;
    }

    const timer = window.setTimeout(() => setToast(null), 3200);
    return () => window.clearTimeout(timer);
  }, [toast]);

  const presets = useMemo<DisplayPreset[]>(() => {
    const runtime = new Map((bootstrap?.presets ?? []).map((item) => [item.id, item]));

    return presetCatalog.map((preset) => {
      const state = runtime.get(preset.id);
      return {
        ...preset,
        status: state?.status ?? "soon",
        available: state?.available ?? false,
        current: state?.current ?? false,
      };
    });
  }, [bootstrap]);

  const currentPreset = presets[currentIndex] ?? presets[0];
  const canGoPrev = currentIndex > 0;
  const canGoNext = currentIndex < presets.length - 1;
  const isReady = currentPreset?.status === "ready";
  const canApply =
    !!currentPreset &&
    isReady &&
    currentPreset.available &&
    !currentPreset.current &&
    !!bootstrap?.isAdmin &&
    !isBusy;

  async function refresh() {
    try {
      const next = await loadBootstrap();
      setBootstrap(next);

      const selectedId = next.activePresetId ?? presetCatalog[0]?.id;
      const index = presetCatalog.findIndex((item) => item.id === selectedId);
      setCurrentIndex(index >= 0 ? index : 0);
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "启动时没能读到当前系统状态。"),
      });
    }
  }

  async function handleApply() {
    if (!currentPreset || !canApply) {
      return;
    }

    setIsBusy(true);
    try {
      const result = await applyPreset(currentPreset.id);
      await refresh();
      setToast({
        tone: "success",
        message: result.message || "✨ 风格已应用，注销电脑后即可感受。",
      });
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "应用风格时出了点问题。"),
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
        message: result.message || "已恢复到 Windows 默认字体链路。",
      });
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "恢复默认时没有成功。"),
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
        message: "系统修复工具已经打开，接下来会自动跑 DISM 和 SFC。",
      });
    } catch (error) {
      setToast({
        tone: "warning",
        message: formatError(error, "系统修复工具没有顺利启动。"),
      });
    }
  }

  function showComingSoon(label: string) {
    setToast({
      tone: "info",
      message: `${label} 会在下一阶段接进 2.0，这一版先把核心字体链路打稳。`,
    });
  }

  return (
    <div className="relative z-10 flex h-full flex-col overflow-hidden text-[var(--app-ink)]">
      <header
        data-tauri-drag-region
        className="flex h-14 shrink-0 items-center justify-between border-b border-white/35 bg-white/30 px-4 backdrop-blur-xl"
      >
        <div className="flex items-center gap-3">
          <div className="grid h-9 w-9 place-items-center rounded-2xl bg-[linear-gradient(135deg,#2d6df7_0%,#82b9ff_100%)] text-lg font-semibold text-white shadow-[0_16px_32px_rgba(47,109,246,0.28)]">
            Aa
          </div>
          <div className="leading-tight">
            <div className="text-[15px] font-semibold tracking-tight">WindowsFontTuner 2.0</div>
            <div className="text-[11px] text-[var(--app-muted)]">Windows 视觉调音师</div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            className="rounded-full bg-white/55 px-3 py-1.5 text-xs font-medium text-[var(--app-muted)] transition hover:bg-white/80"
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

      <main className="flex-1 overflow-y-auto px-6 pb-8 pt-6">
        <div className="mx-auto flex min-h-full w-full max-w-[1320px] flex-col gap-6">
          <section className="rounded-[32px] border border-white/40 bg-[rgba(255,255,255,0.58)] px-6 py-5 shadow-[0_24px_80px_rgba(112,130,178,0.12)] backdrop-blur-2xl">
            <div className="flex flex-wrap items-center justify-between gap-4">
              <div className="space-y-2">
                <div className="inline-flex items-center gap-2 rounded-full border border-white/45 bg-white/60 px-3 py-1 text-xs font-medium text-[var(--app-muted)]">
                  <Sparkles size={14} className="text-[var(--app-blue)]" />
                  Less is More
                </div>
                <h1 className="text-[clamp(2rem,4.6vw,3.75rem)] font-semibold tracking-[-0.06em] text-slate-900">
                  把系统字体换成你真正想看的样子
                </h1>
                <p className="max-w-3xl text-[15px] leading-7 text-[var(--app-muted)]">
                  不再堆技术名词，不再给你一坨下拉框。选一个你喜欢的风格，剩下的脏活累活交给底层静默完成。
                </p>
              </div>

              <div className="flex flex-wrap justify-end gap-2">
                <StatusPill icon={<MonitorSmartphone size={14} />} label={bootstrap?.display.resolutionLabel ?? "读取中"} />
                <StatusPill icon={<WandSparkles size={14} />} label={bootstrap?.display.matrixProfile ?? "自适应矩阵"} />
                <StatusPill
                  icon={<ShieldCheck size={14} />}
                  label={bootstrap?.isAdmin ? "管理员已就绪" : "等待管理员权限"}
                  tone={bootstrap?.isAdmin ? "green" : "amber"}
                />
                <StatusPill icon={<RefreshCw size={14} />} label={`${bootstrap?.backupCount ?? 0} 份后悔药`} />
              </div>
            </div>
          </section>

          <section className="relative flex min-h-[720px] items-center justify-center overflow-hidden rounded-[36px] border border-white/35 bg-[rgba(255,255,255,0.58)] px-4 py-6 shadow-[0_28px_90px_rgba(112,130,178,0.18)] backdrop-blur-[28px]">
            <div className="pointer-events-none absolute inset-0">
              <div className="absolute left-8 top-8 h-48 w-48 rounded-full bg-[#77a7ff]/16 blur-3xl" />
              <div className="absolute right-10 top-10 h-64 w-64 rounded-full bg-[#86c8ff]/18 blur-3xl" />
              <div className="absolute bottom-8 left-1/3 h-36 w-36 rounded-full bg-[#9db1ff]/18 blur-3xl" />
            </div>

            <button
              className="absolute left-6 top-1/2 z-20 grid h-12 w-12 -translate-y-1/2 place-items-center rounded-full border border-white/50 bg-white/70 text-slate-700 shadow-[0_10px_30px_rgba(89,104,138,0.16)] transition hover:-translate-y-[52%] hover:bg-white disabled:cursor-not-allowed disabled:opacity-40"
              disabled={!canGoPrev}
              onClick={() => canGoPrev && setCurrentIndex((value) => value - 1)}
              type="button"
            >
              <ArrowLeft size={18} />
            </button>

            <div className="relative z-10 flex w-full max-w-[860px] flex-col items-center gap-5">
              <div className="flex items-center gap-2 rounded-full border border-white/45 bg-white/60 px-3 py-1.5 text-xs font-medium text-[var(--app-muted)]">
                <Crown size={14} className="text-[var(--app-blue)]" />
                当前正在看的风格
              </div>

              <AnimatePresence mode="wait">
                <motion.article
                  key={currentPreset.id}
                  {...cardMotion}
                  className="relative w-full overflow-hidden rounded-[34px] border border-white/45 bg-[rgba(255,255,255,0.78)] p-7 shadow-[0_24px_70px_rgba(70,98,150,0.16)]"
                >
                  <div className={`pointer-events-none absolute inset-x-0 top-0 h-48 bg-gradient-to-br ${currentPreset.accentClass}`} />
                  <div className="relative z-10 grid gap-7 lg:grid-cols-[1.28fr_0.9fr]">
                    <div className="space-y-6">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="rounded-full border border-white/45 bg-white/72 px-3 py-1 text-xs font-semibold tracking-[0.18em] text-slate-700">
                          {currentPreset.tag}
                        </span>
                        <span className="rounded-full bg-slate-900/6 px-3 py-1 text-xs font-medium text-slate-600">{currentPreset.vibe}</span>
                        {currentPreset.current ? (
                          <span className="rounded-full bg-emerald-500/12 px-3 py-1 text-xs font-semibold text-emerald-700">
                            当前正在使用
                          </span>
                        ) : null}
                        {currentPreset.status === "soon" ? (
                          <span className="rounded-full bg-amber-500/12 px-3 py-1 text-xs font-semibold text-amber-700">
                            筹备中
                          </span>
                        ) : null}
                      </div>

                      <div className="space-y-3">
                        <h2 className="text-[clamp(2.2rem,4vw,3.5rem)] font-semibold tracking-[-0.06em] text-slate-950">
                          {currentPreset.name}
                        </h2>
                        <p className="max-w-xl text-[15px] leading-7 text-[var(--app-muted)]">{currentPreset.description}</p>
                      </div>

                      <div className="rounded-[28px] border border-white/50 bg-white/64 px-6 py-5 shadow-[inset_0_1px_0_rgba(255,255,255,0.7)]">
                        <div className="space-y-3" style={{ fontFamily: currentPreset.previewFont }}>
                          <p className="text-[clamp(2rem,3vw,3rem)] font-semibold tracking-[-0.05em] text-slate-950">
                            {currentPreset.chinesePreview}
                          </p>
                          <p className="text-lg leading-8 text-slate-600">{currentPreset.englishPreview}</p>
                        </div>
                      </div>

                      <div className="grid gap-3 rounded-[26px] border border-white/45 bg-white/52 p-4 sm:grid-cols-3">
                        <AddonSwitch
                          checked={addons.cursor}
                          label="高精度指针"
                          onChange={() => setAddons((state) => ({ ...state, cursor: !state.cursor }))}
                        />
                        <AddonSwitch
                          checked={addons.terminal}
                          label="现代终端字体"
                          onChange={() => setAddons((state) => ({ ...state, terminal: !state.terminal }))}
                        />
                        <AddonSwitch
                          checked={addons.emoji}
                          label="Modern Emoji"
                          onChange={() => setAddons((state) => ({ ...state, emoji: !state.emoji }))}
                        />
                      </div>
                    </div>

                    <div className="flex flex-col justify-between gap-6">
                      <div className="space-y-4 rounded-[28px] border border-white/45 bg-white/68 p-5">
                        <div className="space-y-1">
                          <div className="text-xs font-semibold uppercase tracking-[0.24em] text-[var(--app-muted)]">当前状态</div>
                          <div className="text-[28px] font-semibold tracking-[-0.05em] text-slate-950">
                            {currentPreset.current
                              ? "已经在用"
                              : currentPreset.status === "soon"
                                ? "正在打磨"
                                : currentPreset.available
                                  ? "可以直接上"
                                  : "等字体包就位"}
                          </div>
                          <p className="text-sm leading-7 text-[var(--app-muted)]">{currentPreset.recommendedFor}</p>
                        </div>

                        <div className="space-y-2 text-sm text-slate-600">
                          <MetaRow label="当前识别到的系统风格" value={bootstrap?.activeFontLabel ?? "Windows 默认"} />
                          <MetaRow label="屏幕匹配矩阵" value={bootstrap?.display.matrixProfile ?? "自适应中"} />
                          <MetaRow label="备份位置" value={bootstrap?.backupCount ? `${bootstrap.backupCount} 份快照已就绪` : "还没写入任何修改"} />
                        </div>
                      </div>

                      <div className="space-y-3">
                        <button
                          className="flex h-14 w-full items-center justify-center gap-2 rounded-full bg-[linear-gradient(135deg,#2f6df6_0%,#5d8fff_100%)] px-6 text-base font-semibold text-white shadow-[0_18px_36px_rgba(47,109,246,0.28)] transition hover:-translate-y-0.5 hover:shadow-[0_22px_40px_rgba(47,109,246,0.32)] disabled:cursor-not-allowed disabled:bg-slate-200 disabled:text-slate-500 disabled:shadow-none"
                          disabled={!canApply}
                          onClick={handleApply}
                          type="button"
                        >
                          {isBusy ? <RefreshCw size={18} className="animate-spin" /> : <CheckCheck size={18} />}
                          {resolveApplyLabel(currentPreset, bootstrap?.isAdmin ?? false)}
                        </button>

                        <div className="grid gap-3 sm:grid-cols-2">
                          <SoftButton onClick={() => showComingSoon("高级自定义导入")} icon={<DownloadCloud size={16} />}>
                            自定义导入
                          </SoftButton>
                          <SoftButton
                            onClick={() => openUrl("https://github.com/soberbw-hash/WindowsFontTuner/releases")}
                            icon={<ArrowRight size={16} />}
                          >
                            看发布页
                          </SoftButton>
                        </div>
                      </div>
                    </div>
                  </div>
                </motion.article>
              </AnimatePresence>

              <div className="flex items-center gap-2">
                {presets.map((preset, index) => (
                  <button
                    key={preset.id}
                    className={`h-2.5 rounded-full transition-all ${
                      index === currentIndex ? "w-9 bg-slate-900" : "w-2.5 bg-slate-300 hover:bg-slate-400"
                    }`}
                    onClick={() => setCurrentIndex(index)}
                    type="button"
                  />
                ))}
              </div>
            </div>

            <button
              className="absolute right-6 top-1/2 z-20 grid h-12 w-12 -translate-y-1/2 place-items-center rounded-full border border-white/50 bg-white/70 text-slate-700 shadow-[0_10px_30px_rgba(89,104,138,0.16)] transition hover:-translate-y-[52%] hover:bg-white disabled:cursor-not-allowed disabled:opacity-40"
              disabled={!canGoNext}
              onClick={() => canGoNext && setCurrentIndex((value) => value + 1)}
              type="button"
            >
              <ArrowRight size={18} />
            </button>
          </section>
        </div>
      </main>

      <div className="pointer-events-none absolute bottom-6 right-6 z-30">
        <div className="pointer-events-auto flex flex-col items-end gap-3">
          <AnimatePresence>
            {rescueOpen ? (
              <motion.div
                initial={{ opacity: 0, y: 16, scale: 0.96 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                exit={{ opacity: 0, y: 12, scale: 0.96 }}
                transition={{ duration: 0.2 }}
                className="w-[280px] rounded-[26px] border border-white/45 bg-white/78 p-3 shadow-[0_20px_60px_rgba(65,85,128,0.22)] backdrop-blur-2xl"
              >
                <button
                  className="flex w-full items-center gap-3 rounded-2xl px-4 py-3 text-left text-sm text-slate-700 transition hover:bg-slate-900/[0.04]"
                  onClick={handleRestoreDefaults}
                  type="button"
                >
                  <RotateCcw size={16} className="text-[var(--app-blue)]" />
                  恢复 Windows 默认字体
                </button>
                <button
                  className="flex w-full items-center gap-3 rounded-2xl px-4 py-3 text-left text-sm text-slate-700 transition hover:bg-slate-900/[0.04]"
                  onClick={handleRepairFonts}
                  type="button"
                >
                  <Wrench size={16} className="text-[var(--app-blue)]" />
                  修复系统字体文件
                </button>
                <button
                  className="flex w-full items-center gap-3 rounded-2xl px-4 py-3 text-left text-sm text-slate-700 transition hover:bg-slate-900/[0.04]"
                  onClick={() => {
                    setRescueOpen(false);
                    showComingSoon("高级自定义导入");
                  }}
                  type="button"
                >
                  <DownloadCloud size={16} className="text-[var(--app-blue)]" />
                  高级自定义导入
                </button>
              </motion.div>
            ) : null}
          </AnimatePresence>

          <button
            className="grid h-12 w-12 place-items-center rounded-full border border-white/50 bg-white/55 text-slate-700 shadow-[0_18px_42px_rgba(80,101,144,0.22)] backdrop-blur-2xl transition hover:bg-white/78"
            onClick={() => setRescueOpen((value) => !value)}
            type="button"
          >
            <Settings2 size={18} />
          </button>
        </div>
      </div>

      <AnimatePresence>
        {toast ? (
          <motion.div
            initial={{ opacity: 0, y: -18, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -12, scale: 0.98 }}
            transition={{ duration: 0.2 }}
            className="pointer-events-none absolute right-6 top-20 z-40 max-w-[420px]"
          >
            <div
              className={`rounded-[22px] border px-4 py-3 shadow-[0_16px_40px_rgba(72,96,143,0.18)] backdrop-blur-xl ${
                toast.tone === "success"
                  ? "border-emerald-200 bg-emerald-50/90 text-emerald-800"
                  : toast.tone === "warning"
                    ? "border-amber-200 bg-amber-50/92 text-amber-800"
                    : "border-sky-200 bg-sky-50/92 text-sky-800"
              }`}
            >
              <div className="text-sm leading-7">{toast.message}</div>
            </div>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}

function WindowButton(props: {
  icon: ReactNode;
  danger?: boolean;
  onClick: () => Promise<void> | void;
}) {
  return (
    <button
      className={`grid h-9 w-9 place-items-center rounded-full transition ${
        props.danger
          ? "bg-white/55 text-slate-700 hover:bg-[#ef4444] hover:text-white"
          : "bg-white/55 text-slate-700 hover:bg-white/85"
      }`}
      onClick={props.onClick}
      type="button"
    >
      {props.icon}
    </button>
  );
}

function StatusPill(props: { icon: ReactNode; label: string; tone?: "green" | "amber" }) {
  const toneClass =
    props.tone === "green"
      ? "text-emerald-700"
      : props.tone === "amber"
        ? "text-amber-700"
        : "text-[var(--app-muted)]";

  return (
    <div className={`inline-flex items-center gap-2 rounded-full border border-white/45 bg-white/60 px-3 py-1.5 text-xs font-medium ${toneClass}`}>
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
    <button
      className="flex h-12 items-center justify-center gap-2 rounded-full border border-white/55 bg-white/62 px-4 text-sm font-medium text-slate-700 transition hover:bg-white/82"
      onClick={props.onClick}
      type="button"
    >
      {props.icon}
      {props.children}
    </button>
  );
}

function AddonSwitch(props: { checked: boolean; label: string; onChange: () => void }) {
  return (
    <button
      className={`flex items-center justify-between rounded-[22px] border px-4 py-3 text-left transition ${
        props.checked
          ? "border-[#6b80ff]/30 bg-[#eff3ff] text-slate-800"
          : "border-white/50 bg-white/72 text-slate-600 hover:bg-white"
      }`}
      onClick={props.onChange}
      type="button"
    >
      <div>
        <div className="text-sm font-medium">{props.label}</div>
        <div className="text-xs text-slate-500">这一步会在核心字体链路稳定后接入</div>
      </div>
      <div className={`relative h-6 w-11 rounded-full transition ${props.checked ? "bg-[#2f6df6]" : "bg-slate-300"}`}>
        <span className={`absolute top-1 h-4 w-4 rounded-full bg-white transition ${props.checked ? "left-6" : "left-1"}`} />
      </div>
    </button>
  );
}

function MetaRow(props: { label: string; value: string }) {
  return (
    <div className="flex items-start justify-between gap-4">
      <span className="shrink-0 text-[13px] text-slate-500">{props.label}</span>
      <span className="text-right text-[13px] font-medium text-slate-700">{props.value}</span>
    </div>
  );
}

function resolveApplyLabel(preset: DisplayPreset, isAdmin: boolean) {
  if (!isAdmin) {
    return "请以管理员启动";
  }
  if (preset.status === "soon") {
    return "即将支持";
  }
  if (!preset.available) {
    return "字体包准备中";
  }
  if (preset.current) {
    return "当前正在使用";
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
