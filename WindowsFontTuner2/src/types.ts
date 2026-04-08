export type PresetStatus = "ready";
export type InstallMode = "autoDownload" | "manualImport";
export type RenderStyleId = "clear" | "balanced" | "soft" | "reading" | "code" | "rounded";
export type RiskLevel = "low" | "medium" | "high";
export type CheckStatus = "pass" | "warn" | "risk";

export interface RuntimePresetState {
  id: string;
  status: PresetStatus;
  installMode: InstallMode;
  available: boolean;
  current: boolean;
}

export interface DisplayProfile {
  width: number;
  height: number;
  scalePercent: number;
  resolutionLabel: string;
  matrixProfile: string;
  ppiLabel: string;
  multiMonitor: boolean;
}

export interface RecommendationPayload {
  title: string;
  summary: string;
  primaryPresetId: string;
  primaryRenderStyle: RenderStyleId;
  alternates: string[];
}

export interface RenderStyleState {
  id: RenderStyleId;
  label: string;
  summary: string;
  recommendedFor: string;
  current: boolean;
  recommended: boolean;
}

export interface RecoveryOverview {
  backupCount: number;
  lastBackupLabel: string;
  lastAppliedAt: string | null;
  safeModeHint: string;
}

export interface BootstrapPayload {
  isAdmin: boolean;
  activePresetId: string | null;
  activeFontLabel: string;
  activeRenderStyleId: RenderStyleId;
  currentStateLabel: string;
  lastModifiedLabel: string;
  backupCount: number;
  backupDir: string;
  display: DisplayProfile;
  recommendation: RecommendationPayload;
  renderStyles: RenderStyleState[];
  recovery: RecoveryOverview;
  presets: RuntimePresetState[];
}

export interface HealthItem {
  label: string;
  status: CheckStatus;
  detail: string;
}

export interface FontHealthReport {
  overallStatus: CheckStatus;
  summary: string;
  items: HealthItem[];
}

export interface ApplySummary {
  presetId: string;
  renderStyleId: RenderStyleId;
  presetLabel: string;
  renderStyleLabel: string;
  riskLevel: RiskLevel;
  willModifyFontSubstitutes: boolean;
  willModifyFontLink: boolean;
  willWriteRendering: boolean;
  willDownloadFonts: boolean;
  requiresExplorerRefresh: boolean;
  recommendSignOut: boolean;
  health: FontHealthReport;
}

export interface ImportedPresetPayload {
  name: string;
  cnFont: string;
  enFont: string | null;
  renderStyleId: RenderStyleId;
  riskLevel: RiskLevel;
  tags: string[];
  warnings: string[];
}

export interface ActionResult {
  message: string;
  backupPath?: string | null;
  activePresetId?: string | null;
  activeRenderStyleId?: RenderStyleId | null;
  exportPath?: string | null;
}

export interface PresetMeta {
  id: string;
  sceneName: string;
  fontFamily: string;
  shortTag: string;
  mood: string;
  description: string;
  recommendedFor: string;
  headline: string;
  englishLine: string;
  previewFont: string;
  accentClass: string;
  compatibility: "稳定" | "谨慎" | "实验";
  tags: string[];
  screens: string[];
}

export interface DisplayPreset extends PresetMeta {
  status: PresetStatus;
  installMode: InstallMode;
  available: boolean;
  current: boolean;
}

export type PreviewMode = "ui" | "reading" | "browser" | "code";
export type PreviewTheme = "light" | "dark";
export type PreviewScale = 100 | 125 | 150 | 200;
