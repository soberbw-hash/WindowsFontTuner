use crate::models::{
    DesktopTextSettings, FontDownload, InstallMode, PresetDefinition, PresetStatus, RenderingSettings,
    RiskLevel,
};

const DESKTOP_SETTINGS: DesktopTextSettings = DesktopTextSettings {
    font_smoothing: "2",
    font_smoothing_type: 1,
    font_smoothing_gamma: 1850,
    font_smoothing_orientation: 1,
};

const RENDERING_SETTINGS: RenderingSettings = RenderingSettings {
    pixel_structure: 0,
    gamma_level: 1850,
    clear_type_level: 0,
    text_contrast_level: 5,
};

const HARMONY_DOWNLOADS: &[FontDownload] = &[
    FontDownload {
        file_name: "HarmonyOS_Sans_SC_Regular.ttf",
        url: "https://raw.githubusercontent.com/soberbw-hash/WindowsFontTuner/main/FontPackages/harmonyos-sc/HarmonyOS_Sans_SC_Regular.ttf",
    },
    FontDownload {
        file_name: "HarmonyOS_Sans_SC_Medium.ttf",
        url: "https://raw.githubusercontent.com/soberbw-hash/WindowsFontTuner/main/FontPackages/harmonyos-sc/HarmonyOS_Sans_SC_Medium.ttf",
    },
];

const SARASA_DOWNLOADS: &[FontDownload] = &[
    FontDownload {
        file_name: "SarasaUiSC-Regular.ttf",
        url: "https://raw.githubusercontent.com/soberbw-hash/WindowsFontTuner/main/FontPackages/sarasa-ui-sc/SarasaUiSC-Regular.ttf",
    },
    FontDownload {
        file_name: "SarasaUiSC-SemiBold.ttf",
        url: "https://raw.githubusercontent.com/soberbw-hash/WindowsFontTuner/main/FontPackages/sarasa-ui-sc/SarasaUiSC-SemiBold.ttf",
    },
];

const SOURCE_HAN_DOWNLOADS: &[FontDownload] = &[
    FontDownload {
        file_name: "SourceHanSansCN-Regular.otf",
        url: "https://raw.githubusercontent.com/soberbw-hash/WindowsFontTuner/main/FontPackages/source-han-sans-cn/SourceHanSansCN-Regular.otf",
    },
    FontDownload {
        file_name: "SourceHanSansCN-Medium.otf",
        url: "https://raw.githubusercontent.com/soberbw-hash/WindowsFontTuner/main/FontPackages/source-han-sans-cn/SourceHanSansCN-Medium.otf",
    },
];

const LXGW_DOWNLOADS: &[FontDownload] = &[FontDownload {
    file_name: "LXGWWenKai-Regular.ttf",
    url: "https://github.com/lxgw/LxgwWenKai/releases/download/v1.522/LXGWWenKai-Regular.ttf",
}];

const INTER_HARMONY_DOWNLOADS: &[FontDownload] = &[
    FontDownload {
        file_name: "InterVariable.ttf",
        url: "https://raw.githubusercontent.com/rsms/inter/v4.1/docs/font-files/InterVariable.ttf",
    },
    FontDownload {
        file_name: "HarmonyOS_Sans_SC_Regular.ttf",
        url: "https://raw.githubusercontent.com/soberbw-hash/WindowsFontTuner/main/FontPackages/harmonyos-sc/HarmonyOS_Sans_SC_Regular.ttf",
    },
    FontDownload {
        file_name: "HarmonyOS_Sans_SC_Medium.ttf",
        url: "https://raw.githubusercontent.com/soberbw-hash/WindowsFontTuner/main/FontPackages/harmonyos-sc/HarmonyOS_Sans_SC_Medium.ttf",
    },
];

