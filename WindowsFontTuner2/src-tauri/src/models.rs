use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum InstallMode {
    AutoDownload,
    ManualImport,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PresetStatus {
    Ready,
}

#[derive(Clone, Copy, Debug)]
pub struct DesktopTextSettings {
    pub font_smoothing: &'static str,
    pub font_smoothing_type: u32,
    pub font_smoothing_gamma: u32,
    pub font_smoothing_orientation: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct RenderingSettings {
    pub pixel_structure: u32,
    pub gamma_level: u32,
    pub clear_type_level: u32,
    pub text_contrast_level: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct FontDownload {
    pub file_name: &'static str,
    pub url: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct PresetDefinition {
    pub id: &'static str,
    pub status: PresetStatus,
    pub install_mode: InstallMode,
    pub font_family: &'static str,
    pub required_fonts: &'static [&'static str],
    pub fallback_families: &'static [&'static str],
    pub downloads: &'static [FontDownload],
    pub font_substitutes: &'static [(&'static str, &'static str)],
    pub desktop: DesktopTextSettings,
    pub rendering: RenderingSettings,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePresetState {
    pub id: String,
    pub status: PresetStatus,
    pub install_mode: InstallMode,
    pub available: bool,
    pub current: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayProfile {
    pub width: i32,
    pub height: i32,
    pub scale_percent: u32,
    pub resolution_label: String,
    pub matrix_profile: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub is_admin: bool,
    pub active_preset_id: Option<String>,
    pub active_font_label: String,
    pub backup_count: usize,
    pub backup_dir: String,
    pub display: DisplayProfile,
    pub presets: Vec<RuntimePresetState>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    pub message: String,
    pub backup_path: Option<String>,
    pub active_preset_id: Option<String>,
}
