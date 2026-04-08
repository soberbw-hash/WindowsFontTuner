import { invoke } from "@tauri-apps/api/core";
import type {
  ActionResult,
  ApplySummary,
  BootstrapPayload,
  ImportedPresetPayload,
  RenderStyleId,
} from "../types";

export async function loadBootstrap() {
  return invoke<BootstrapPayload>("load_bootstrap");
}

export async function getApplySummary(presetId: string, renderStyleId: RenderStyleId) {
  return invoke<ApplySummary>("get_apply_summary", { presetId, renderStyleId });
}

export async function applyPreset(presetId: string, renderStyleId: RenderStyleId) {
  return invoke<ActionResult>("apply_preset", { presetId, renderStyleId });
}

export async function importFontFiles(paths: string[]) {
  return invoke<ActionResult>("import_font_files", { paths });
}

export async function restoreWindowsDefault() {
  return invoke<ActionResult>("restore_windows_default");
}

export async function runRecoveryAction(action: string) {
  return invoke<ActionResult>("run_recovery_action", { action });
}

export async function exportCurrentScheme(presetId: string, renderStyleId: RenderStyleId) {
  return invoke<ActionResult>("export_current_scheme", { presetId, renderStyleId });
}

export async function importSharedScheme(path: string) {
  return invoke<ImportedPresetPayload>("import_shared_scheme", { path });
}

export async function repairSystemFonts() {
  return invoke<void>("repair_system_fonts");
}