const HARMONY_SUBSTITUTES: &[(&str, &str)] = &[
    ("Segoe UI", "HarmonyOS Sans SC Medium"),
    ("Segoe UI Light", "HarmonyOS Sans SC"),
    ("Segoe UI Semilight", "HarmonyOS Sans SC"),
    ("Segoe UI Semibold", "HarmonyOS Sans SC Medium"),
    ("Segoe UI Black", "HarmonyOS Sans SC Medium"),
    ("Segoe UI Variable", "HarmonyOS Sans SC Medium"),
    ("Segoe UI Variable Text", "HarmonyOS Sans SC Medium"),
    ("Segoe UI Variable Text Light", "HarmonyOS Sans SC"),
    ("Segoe UI Variable Text Semibold", "HarmonyOS Sans SC Medium"),
    ("Segoe UI Variable Display", "HarmonyOS Sans SC Medium"),
    ("Segoe UI Variable Display Light", "HarmonyOS Sans SC"),
    ("Segoe UI Variable Display Semibold", "HarmonyOS Sans SC Medium"),
    ("Segoe UI Variable Small", "HarmonyOS Sans SC Medium"),
    ("Segoe UI Variable Small Light", "HarmonyOS Sans SC"),
    ("Segoe UI Variable Small Semibold", "HarmonyOS Sans SC Medium"),
    ("Microsoft YaHei", "HarmonyOS Sans SC Medium"),
    ("Microsoft YaHei UI", "HarmonyOS Sans SC Medium"),
    ("Microsoft YaHei Light", "HarmonyOS Sans SC"),
    ("Microsoft YaHei UI Light", "HarmonyOS Sans SC"),
    ("Microsoft YaHei Semibold", "HarmonyOS Sans SC Medium"),
    ("Microsoft YaHei UI Semibold", "HarmonyOS Sans SC Medium"),
];

const SARASA_SUBSTITUTES: &[(&str, &str)] = &[
    ("Segoe UI", "Sarasa UI SC SemiBold"),
    ("Segoe UI Light", "Sarasa UI SC"),
    ("Segoe UI Semilight", "Sarasa UI SC"),
    ("Segoe UI Semibold", "Sarasa UI SC SemiBold"),
    ("Segoe UI Black", "Sarasa UI SC SemiBold"),
    ("Segoe UI Variable", "Sarasa UI SC SemiBold"),
    ("Segoe UI Variable Text", "Sarasa UI SC SemiBold"),
    ("Segoe UI Variable Text Light", "Sarasa UI SC"),
    ("Segoe UI Variable Text Semibold", "Sarasa UI SC SemiBold"),
    ("Segoe UI Variable Display", "Sarasa UI SC SemiBold"),
    ("Segoe UI Variable Display Light", "Sarasa UI SC"),
    ("Segoe UI Variable Display Semibold", "Sarasa UI SC SemiBold"),
    ("Segoe UI Variable Small", "Sarasa UI SC SemiBold"),
    ("Segoe UI Variable Small Light", "Sarasa UI SC"),
    ("Segoe UI Variable Small Semibold", "Sarasa UI SC SemiBold"),
    ("Microsoft YaHei", "Sarasa UI SC SemiBold"),
    ("Microsoft YaHei UI", "Sarasa UI SC SemiBold"),
    ("Microsoft YaHei Light", "Sarasa UI SC"),
    ("Microsoft YaHei UI Light", "Sarasa UI SC"),
    ("Microsoft YaHei Semibold", "Sarasa UI SC SemiBold"),
    ("Microsoft YaHei UI Semibold", "Sarasa UI SC SemiBold"),
];

const SOURCE_HAN_SUBSTITUTES: &[(&str, &str)] = &[
    ("Segoe UI", "Source Han Sans CN Medium"),
    ("Segoe UI Light", "Source Han Sans CN"),
    ("Segoe UI Semilight", "Source Han Sans CN"),
    ("Segoe UI Semibold", "Source Han Sans CN Medium"),
    ("Segoe UI Black", "Source Han Sans CN Medium"),
    ("Segoe UI Variable", "Source Han Sans CN Medium"),
    ("Segoe UI Variable Text", "Source Han Sans CN Medium"),
    ("Segoe UI Variable Text Light", "Source Han Sans CN"),
    ("Segoe UI Variable Text Semibold", "Source Han Sans CN Medium"),
    ("Segoe UI Variable Display", "Source Han Sans CN Medium"),
    ("Segoe UI Variable Display Light", "Source Han Sans CN"),
    ("Segoe UI Variable Display Semibold", "Source Han Sans CN Medium"),
    ("Segoe UI Variable Small", "Source Han Sans CN Medium"),
    ("Segoe UI Variable Small Light", "Source Han Sans CN"),
    ("Segoe UI Variable Small Semibold", "Source Han Sans CN Medium"),
    ("Microsoft YaHei", "Source Han Sans CN Medium"),
    ("Microsoft YaHei UI", "Source Han Sans CN Medium"),
    ("Microsoft YaHei Light", "Source Han Sans CN"),
    ("Microsoft YaHei UI Light", "Source Han Sans CN"),
    ("Microsoft YaHei Semibold", "Source Han Sans CN Medium"),
    ("Microsoft YaHei UI Semibold", "Source Han Sans CN Medium"),
];

