use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum InstallMode {
    AutoDownload,
    ManualImport,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PresetStatus {
    Ready,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Risk,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RenderStyleId {
    Clear,
    Balanced,
    Soft,
    Reading,
    Code,
    Rounded,
}

impl RenderStyleId {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "clear" => Some(Self::Clear),
            "balanced" => Some(Self::Balanced),
            "soft" => Some(Self::Soft),
            "reading" => Some(Self::Reading),
            "code" => Some(Self::Code),
            "rounded" => Some(Self::Rounded),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clear => "clear",
            Self::Balanced => "balanced",
            Self::Soft => "soft",
            Self::Reading => "reading",
            Self::Code => "code",
            Self::Rounded => "rounded",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Clear => "清晰",
            Self::Balanced => "平衡",
            Self::Soft => "柔和",
            Self::Reading => "阅读",
            Self::Code => "代码",
            Self::Rounded => "圆润观感",
        }
    }

    pub fn summary(self) -> &'static str {
        match self {
            Self::Clear => "边缘更利，适合低分屏和默认办公。",
            Self::Balanced => "默认推荐，兼顾清晰和舒适。",
            Self::Soft => "灰度更松，整体更柔和。",
            Self::Reading => "长文更耐看，刺感更低。",
            Self::Code => "小字号更稳，代码辨识更强。",
            Self::Rounded => "高分屏更圆润，观感更现代。",
        }
    }

    pub fn recommendation_hint(self) -> &'static str {
        match self {
            Self::Clear => "适合 1080p 和偏清晰取向。",
            Self::Balanced => "适合大多数 2K 桌面和长期默认。",
            Self::Soft => "适合高分屏和轻柔观感。",
            Self::Reading => "适合长文阅读和写作场景。",
            Self::Code => "适合开发者与高信息密度场景。",
            Self::Rounded => "适合 4K 高缩放和圆润观感。",
        }
    }
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
pub struct RenderStyleTemplate {
    pub id: RenderStyleId,
    pub desktop: DesktopTextSettings,
    pub rendering: RenderingSettings,
    pub gdi_bias: u8,
    pub directwrite_bias: u8,
    pub rendering_mode: &'static str,
    pub small_text_boost: f32,
    pub roundedness: f32,
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
    pub risk_level: RiskLevel,
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
    pub ppi_label: String,
    pub multi_monitor: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationPayload {
    pub title: String,
    pub summary: String,
    pub primary_preset_id: String,
    pub primary_render_style: RenderStyleId,
    pub alternates: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderStyleState {
    pub id: RenderStyleId,
    pub label: String,
    pub summary: String,
    pub recommended_for: String,
    pub current: bool,
    pub recommended: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryOverview {
    pub backup_count: usize,
    pub last_backup_label: String,
    pub last_applied_at: Option<String>,
    pub safe_mode_hint: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthItem {
    pub label: String,
    pub status: CheckStatus,
    pub detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FontHealthReport {
    pub overall_status: CheckStatus,
    pub summary: String,
    pub items: Vec<HealthItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplySummary {
    pub preset_id: String,
    pub render_style_id: RenderStyleId,
    pub preset_label: String,
    pub render_style_label: String,
    pub risk_level: RiskLevel,
    pub will_modify_font_substitutes: bool,
    pub will_modify_font_link: bool,
    pub will_write_rendering: bool,
    pub will_download_fonts: bool,
    pub requires_explorer_refresh: bool,
    pub recommend_sign_out: bool,
    pub health: FontHealthReport,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedPresetPayload {
    pub name: String,
    pub cn_font: String,
    pub en_font: Option<String>,
    pub render_style_id: RenderStyleId,
    pub risk_level: RiskLevel,
    pub tags: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedPresetFile {
    pub id: Option<String>,
    pub name: String,
    pub cn_font: String,
    pub en_font: Option<String>,
    pub render_style: Option<String>,
    pub screen_profile: Option<String>,
    pub risk_level: Option<RiskLevel>,
    pub tags: Option<Vec<String>>,
    pub font_link_fallbacks: Option<Vec<String>>,
    pub created_at: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub is_admin: bool,
    pub active_preset_id: Option<String>,
    pub active_font_label: String,
    pub active_render_style_id: RenderStyleId,
    pub current_state_label: String,
    pub last_modified_label: String,
    pub backup_count: usize,
    pub backup_dir: String,
    pub display: DisplayProfile,
    pub recommendation: RecommendationPayload,
    pub render_styles: Vec<RenderStyleState>,
    pub recovery: RecoveryOverview,
    pub presets: Vec<RuntimePresetState>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    pub message: String,
    pub backup_path: Option<String>,
    pub active_preset_id: Option<String>,
    pub active_render_style_id: Option<RenderStyleId>,
    pub export_path: Option<String>,
}
