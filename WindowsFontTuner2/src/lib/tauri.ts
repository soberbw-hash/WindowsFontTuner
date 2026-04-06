import { invoke } from "@tauri-apps/api/core";
import type { ActionResult, BootstrapPayload } from "../types";

export async function loadBootstrap() {
  return invoke<BootstrapPayload>("load_bootstrap");
}

export async function applyPreset(presetId: string) {
  return invoke<ActionResult>("apply_preset", { presetId });
}

export async function restoreWindowsDefault() {
  return invoke<ActionResult>("restore_windows_default");
}

export async function repairSystemFonts() {
  return invoke<void>("repair_system_fonts");
}