const LXGW_SUBSTITUTES: &[(&str, &str)] = &[
    ("Segoe UI", "LXGW WenKai"),
    ("Segoe UI Light", "LXGW WenKai"),
    ("Segoe UI Semilight", "LXGW WenKai"),
    ("Segoe UI Semibold", "LXGW WenKai"),
    ("Segoe UI Black", "LXGW WenKai"),
    ("Segoe UI Variable", "LXGW WenKai"),
    ("Segoe UI Variable Text", "LXGW WenKai"),
    ("Segoe UI Variable Text Light", "LXGW WenKai"),
    ("Segoe UI Variable Text Semibold", "LXGW WenKai"),
    ("Segoe UI Variable Display", "LXGW WenKai"),
    ("Segoe UI Variable Display Light", "LXGW WenKai"),
    ("Segoe UI Variable Display Semibold", "LXGW WenKai"),
    ("Segoe UI Variable Small", "LXGW WenKai"),
    ("Segoe UI Variable Small Light", "LXGW WenKai"),
    ("Segoe UI Variable Small Semibold", "LXGW WenKai"),
    ("Microsoft YaHei", "LXGW WenKai"),
    ("Microsoft YaHei UI", "LXGW WenKai"),
    ("Microsoft YaHei Light", "LXGW WenKai"),
    ("Microsoft YaHei UI Light", "LXGW WenKai"),
    ("Microsoft YaHei Semibold", "LXGW WenKai"),
    ("Microsoft YaHei UI Semibold", "LXGW WenKai"),
];

const OPPOSANS_SUBSTITUTES: &[(&str, &str)] = &[
    ("Segoe UI", "OPPOSans 3.0"),
    ("Segoe UI Light", "OPPOSans 3.0"),
    ("Segoe UI Semilight", "OPPOSans 3.0"),
    ("Segoe UI Semibold", "OPPOSans 3.0"),
    ("Segoe UI Black", "OPPOSans 3.0"),
    ("Segoe UI Variable", "OPPOSans 3.0"),
    ("Segoe UI Variable Text", "OPPOSans 3.0"),
    ("Segoe UI Variable Text Light", "OPPOSans 3.0"),
    ("Segoe UI Variable Text Semibold", "OPPOSans 3.0"),
    ("Segoe UI Variable Display", "OPPOSans 3.0"),
    ("Segoe UI Variable Display Light", "OPPOSans 3.0"),
    ("Segoe UI Variable Display Semibold", "OPPOSans 3.0"),
    ("Segoe UI Variable Small", "OPPOSans 3.0"),
    ("Segoe UI Variable Small Light", "OPPOSans 3.0"),
    ("Segoe UI Variable Small Semibold", "OPPOSans 3.0"),
    ("Microsoft YaHei", "OPPOSans 3.0"),
    ("Microsoft YaHei UI", "OPPOSans 3.0"),
    ("Microsoft YaHei Light", "OPPOSans 3.0"),
    ("Microsoft YaHei UI Light", "OPPOSans 3.0"),
    ("Microsoft YaHei Semibold", "OPPOSans 3.0"),
    ("Microsoft YaHei UI Semibold", "OPPOSans 3.0"),
];

const INTER_HARMONY_SUBSTITUTES: &[(&str, &str)] = &[
    ("Segoe UI", "Inter"),
    ("Segoe UI Light", "Inter"),
    ("Segoe UI Semilight", "Inter"),
    ("Segoe UI Semibold", "Inter"),
    ("Segoe UI Black", "Inter"),
    ("Segoe UI Variable", "Inter"),
    ("Segoe UI Variable Text", "Inter"),
    ("Segoe UI Variable Text Light", "Inter"),
    ("Segoe UI Variable Text Semibold", "Inter"),
    ("Segoe UI Variable Display", "Inter"),
    ("Segoe UI Variable Display Light", "Inter"),
    ("Segoe UI Variable Display Semibold", "Inter"),
    ("Segoe UI Variable Small", "Inter"),
    ("Segoe UI Variable Small Light", "Inter"),
    ("Segoe UI Variable Small Semibold", "Inter"),
    ("Microsoft YaHei", "HarmonyOS Sans SC Medium"),
    ("Microsoft YaHei UI", "HarmonyOS Sans SC Medium"),
    ("Microsoft YaHei Light", "HarmonyOS Sans SC"),
    ("Microsoft YaHei UI Light", "HarmonyOS Sans SC"),
    ("Microsoft YaHei Semibold", "HarmonyOS Sans SC Medium"),
    ("Microsoft YaHei UI Semibold", "HarmonyOS Sans SC Medium"),
];

