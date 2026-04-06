export type PresetStatus = "ready" | "soon";

export interface RuntimePresetState {
  id: string;
  status: PresetStatus;
  available: boolean;
  current: boolean;
}

export interface DisplayProfile {
  width: number;
  height: number;
  scalePercent: number;
  resolutionLabel: string;
  matrixProfile: string;
}

export interface BootstrapPayload {
  isAdmin: boolean;
  activePresetId: string | null;
  activeFontLabel: string;
  backupCount: number;
  backupDir: string;
  display: DisplayProfile;
  presets: RuntimePresetState[];
}

export interface ActionResult {
  message: string;
  backupPath?: string | null;
  activePresetId?: string | null;
}

export interface PresetMeta {
  id: string;
  name: string;
  tag: string;
  vibe: string;
  description: string;
  recommendedFor: string;
  chinesePreview: string;
  englishPreview: string;
  previewFont: string;
  accentClass: string;
  glowClass: string;
}

export interface DisplayPreset extends PresetMeta {
  status: PresetStatus;
  available: boolean;
  current: boolean;
}