pub fn preset_definitions() -> Vec<PresetDefinition> {
    vec![
        PresetDefinition {
            id: "harmonyos-sc",
            status: PresetStatus::Ready,
            install_mode: InstallMode::AutoDownload,
            font_family: "HarmonyOS Sans SC",
            required_fonts: &["HarmonyOS Sans SC", "HarmonyOS Sans SC Medium"],
            fallback_families: &["HarmonyOS Sans SC", "Microsoft YaHei", "Segoe UI Emoji"],
            downloads: HARMONY_DOWNLOADS,
            font_substitutes: HARMONY_SUBSTITUTES,
            desktop: DESKTOP_SETTINGS,
            rendering: RENDERING_SETTINGS,
            risk_level: RiskLevel::Low,
        },
        PresetDefinition {
            id: "sarasa-ui-sc",
            status: PresetStatus::Ready,
            install_mode: InstallMode::AutoDownload,
            font_family: "Sarasa UI SC",
            required_fonts: &["Sarasa UI SC", "Sarasa UI SC SemiBold"],
            fallback_families: &["Sarasa UI SC", "Microsoft YaHei", "Segoe UI Emoji"],
            downloads: SARASA_DOWNLOADS,
            font_substitutes: SARASA_SUBSTITUTES,
            desktop: DESKTOP_SETTINGS,
            rendering: RENDERING_SETTINGS,
            risk_level: RiskLevel::Medium,
        },
        PresetDefinition {
            id: "source-han-sans-cn",
            status: PresetStatus::Ready,
            install_mode: InstallMode::AutoDownload,
            font_family: "Source Han Sans CN",
            required_fonts: &["Source Han Sans CN", "Source Han Sans CN Medium"],
            fallback_families: &["Source Han Sans CN", "Microsoft YaHei", "Segoe UI Emoji"],
            downloads: SOURCE_HAN_DOWNLOADS,
            font_substitutes: SOURCE_HAN_SUBSTITUTES,
            desktop: DESKTOP_SETTINGS,
            rendering: RENDERING_SETTINGS,
            risk_level: RiskLevel::Low,
        },
        PresetDefinition {
            id: "lxgw-wenkai",
            status: PresetStatus::Ready,
            install_mode: InstallMode::AutoDownload,
            font_family: "LXGW WenKai",
            required_fonts: &["LXGW WenKai"],
            fallback_families: &["LXGW WenKai", "Microsoft YaHei", "Segoe UI Emoji"],
            downloads: LXGW_DOWNLOADS,
            font_substitutes: LXGW_SUBSTITUTES,
            desktop: DESKTOP_SETTINGS,
            rendering: RENDERING_SETTINGS,
            risk_level: RiskLevel::Medium,
        },
        PresetDefinition {
            id: "opposans",
            status: PresetStatus::Ready,
            install_mode: InstallMode::ManualImport,
            font_family: "OPPOSans 3.0",
            required_fonts: &["OPPOSans 3.0"],
            fallback_families: &["OPPOSans 3.0", "Microsoft YaHei", "Segoe UI Emoji"],
            downloads: &[],
            font_substitutes: OPPOSANS_SUBSTITUTES,
            desktop: DESKTOP_SETTINGS,
            rendering: RENDERING_SETTINGS,
            risk_level: RiskLevel::High,
        },
        PresetDefinition {
            id: "inter-harmonyos",
            status: PresetStatus::Ready,
            install_mode: InstallMode::AutoDownload,
            font_family: "Inter + HarmonyOS",
            required_fonts: &["Inter", "HarmonyOS Sans SC", "HarmonyOS Sans SC Medium"],
            fallback_families: &["HarmonyOS Sans SC", "Microsoft YaHei", "Segoe UI Emoji"],
            downloads: INTER_HARMONY_DOWNLOADS,
            font_substitutes: INTER_HARMONY_SUBSTITUTES,
            desktop: DESKTOP_SETTINGS,
            rendering: RENDERING_SETTINGS,
            risk_level: RiskLevel::Low,
        },
    ]
}
