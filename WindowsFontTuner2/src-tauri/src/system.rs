use crate::models::{
    ActionResult, ApplySummary, BootstrapPayload, CheckStatus, DesktopTextSettings, DisplayProfile,
    FontDownload, FontHealthReport, HealthItem, ImportedPresetFile, ImportedPresetPayload,
    InstallMode, PresetDefinition, PresetStatus, RecoveryOverview, RecommendationPayload,
    RenderStyleId, RenderStyleState, RenderStyleTemplate, RenderingSettings, RiskLevel,
    RuntimePresetState,
};
use anyhow::{anyhow, bail, Context, Result};
use chrono::Local;
use dirs::data_local_dir;
use reqwest::blocking::Client;
use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde_json::{from_str, to_string_pretty};
use ttf_parser::{name_id, Face};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{AddFontResourceExW, FONT_RESOURCE_CHARACTERISTICS};
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_SHIFT};
use windows::Win32::UI::Shell::{IsUserAnAdmin, ShellExecuteW};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, SM_CMONITORS,
    SM_CXSCREEN, SM_CYSCREEN, SW_SHOWNORMAL, WM_FONTCHANGE, WM_SETTINGCHANGE,
};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY, KEY_WRITE};
use winreg::RegKey;

const FONT_SUBSTITUTES_PATH: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\FontSubstitutes";
const FONTS_PATH: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts";
const FONT_LINK_PATH: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\FontLink\SystemLink";
const DESKTOP_PATH: &str = r"Control Panel\Desktop";
const AVALON_GRAPHICS_PATH: &str = r"Software\Microsoft\Avalon.Graphics";
const APP_STATE_PATH: &str = r"Software\WindowsFontTuner\State";
const MANAGED_FONT_SUBSTITUTE_NAMES: &[&str] = &[
    "Segoe UI",
    "Segoe UI Light",
    "Segoe UI Semilight",
    "Segoe UI Semibold",
    "Segoe UI Black",
    "Segoe UI Variable",
    "Segoe UI Variable Text",
    "Segoe UI Variable Text Light",
    "Segoe UI Variable Text Semibold",
    "Segoe UI Variable Display",
    "Segoe UI Variable Display Light",
    "Segoe UI Variable Display Semibold",
    "Segoe UI Variable Small",
    "Segoe UI Variable Small Light",
    "Segoe UI Variable Small Semibold",
    "Microsoft YaHei",
    "Microsoft YaHei UI",
    "Microsoft YaHei Light",
    "Microsoft YaHei UI Light",
    "Microsoft YaHei Semibold",
    "Microsoft YaHei UI Semibold",
];

#[derive(Clone, Debug)]
struct FontInstallRecord {
    registry_name: String,
    file_name: String,
}

#[derive(Clone, Debug, Default)]
struct AppStateSnapshot {
    active_preset_id: Option<String>,
    active_render_style_id: Option<RenderStyleId>,
    last_backup_path: Option<String>,
    last_applied_at: Option<String>,
}

pub fn should_run_headless_recovery() -> bool {
    env::args().any(|arg| arg == "--emergency-reset") || is_shift_pressed()
}

pub fn run_headless_recovery(presets: &[PresetDefinition]) -> Result<()> {
    if !is_admin() {
        run_self_elevated("--emergency-reset")?;
        return Ok(());
    }

    restore_windows_default_internal(presets)?;
    restart_explorer()?;
    Ok(())
}

pub fn render_style_templates() -> [RenderStyleTemplate; 6] {
    [
        RenderStyleTemplate {
            id: RenderStyleId::Clear,
            desktop: DesktopTextSettings {
                font_smoothing: "2",
                font_smoothing_type: 2,
                font_smoothing_gamma: 1980,
                font_smoothing_orientation: 1,
            },
            rendering: RenderingSettings {
                pixel_structure: 1,
                gamma_level: 2000,
                clear_type_level: 100,
                text_contrast_level: 6,
            },
            gdi_bias: 86,
            directwrite_bias: 62,
            rendering_mode: "cleartypeNatural",
            small_text_boost: 1.18,
            roundedness: 0.2,
        },
        RenderStyleTemplate {
            id: RenderStyleId::Balanced,
            desktop: DesktopTextSettings {
                font_smoothing: "2",
                font_smoothing_type: 1,
                font_smoothing_gamma: 1850,
                font_smoothing_orientation: 1,
            },
            rendering: RenderingSettings {
                pixel_structure: 0,
                gamma_level: 1850,
                clear_type_level: 0,
                text_contrast_level: 5,
            },
            gdi_bias: 64,
            directwrite_bias: 72,
            rendering_mode: "naturalBalanced",
            small_text_boost: 1.0,
            roundedness: 0.45,
        },
        RenderStyleTemplate {
            id: RenderStyleId::Soft,
            desktop: DesktopTextSettings {
                font_smoothing: "2",
                font_smoothing_type: 1,
                font_smoothing_gamma: 1780,
                font_smoothing_orientation: 1,
            },
            rendering: RenderingSettings {
                pixel_structure: 0,
                gamma_level: 1780,
                clear_type_level: 0,
                text_contrast_level: 4,
            },
            gdi_bias: 42,
            directwrite_bias: 80,
            rendering_mode: "grayscaleSoft",
            small_text_boost: 0.94,
            roundedness: 0.64,
        },
        RenderStyleTemplate {
            id: RenderStyleId::Reading,
            desktop: DesktopTextSettings {
                font_smoothing: "2",
                font_smoothing_type: 1,
                font_smoothing_gamma: 1825,
                font_smoothing_orientation: 1,
            },
            rendering: RenderingSettings {
                pixel_structure: 0,
                gamma_level: 1825,
                clear_type_level: 0,
                text_contrast_level: 3,
            },
            gdi_bias: 36,
            directwrite_bias: 78,
            rendering_mode: "readingGrayscale",
            small_text_boost: 0.92,
            roundedness: 0.68,
        },
        RenderStyleTemplate {
            id: RenderStyleId::Code,
            desktop: DesktopTextSettings {
                font_smoothing: "2",
                font_smoothing_type: 2,
                font_smoothing_gamma: 1940,
                font_smoothing_orientation: 1,
            },
            rendering: RenderingSettings {
                pixel_structure: 1,
                gamma_level: 1950,
                clear_type_level: 100,
                text_contrast_level: 6,
            },
            gdi_bias: 92,
            directwrite_bias: 66,
            rendering_mode: "monoSharp",
            small_text_boost: 1.24,
            roundedness: 0.16,
        },
        RenderStyleTemplate {
            id: RenderStyleId::Rounded,
            desktop: DesktopTextSettings {
                font_smoothing: "2",
                font_smoothing_type: 1,
                font_smoothing_gamma: 1750,
                font_smoothing_orientation: 1,
            },
            rendering: RenderingSettings {
                pixel_structure: 0,
                gamma_level: 1750,
                clear_type_level: 0,
                text_contrast_level: 4,
            },
            gdi_bias: 30,
            directwrite_bias: 84,
            rendering_mode: "roundedHighDpi",
            small_text_boost: 0.9,
            roundedness: 0.78,
        },
    ]
}

fn render_style_template(id: RenderStyleId) -> RenderStyleTemplate {
    render_style_templates()
        .into_iter()
        .find(|template| template.id == id)
        .unwrap_or_else(|| render_style_templates()[1])
}

fn recommend_profile(display: &DisplayProfile) -> RecommendationPayload {
    let (preset_id, render_style, title, summary, alternates) =
        if display.width >= 3800 || display.scale_percent >= 175 {
            (
                "inter-harmonyos",
                RenderStyleId::Rounded,
                "你的屏幕更适合：圆润观感",
                "高分屏会更适合轻柔、圆润的渲染，英文数字细节也更漂亮。",
                vec!["harmonyos-sc".to_string(), "source-han-sans-cn".to_string()],
            )
        } else if display.width >= 2500 || display.scale_percent >= 125 {
            (
                "harmonyos-sc",
                RenderStyleId::Balanced,
                "你的屏幕更适合：平衡",
                "这档兼顾清晰和松弛感，适合大多数 2K / 150% 的长期默认。",
                vec!["source-han-sans-cn".to_string(), "inter-harmonyos".to_string()],
            )
        } else {
            (
                "sarasa-ui-sc",
                RenderStyleId::Clear,
                "你的屏幕更适合：清晰",
                "低分屏和 100% 缩放更需要更利落的小字号策略。",
                vec!["harmonyos-sc".to_string(), "source-han-sans-cn".to_string()],
            )
        };

    RecommendationPayload {
        title: title.to_string(),
        summary: summary.to_string(),
        primary_preset_id: preset_id.to_string(),
        primary_render_style: render_style,
        alternates,
    }
}

fn render_style_states(active: RenderStyleId, recommended: RenderStyleId) -> Vec<RenderStyleState> {
    render_style_templates()
        .into_iter()
        .map(|template| RenderStyleState {
            id: template.id,
            label: template.id.label().to_string(),
            summary: template.id.summary().to_string(),
            recommended_for: template.id.recommendation_hint().to_string(),
            current: template.id == active,
            recommended: template.id == recommended,
        })
        .collect()
}

pub fn load_bootstrap(presets: &[PresetDefinition]) -> Result<BootstrapPayload> {
    let display = sniff_display_profile();
    let recommendation = recommend_profile(&display);
    let app_state = read_app_state()?;
    let active_preset_id = detect_active_preset_id(presets)?;
    let active_font_label = resolve_active_font_label(presets, active_preset_id.as_deref())?;
    let active_render_style_id = detect_active_render_style(app_state.active_render_style_id)?;
    let presets = presets
        .iter()
        .map(|preset| RuntimePresetState {
            id: preset.id.to_string(),
            status: preset.status,
            install_mode: preset.install_mode,
            available: preset.required_fonts.iter().all(|family| font_family_exists(family)),
            current: active_preset_id.as_deref() == Some(preset.id),
        })
        .collect::<Vec<_>>();

    let backup_dir = backup_root();
    Ok(BootstrapPayload {
        is_admin: is_admin(),
        active_preset_id,
        active_font_label,
        active_render_style_id,
        current_state_label: describe_current_state(&app_state),
        last_modified_label: describe_last_modified(&app_state),
        backup_count: list_backup_directories().len(),
        backup_dir: backup_dir.to_string_lossy().to_string(),
        display,
        recommendation: recommendation.clone(),
        render_styles: render_style_states(active_render_style_id, recommendation.primary_render_style),
        recovery: RecoveryOverview {
            backup_count: list_backup_directories().len(),
            last_backup_label: describe_last_backup_label(&app_state),
            last_applied_at: app_state.last_applied_at.clone(),
            safe_mode_hint: "按住 Shift 再双击启动，可直接进入后台急救恢复。".to_string(),
        },
        presets,
    })
}

pub fn apply_preset(
    preset_id: &str,
    render_style_id: RenderStyleId,
    presets: &[PresetDefinition],
) -> Result<ActionResult> {
    ensure_admin()?;

    let preset = presets
        .iter()
        .find(|preset| preset.id == preset_id)
        .copied()
        .ok_or_else(|| anyhow!("没找到这套风格。"))?;

    ensure_preset_assets(&preset)?;

    if !preset.required_fonts.iter().all(|family| font_family_exists(family)) {
        match preset.install_mode {
            InstallMode::AutoDownload => {
                bail!("字体还没准备好，下载似乎没有成功，请稍后再试。");
            }
            InstallMode::ManualImport => {
                bail!("这套风格需要你先导入字体文件，再继续应用。");
            }
        }
    }

    let backup_path = create_backup()?;
    let display = sniff_display_profile();
    write_font_substitutes(&preset)?;
    write_font_link_fallbacks(&preset)?;
    write_desktop_settings(resolve_desktop_settings(preset.desktop, render_style_id, &display))?;
    write_rendering_settings(resolve_rendering_settings(preset.rendering, render_style_id, &display))?;
    write_app_state(&AppStateSnapshot {
        active_preset_id: Some(preset.id.to_string()),
        active_render_style_id: Some(render_style_id),
        last_backup_path: Some(backup_path.to_string_lossy().to_string()),
        last_applied_at: Some(now_label()),
    })?;
    notify_system();
    let _ = restart_explorer();

    Ok(ActionResult {
        message: "✨ 风格已应用，注销电脑后即可感受。".to_string(),
        backup_path: Some(backup_path.to_string_lossy().to_string()),
        active_preset_id: Some(preset.id.to_string()),
        active_render_style_id: Some(render_style_id),
        export_path: None,
    })
}

pub fn build_apply_summary(
    preset_id: &str,
    render_style_id: RenderStyleId,
    presets: &[PresetDefinition],
) -> Result<ApplySummary> {
    let preset = presets
        .iter()
        .find(|preset| preset.id == preset_id)
        .copied()
        .ok_or_else(|| anyhow!("没找到这套风格。"))?;
    let health = evaluate_font_health(&preset, render_style_id);

    Ok(ApplySummary {
        preset_id: preset.id.to_string(),
        render_style_id,
        preset_label: preset.font_family.to_string(),
        render_style_label: render_style_id.label().to_string(),
        risk_level: preset.risk_level,
        will_modify_font_substitutes: true,
        will_modify_font_link: true,
        will_write_rendering: true,
        will_download_fonts: preset.install_mode == InstallMode::AutoDownload
            && !preset.required_fonts.iter().all(|family| font_family_exists(family)),
        requires_explorer_refresh: true,
        recommend_sign_out: true,
        health,
    })
}

pub fn import_font_files(paths: &[String]) -> Result<ActionResult> {
    if paths.is_empty() {
        bail!("这次没有选中任何字体文件。");
    }

    let mut imported = 0usize;
    for path in paths {
        install_font_file(Path::new(path))?;
        imported += 1;
    }

    notify_system();
    Ok(ActionResult {
        message: format!("已导入 {imported} 个字体文件，现在可以继续应用喜欢的风格了。"),
        backup_path: None,
        active_preset_id: None,
        active_render_style_id: None,
        export_path: None,
    })
}

pub fn restore_windows_default(presets: &[PresetDefinition]) -> Result<ActionResult> {
    ensure_admin()?;
    let backup_path = restore_windows_default_internal(presets)?;
    Ok(ActionResult {
        message: "已经恢复到 Windows 原生设定。".to_string(),
        backup_path: Some(backup_path.to_string_lossy().to_string()),
        active_preset_id: None,
        active_render_style_id: Some(RenderStyleId::Balanced),
        export_path: None,
    })
}

pub fn run_recovery_action(action: &str, presets: &[PresetDefinition]) -> Result<ActionResult> {
    ensure_admin()?;

    let result = match action {
        "rollbackLast" => rollback_last_change(presets)?,
        "restoreFontMappings" => restore_component("FontSubstitutes.reg", "已恢复字体映射。")?,
        "restoreFontLink" => restore_component("FontLink.reg", "已恢复 FontLink 回退链。")?,
        "restoreRendering" => restore_component("Avalon.Graphics.reg", "已恢复渲染参数。")?,
        "restoreWindowsDefault" => restore_windows_default(presets)?,
        "repairSystemFonts" => {
            launch_system_font_repair()?;
            ActionResult {
                message: "系统修复工具已经打开，接下来会自动运行 DISM 和 SFC。".to_string(),
                backup_path: None,
                active_preset_id: None,
                active_render_style_id: None,
                export_path: None,
            }
        }
        "refreshExplorer" => {
            restart_explorer()?;
            ActionResult {
                message: "资源管理器已经刷新。".to_string(),
                backup_path: None,
                active_preset_id: None,
                active_render_style_id: None,
                export_path: None,
            }
        }
        other => bail!("不认识这个恢复动作：{other}"),
    };

    Ok(result)
}

pub fn export_current_scheme(
    preset_id: &str,
    render_style_id: RenderStyleId,
    presets: &[PresetDefinition],
) -> Result<ActionResult> {
    let preset = presets
        .iter()
        .find(|preset| preset.id == preset_id)
        .copied()
        .ok_or_else(|| anyhow!("没找到当前方案。"))?;

    let export_dir = data_local_dir()
        .unwrap_or_else(env::temp_dir)
        .join("WindowsFontTuner")
        .join("Exports");
    fs::create_dir_all(&export_dir).context("创建导出目录失败。")?;

    let file_name = format!("{}-{}.wftpreset.json", preset.id, render_style_id.as_str());
    let export_path = export_dir.join(file_name);

    let payload = serde_json::json!({
        "id": preset.id,
        "name": preset.font_family,
        "cnFont": preset.font_family,
        "enFont": if preset.id == "inter-harmonyos" { Some("Inter") } else { None::<&str> },
        "renderStyle": render_style_id.as_str(),
        "screenProfile": sniff_display_profile().matrix_profile,
        "riskLevel": preset.risk_level,
        "tags": [],
        "fontLinkFallbacks": preset.fallback_families,
        "createdAt": now_label(),
        "version": "3.0.0"
    });

    fs::write(&export_path, to_string_pretty(&payload)?).with_context(|| {
        format!("导出方案失败：{}", export_path.display())
    })?;

    Ok(ActionResult {
        message: "当前方案已经导出，可以拿去分享或备份。".to_string(),
        backup_path: None,
        active_preset_id: Some(preset.id.to_string()),
        active_render_style_id: Some(render_style_id),
        export_path: Some(export_path.to_string_lossy().to_string()),
    })
}

pub fn import_shared_scheme(path: &str) -> Result<ImportedPresetPayload> {
    let content = fs::read_to_string(path).with_context(|| format!("读取方案文件失败：{path}"))?;
    let imported: ImportedPresetFile = from_str(&content).context("方案文件格式不正确。")?;
    let render_style_id = imported
        .render_style
        .as_deref()
        .and_then(RenderStyleId::from_str)
        .unwrap_or(RenderStyleId::Balanced);

    let mut warnings = Vec::new();
    if !font_family_exists(&imported.cn_font) {
        warnings.push("这套方案引用的中文字体当前机器上还没有准备好。".to_string());
    }
    if let Some(font) = imported.en_font.as_deref() {
        if !font_family_exists(font) {
            warnings.push("英文字体当前不存在，应用后会退回系统默认。".to_string());
        }
    }
    if imported
        .font_link_fallbacks
        .as_ref()
        .map(|items| !items.iter().any(|item| item.contains("Segoe UI Emoji")))
        .unwrap_or(true)
    {
        warnings.push("这份方案没有显式写 Emoji 回退链，应用时会自动补上。".to_string());
    }

    Ok(ImportedPresetPayload {
        name: imported.name,
        cn_font: imported.cn_font,
        en_font: imported.en_font,
        render_style_id,
        risk_level: imported.risk_level.unwrap_or(RiskLevel::Medium),
        tags: imported.tags.unwrap_or_default(),
        warnings,
    })
}

pub fn launch_system_font_repair() -> Result<()> {
    let cmd = system32_path("cmd.exe");
    let args = "/k title Windows 系统字体修复 && echo 正在运行 DISM 和 SFC，这个过程可能需要几分钟到十几分钟... && echo. && DISM.exe /Online /Cleanup-Image /RestoreHealth && echo. && echo DISM 完成，继续执行 sfc /scannow... && sfc /scannow";

    if is_admin() {
        Command::new(cmd)
            .args(["/k", "title Windows 系统字体修复 && echo 正在运行 DISM 和 SFC，这个过程可能需要几分钟到十几分钟... && echo. && DISM.exe /Online /Cleanup-Image /RestoreHealth && echo. && echo DISM 完成，继续执行 sfc /scannow... && sfc /scannow"])
            .spawn()
            .context("没能启动系统修复工具。")?;
        return Ok(());
    }

    shell_execute_runas(&cmd, args)
}

fn restore_windows_default_internal(_presets: &[PresetDefinition]) -> Result<PathBuf> {
    let backup_path = create_backup()?;
    clear_managed_font_substitutes()?;
    clear_managed_font_links()?;
    reset_desktop_settings()?;
    reset_rendering_settings()?;
    clear_app_state()?;
    notify_system();
    Ok(backup_path)
}

fn rollback_last_change(presets: &[PresetDefinition]) -> Result<ActionResult> {
    let app_state = read_app_state()?;
    let backup_path = app_state
        .last_backup_path
        .clone()
        .ok_or_else(|| anyhow!("还没有可回滚的修改记录。"))?;
    let backup_dir = PathBuf::from(&backup_path);
    restore_backup_directory(&backup_dir)?;
    let refreshed = load_bootstrap(presets)?;

    Ok(ActionResult {
        message: "已经回滚到上一次修改前的状态。".to_string(),
        backup_path: Some(backup_path),
        active_preset_id: refreshed.active_preset_id,
        active_render_style_id: Some(refreshed.active_render_style_id),
        export_path: None,
    })
}

fn restore_component(file_name: &str, message: &str) -> Result<ActionResult> {
    let backup_path = read_app_state()?
        .last_backup_path
        .ok_or_else(|| anyhow!("还没有备份记录可供恢复。"))?;
    let target = PathBuf::from(&backup_path).join(file_name);
    import_registry(&target)?;
    notify_system();
    let _ = restart_explorer();

    Ok(ActionResult {
        message: message.to_string(),
        backup_path: Some(backup_path),
        active_preset_id: None,
        active_render_style_id: None,
        export_path: None,
    })
}

fn restore_backup_directory(backup_dir: &Path) -> Result<()> {
    for file_name in [
        "FontSubstitutes.reg",
        "FontLink.reg",
        "CurrentUserFonts.reg",
        "Desktop.reg",
        "Avalon.Graphics.reg",
    ] {
        let path = backup_dir.join(file_name);
        if path.exists() {
            import_registry(&path)?;
        }
    }

    notify_system();
    restart_explorer()?;
    Ok(())
}

fn import_registry(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("备份文件不存在：{}", path.display());
    }

    Command::new(system32_path("reg.exe"))
        .args(["import", path.to_string_lossy().as_ref()])
        .status()
        .with_context(|| format!("导入注册表失败：{}", path.display()))?;
    Ok(())
}

fn ensure_preset_assets(preset: &PresetDefinition) -> Result<()> {
    if preset.required_fonts.iter().all(|family| font_family_exists(family)) {
        return Ok(());
    }

    if preset.install_mode == InstallMode::ManualImport {
        bail!("这套风格需要你先导入字体文件。");
    }

    download_and_install_required_fonts(preset)
}

fn download_and_install_required_fonts(preset: &PresetDefinition) -> Result<()> {
    if preset.downloads.is_empty() {
        bail!("这套风格目前没有可用的自动下载源。");
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .context("初始化下载器失败。")?;

    let cache_root = download_root().join(preset.id);
    fs::create_dir_all(&cache_root).context("创建字体下载目录失败。")?;

    for asset in preset.downloads {
        let destination = cache_root.join(asset.file_name);
        if !destination.exists() {
            download_to_file(&client, asset, &destination)?;
        }

        install_font_file(&destination)?;
    }

    Ok(())
}

fn download_to_file(client: &Client, asset: &FontDownload, destination: &Path) -> Result<()> {
    let response = client
        .get(asset.url)
        .send()
        .and_then(|resp| resp.error_for_status())
        .with_context(|| format!("下载字体失败：{}", asset.file_name))?;

    let bytes = response.bytes().context("读取字体文件响应失败。")?;
    let mut file = fs::File::create(destination)
        .with_context(|| format!("写入字体缓存失败：{}", destination.display()))?;
    file.write_all(&bytes)
        .with_context(|| format!("保存字体缓存失败：{}", destination.display()))?;
    Ok(())
}

fn install_font_file(source: &Path) -> Result<()> {
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_default();

    if !matches!(extension.as_str(), "ttf" | "otf" | "ttc" | "otc") {
        bail!("只支持导入 ttf / otf / ttc / otc 字体文件。");
    }

    let install_dir = user_font_store();
    fs::create_dir_all(&install_dir).context("创建用户字体目录失败。")?;

    let file_name = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("字体文件名无效。"))?;

    let destination = install_dir.join(file_name);
    fs::copy(source, &destination)
        .with_context(|| format!("复制字体文件失败：{}", source.display()))?;

    let records = inspect_font_records(&destination)?;
    register_user_fonts(&records)?;

    let destination_wide = wide(destination.as_os_str());
    let added = unsafe {
        AddFontResourceExW(
            PCWSTR(destination_wide.as_ptr()),
            FONT_RESOURCE_CHARACTERISTICS(0),
            None,
        )
    };
    if added == 0 {
        // Copy + registry is already enough for next login; keep going.
    }

    Ok(())
}

fn inspect_font_records(path: &Path) -> Result<Vec<FontInstallRecord>> {
    let bytes = fs::read(path).with_context(|| format!("读取字体文件失败：{}", path.display()))?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_else(|| "ttf".to_string());
    let registry_kind = if matches!(extension.as_str(), "otf" | "otc") {
        "OpenType"
    } else {
        "TrueType"
    };

    let family = read_name(&bytes, name_id::TYPOGRAPHIC_FAMILY)
        .or_else(|| read_name(&bytes, name_id::FULL_NAME))
        .or_else(|| read_name(&bytes, name_id::FAMILY))
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("Imported Font")
                .to_string()
        });
    let subfamily = read_name(&bytes, name_id::TYPOGRAPHIC_SUBFAMILY)
        .or_else(|| read_name(&bytes, name_id::SUBFAMILY))
        .unwrap_or_default();

    let display_name = if subfamily.is_empty()
        || matches!(
            subfamily.to_ascii_lowercase().as_str(),
            "regular" | "roman" | "book" | "normal"
        )
    {
        family
    } else {
        format!("{family} {subfamily}")
    };

    Ok(vec![FontInstallRecord {
        registry_name: format!("{display_name} ({registry_kind})"),
        file_name: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string(),
    }])
}

fn read_name(bytes: &[u8], name_id: u16) -> Option<String> {
    let face = Face::parse(bytes, 0).ok()?;
    face.names()
        .into_iter()
        .find(|name| name.name_id == name_id)
        .and_then(|name| name.to_string())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn register_user_fonts(records: &[FontInstallRecord]) -> Result<()> {
    if records.is_empty() {
        return Ok(());
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .create_subkey(FONTS_PATH)
        .context("写入当前用户字体注册表失败。")?
        .0;

    for record in records {
        key.set_value(&record.registry_name, &record.file_name)
            .with_context(|| format!("注册字体失败：{}", record.registry_name))?;
    }

    Ok(())
}

fn sniff_display_profile() -> DisplayProfile {
    let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    let monitor_count = unsafe { GetSystemMetrics(SM_CMONITORS) };
    let dpi = unsafe { GetDpiForSystem() };
    let scale_percent = ((dpi as f32 / 96.0) * 100.0).round() as u32;

    let resolution_label = if width >= 3800 {
        format!("4K · {}%", scale_percent)
    } else if width >= 2500 {
        format!("2K · {}%", scale_percent)
    } else {
        format!("1080p · {}%", scale_percent)
    };

    let matrix_profile = if width >= 3800 || scale_percent >= 175 {
        "4K 轻锐矩阵"
    } else if width >= 2500 || scale_percent >= 125 {
        "2K 均衡矩阵"
    } else {
        "1080p 清晰矩阵"
    }
    .to_string();

    let ppi_label = if width >= 3800 || scale_percent >= 175 {
        "高 PPI"
    } else if width >= 2500 || scale_percent >= 125 {
        "中高 PPI"
    } else {
        "标准 PPI"
    }
    .to_string();

    DisplayProfile {
        width,
        height,
        scale_percent,
        resolution_label,
        matrix_profile,
        ppi_label,
        multi_monitor: monitor_count > 1,
    }
}

fn detect_active_preset_id(presets: &[PresetDefinition]) -> Result<Option<String>> {
    let key = open_hklm_read(FONT_SUBSTITUTES_PATH)?;

    for preset in presets.iter().filter(|preset| preset.status == PresetStatus::Ready) {
        let all_match = preset.font_substitutes.iter().all(|(name, value)| {
            key.get_value::<String, _>(*name)
                .map(|current| current.eq_ignore_ascii_case(value))
                .unwrap_or(false)
        });

        if all_match {
            return Ok(Some(preset.id.to_string()));
        }
    }

    Ok(None)
}

fn resolve_active_font_label(
    presets: &[PresetDefinition],
    active_preset_id: Option<&str>,
) -> Result<String> {
    if let Some(active_id) = active_preset_id {
        if let Some(preset) = presets.iter().find(|preset| preset.id == active_id) {
            return Ok(preset.font_family.to_string());
        }
    }

    let key = open_hklm_read(FONT_SUBSTITUTES_PATH)?;
    if let Ok(value) = key.get_value::<String, _>("Segoe UI") {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }

    Ok("Windows 默认".to_string())
}

fn write_font_substitutes(preset: &PresetDefinition) -> Result<()> {
    let key = open_hklm_write(FONT_SUBSTITUTES_PATH)?;
    for name in MANAGED_FONT_SUBSTITUTE_NAMES {
        let _ = key.delete_value(name);
    }

    for (name, value) in preset.font_substitutes {
        key.set_value(*name, &value.to_string())
            .with_context(|| format!("写入字体映射失败：{name}"))?;
    }

    Ok(())
}

fn clear_managed_font_substitutes() -> Result<()> {
    let key = open_hklm_write(FONT_SUBSTITUTES_PATH)?;
    for name in MANAGED_FONT_SUBSTITUTE_NAMES {
        let _ = key.delete_value(name);
    }
    Ok(())
}

fn write_font_link_fallbacks(preset: &PresetDefinition) -> Result<()> {
    let key = open_hklm_write(FONT_LINK_PATH)?;
    let entries = resolve_font_link_entries(preset.fallback_families)?;

    if entries.is_empty() {
        return Ok(());
    }

    for base in MANAGED_FONT_SUBSTITUTE_NAMES {
        key.set_value(base, &entries)
            .with_context(|| format!("写入 FontLink 回退链失败：{base}"))?;
    }

    Ok(())
}

fn clear_managed_font_links() -> Result<()> {
    let key = open_hklm_write(FONT_LINK_PATH)?;
    for name in MANAGED_FONT_SUBSTITUTE_NAMES {
        let _ = key.delete_value(name);
    }
    Ok(())
}

fn resolve_font_link_entries(families: &[&str]) -> Result<Vec<String>> {
    let mut entries = Vec::new();
    let mut seen = BTreeSet::new();

    for family in families {
        for value in find_font_link_entries(family)? {
            let dedupe = value.to_ascii_lowercase();
            if seen.insert(dedupe) {
                entries.push(value);
            }
        }
    }

    Ok(entries)
}

fn find_font_link_entries(family: &str) -> Result<Vec<String>> {
    let mut entries = Vec::new();
    entries.extend(collect_font_link_entries(open_hklm_read(FONTS_PATH).ok(), family));
    entries.extend(collect_font_link_entries(open_hkcu_read(FONTS_PATH).ok(), family));
    Ok(entries)
}

fn collect_font_link_entries(key: Option<RegKey>, family: &str) -> Vec<String> {
    let Some(key) = key else {
        return Vec::new();
    };

    key.enum_values()
        .flatten()
        .filter_map(|(name, raw_value)| {
            let lower_name = name.to_ascii_lowercase();
            if !lower_name.contains(&family.to_ascii_lowercase()) {
                return None;
            }

            let file_name = decode_reg_value_string(&raw_value.bytes)?;
            let display_name = name
                .split(" (")
                .next()
                .unwrap_or(&name)
                .trim()
                .to_string();
            Some(format!("{file_name},{display_name}"))
        })
        .collect()
}

fn read_app_state() -> Result<AppStateSnapshot> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = match hkcu.open_subkey_with_flags(APP_STATE_PATH, KEY_READ) {
        Ok(key) => key,
        Err(_) => return Ok(AppStateSnapshot::default()),
    };

    let active_preset_id = key.get_value::<String, _>("ActivePresetId").ok();
    let active_render_style_id = key
        .get_value::<String, _>("ActiveRenderStyleId")
        .ok()
        .and_then(|value| RenderStyleId::from_str(&value));
    let last_backup_path = key.get_value::<String, _>("LastBackupPath").ok();
    let last_applied_at = key.get_value::<String, _>("LastAppliedAt").ok();

    Ok(AppStateSnapshot {
        active_preset_id,
        active_render_style_id,
        last_backup_path,
        last_applied_at,
    })
}

fn write_app_state(snapshot: &AppStateSnapshot) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .create_subkey(APP_STATE_PATH)
        .context("写入应用状态失败。")?
        .0;

    if let Some(value) = snapshot.active_preset_id.as_deref() {
        key.set_value("ActivePresetId", &value)?;
    } else {
        let _ = key.delete_value("ActivePresetId");
    }

    if let Some(value) = snapshot.active_render_style_id {
        key.set_value("ActiveRenderStyleId", &value.as_str())?;
    } else {
        let _ = key.delete_value("ActiveRenderStyleId");
    }

    if let Some(value) = snapshot.last_backup_path.as_deref() {
        key.set_value("LastBackupPath", &value)?;
    } else {
        let _ = key.delete_value("LastBackupPath");
    }

    if let Some(value) = snapshot.last_applied_at.as_deref() {
        key.set_value("LastAppliedAt", &value)?;
    } else {
        let _ = key.delete_value("LastAppliedAt");
    }

    Ok(())
}

fn clear_app_state() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if hkcu.open_subkey_with_flags(APP_STATE_PATH, KEY_WRITE).is_ok() {
        let _ = hkcu.delete_subkey_all(APP_STATE_PATH);
    }
    Ok(())
}

fn detect_active_render_style(app_state_style: Option<RenderStyleId>) -> Result<RenderStyleId> {
    if let Some(style) = app_state_style {
        return Ok(style);
    }

    let desktop = current_desktop_settings()?;
    let rendering = current_rendering_settings()?;

    if rendering.clear_type_level >= 100 && rendering.text_contrast_level >= 6 {
        if desktop.font_smoothing_gamma >= 1940 {
            return Ok(RenderStyleId::Code);
        }
        return Ok(RenderStyleId::Clear);
    }

    if rendering.clear_type_level == 0 && rendering.text_contrast_level <= 3 {
        return Ok(RenderStyleId::Reading);
    }

    if rendering.clear_type_level == 0 && rendering.gamma_level <= 1760 {
        return Ok(RenderStyleId::Rounded);
    }

    if rendering.clear_type_level == 0 && rendering.text_contrast_level == 4 {
        return Ok(RenderStyleId::Soft);
    }

    Ok(RenderStyleId::Balanced)
}

fn current_desktop_settings() -> Result<DesktopTextSettings> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .create_subkey(DESKTOP_PATH)
        .context("读取桌面字体设置失败。")?
        .0;

    Ok(DesktopTextSettings {
        font_smoothing: Box::leak(
            key.get_value::<String, _>("FontSmoothing")
                .unwrap_or_else(|_| "2".to_string())
                .into_boxed_str(),
        ),
        font_smoothing_type: key.get_value("FontSmoothingType").unwrap_or(2_u32),
        font_smoothing_gamma: key.get_value("FontSmoothingGamma").unwrap_or(1900_u32),
        font_smoothing_orientation: key
            .get_value("FontSmoothingOrientation")
            .unwrap_or(1_u32),
    })
}

fn current_rendering_settings() -> Result<RenderingSettings> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let root = match hkcu.open_subkey_with_flags(AVALON_GRAPHICS_PATH, KEY_READ) {
        Ok(root) => root,
        Err(_) => {
            return Ok(RenderingSettings {
                pixel_structure: 1,
                gamma_level: 1900,
                clear_type_level: 100,
                text_contrast_level: 1,
            })
        }
    };

    for display_key_name in root.enum_keys().flatten() {
        if let Ok(display_key) = root.open_subkey_with_flags(&display_key_name, KEY_READ) {
            return Ok(RenderingSettings {
                pixel_structure: display_key.get_value("PixelStructure").unwrap_or(1_u32),
                gamma_level: display_key.get_value("GammaLevel").unwrap_or(1900_u32),
                clear_type_level: display_key.get_value("ClearTypeLevel").unwrap_or(100_u32),
                text_contrast_level: display_key.get_value("TextContrastLevel").unwrap_or(1_u32),
            });
        }
    }

    Ok(RenderingSettings {
        pixel_structure: 1,
        gamma_level: 1900,
        clear_type_level: 100,
        text_contrast_level: 1,
    })
}

fn describe_current_state(app_state: &AppStateSnapshot) -> String {
    if app_state.active_preset_id.is_some() {
        "已优化".to_string()
    } else {
        "系统默认".to_string()
    }
}

fn describe_last_modified(app_state: &AppStateSnapshot) -> String {
    app_state
        .last_applied_at
        .clone()
        .unwrap_or_else(|| "这台机器还没有写入过新的字体风格。".to_string())
}

fn describe_last_backup_label(app_state: &AppStateSnapshot) -> String {
    app_state
        .last_backup_path
        .as_deref()
        .and_then(|path| Path::new(path).file_name())
        .and_then(|value| value.to_str())
        .unwrap_or("还没有可恢复的本地快照")
        .to_string()
}

fn evaluate_font_health(preset: &PresetDefinition, render_style_id: RenderStyleId) -> FontHealthReport {
    let available = preset.required_fonts.iter().all(|family| font_family_exists(family));
    let font_status = if available {
        CheckStatus::Pass
    } else if preset.install_mode == InstallMode::AutoDownload {
        CheckStatus::Warn
    } else {
        CheckStatus::Risk
    };

    let emoji_ok = preset
        .fallback_families
        .iter()
        .any(|family| family.contains("Segoe UI Emoji"));
    let emoji_status = if emoji_ok {
        CheckStatus::Pass
    } else {
        CheckStatus::Risk
    };

    let compatibility_status = match preset.risk_level {
        RiskLevel::Low => CheckStatus::Pass,
        RiskLevel::Medium => CheckStatus::Warn,
        RiskLevel::High => CheckStatus::Risk,
    };

    let render_status = match render_style_id {
        RenderStyleId::Balanced | RenderStyleId::Clear | RenderStyleId::Rounded => CheckStatus::Pass,
        RenderStyleId::Soft | RenderStyleId::Reading | RenderStyleId::Code => CheckStatus::Warn,
    };

    let items = vec![
        HealthItem {
            label: "字体文件".to_string(),
            status: font_status,
            detail: if available {
                "主字体已经就绪，应用时不会临时掉回系统默认。".to_string()
            } else if preset.install_mode == InstallMode::AutoDownload {
                "缺少的字体会在应用时自动下载并安装。".to_string()
            } else {
                "这套方案需要先导入本地字体文件再应用。".to_string()
            },
        },
        HealthItem {
            label: "Emoji 与中文兜底".to_string(),
            status: emoji_status,
            detail: if emoji_ok {
                "应用时会自动补 Segoe UI Emoji 和微软雅黑回退链。".to_string()
            } else {
                "当前方案缺少 Emoji 回退链，建议谨慎使用。".to_string()
            },
        },
        HealthItem {
            label: "兼容性".to_string(),
            status: compatibility_status,
            detail: match preset.risk_level {
                RiskLevel::Low => "适合长期系统默认，风险较低。".to_string(),
                RiskLevel::Medium => "部分软件可能会有观感差异，但整体仍可控。".to_string(),
                RiskLevel::High => "风格实验性较强，更适合体验而不是长期默认。".to_string(),
            },
        },
        HealthItem {
            label: "渲染风格".to_string(),
            status: render_status,
            detail: format!("当前准备叠加“{}”这档渲染策略。", render_style_id.label()),
        },
    ];

    let overall_status = items
        .iter()
        .map(|item| item.status)
        .max_by_key(|status| match status {
            CheckStatus::Pass => 0,
            CheckStatus::Warn => 1,
            CheckStatus::Risk => 2,
        })
        .unwrap_or(CheckStatus::Pass);

    let summary = match overall_status {
        CheckStatus::Pass => "这套方案可以放心试，整体风险比较低。".to_string(),
        CheckStatus::Warn => "这套方案能用，但有几处地方值得你先看一眼。".to_string(),
        CheckStatus::Risk => "建议先补齐字体或检查回退链，再决定是否应用。".to_string(),
    };

    FontHealthReport {
        overall_status,
        summary,
        items,
    }
}

fn decode_reg_value_string(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }

    if bytes.len() >= 2 {
        let mut utf16 = Vec::with_capacity(bytes.len() / 2);
        for chunk in bytes.chunks(2) {
            if chunk.len() == 2 {
                utf16.push(u16::from_le_bytes([chunk[0], chunk[1]]));
            }
        }
        let text = String::from_utf16_lossy(&utf16)
            .trim_matches(char::from(0))
            .trim()
            .to_string();
        if !text.is_empty() {
            return Some(text);
        }
    }

    let text = String::from_utf8_lossy(bytes)
        .trim_matches(char::from(0))
        .trim()
        .to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn resolve_desktop_settings(
    base: DesktopTextSettings,
    render_style_id: RenderStyleId,
    display: &DisplayProfile,
) -> DesktopTextSettings {
    let template = render_style_template(render_style_id);
    let mut settings = template.desktop;

    settings.font_smoothing_orientation = base.font_smoothing_orientation;

    if display.width >= 3800 || display.scale_percent >= 175 {
        settings.font_smoothing_gamma = settings.font_smoothing_gamma.saturating_sub(40);
    } else if display.width < 2500 && display.scale_percent <= 100 {
        settings.font_smoothing_type = 2;
        settings.font_smoothing_gamma = settings.font_smoothing_gamma.saturating_add(40);
    }

    settings
}

fn resolve_rendering_settings(
    base: RenderingSettings,
    render_style_id: RenderStyleId,
    display: &DisplayProfile,
) -> RenderingSettings {
    let template = render_style_template(render_style_id);
    let mut settings = template.rendering;

    if display.width >= 3800 || display.scale_percent >= 175 {
        settings.text_contrast_level = settings.text_contrast_level.saturating_sub(1).max(1);
        settings.gamma_level = settings.gamma_level.saturating_sub(50);
    } else if display.width < 2500 && display.scale_percent <= 100 {
        settings.pixel_structure = 1;
        settings.text_contrast_level = settings.text_contrast_level.max(base.text_contrast_level);
        settings.gamma_level = settings.gamma_level.max(1950);
    }

    settings
}

fn write_desktop_settings(settings: DesktopTextSettings) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .create_subkey(DESKTOP_PATH)
        .context("没能打开桌面字体设置。")?
        .0;

    key.set_value("FontSmoothing", &settings.font_smoothing)
        .context("写入 FontSmoothing 失败。")?;
    key.set_value("FontSmoothingType", &settings.font_smoothing_type)
        .context("写入 FontSmoothingType 失败。")?;
    key.set_value("FontSmoothingGamma", &settings.font_smoothing_gamma)
        .context("写入 FontSmoothingGamma 失败。")?;
    key.set_value(
        "FontSmoothingOrientation",
        &settings.font_smoothing_orientation,
    )
    .context("写入 FontSmoothingOrientation 失败。")?;

    Ok(())
}

fn reset_desktop_settings() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .create_subkey(DESKTOP_PATH)
        .context("没能打开桌面字体设置。")?
        .0;

    key.set_value("FontSmoothing", &"2")
        .context("重置 FontSmoothing 失败。")?;
    key.set_value("FontSmoothingType", &2_u32)
        .context("重置 FontSmoothingType 失败。")?;
    key.set_value("FontSmoothingGamma", &1900_u32)
        .context("重置 FontSmoothingGamma 失败。")?;
    key.set_value("FontSmoothingOrientation", &1_u32)
        .context("重置 FontSmoothingOrientation 失败。")?;

    Ok(())
}

fn write_rendering_settings(settings: RenderingSettings) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let root = match hkcu.open_subkey_with_flags(AVALON_GRAPHICS_PATH, KEY_READ | KEY_WRITE) {
        Ok(root) => root,
        Err(_) => return Ok(()),
    };

    for display_key_name in root.enum_keys().flatten() {
        if let Ok(display_key) = root.open_subkey_with_flags(&display_key_name, KEY_READ | KEY_WRITE)
        {
            let _ = display_key.set_value("PixelStructure", &settings.pixel_structure);
            let _ = display_key.set_value("GammaLevel", &settings.gamma_level);
            let _ = display_key.set_value("ClearTypeLevel", &settings.clear_type_level);
            let _ = display_key.set_value("TextContrastLevel", &settings.text_contrast_level);
        }
    }

    Ok(())
}

fn reset_rendering_settings() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let root = match hkcu.open_subkey_with_flags(AVALON_GRAPHICS_PATH, KEY_READ | KEY_WRITE) {
        Ok(root) => root,
        Err(_) => return Ok(()),
    };

    for display_key_name in root.enum_keys().flatten() {
        if let Ok(display_key) = root.open_subkey_with_flags(&display_key_name, KEY_READ | KEY_WRITE)
        {
            let _ = display_key.set_value("PixelStructure", &1_u32);
            let _ = display_key.set_value("GammaLevel", &1900_u32);
            let _ = display_key.set_value("ClearTypeLevel", &100_u32);
            let _ = display_key.set_value("TextContrastLevel", &1_u32);
        }
    }

    Ok(())
}

fn notify_system() {
    unsafe {
        let _ = SendMessageTimeoutW(
            HWND(HWND_BROADCAST.0),
            WM_FONTCHANGE,
            WPARAM(0),
            LPARAM(0),
            SMTO_ABORTIFHUNG,
            1000,
            None,
        );
        let _ = SendMessageTimeoutW(
            HWND(HWND_BROADCAST.0),
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(0),
            SMTO_ABORTIFHUNG,
            1000,
            None,
        );
    }
}

fn font_family_exists(family_name: &str) -> bool {
    let family_name_lower = family_name.to_ascii_lowercase();
    font_registry_matches(open_hklm_read(FONTS_PATH).ok(), &family_name_lower)
        || font_registry_matches(open_hkcu_read(FONTS_PATH).ok(), &family_name_lower)
}

fn font_registry_matches(key: Option<RegKey>, family_name_lower: &str) -> bool {
    let Some(key) = key else {
        return false;
    };

    key.enum_values().flatten().any(|(name, value)| {
        let value_text = decode_reg_value_string(&value.bytes).unwrap_or_default();
        name.to_ascii_lowercase().contains(family_name_lower)
            || value_text.to_ascii_lowercase().contains(family_name_lower)
    })
}

fn backup_root() -> PathBuf {
    data_local_dir()
        .unwrap_or_else(env::temp_dir)
        .join("WindowsFontTuner")
        .join("Backups")
}

fn download_root() -> PathBuf {
    data_local_dir()
        .unwrap_or_else(env::temp_dir)
        .join("WindowsFontTuner")
        .join("Downloads")
}

fn user_font_store() -> PathBuf {
    data_local_dir()
        .unwrap_or_else(env::temp_dir)
        .join("Microsoft")
        .join("Windows")
        .join("Fonts")
}

fn list_backup_directories() -> Vec<PathBuf> {
    let root = backup_root();
    let mut entries = fs::read_dir(root)
        .map(|iter| {
            iter.flatten()
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    entries.sort();
    entries.reverse();
    entries
}

fn create_backup() -> Result<PathBuf> {
    let root = backup_root();
    fs::create_dir_all(&root).context("没能创建备份目录。")?;

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let target = root.join(format!("backup-{stamp}"));
    fs::create_dir_all(&target).context("没能创建当前备份目录。")?;

    export_registry(
        r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\FontSubstitutes",
        target.join("FontSubstitutes.reg"),
    )?;
    export_registry(
        r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\FontLink\SystemLink",
        target.join("FontLink.reg"),
    )?;
    export_registry(
        r"HKCU\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts",
        target.join("CurrentUserFonts.reg"),
    )?;
    export_registry(r"HKCU\Control Panel\Desktop", target.join("Desktop.reg"))?;
    export_registry(
        r"HKCU\Software\Microsoft\Avalon.Graphics",
        target.join("Avalon.Graphics.reg"),
    )?;

    Ok(target)
}

fn export_registry(registry_path: &str, destination: PathBuf) -> Result<()> {
    Command::new(system32_path("reg.exe"))
        .args([
            "export",
            registry_path,
            destination.to_string_lossy().as_ref(),
            "/y",
        ])
        .status()
        .with_context(|| format!("导出注册表失败：{registry_path}"))?;
    Ok(())
}

fn now_label() -> String {
    Local::now().format("%Y-%m-%d %H:%M").to_string()
}

fn restart_explorer() -> Result<()> {
    Command::new(system32_path("taskkill.exe"))
        .args(["/f", "/im", "explorer.exe"])
        .status()
        .context("结束 Explorer 失败。")?;
    Command::new(system32_path("cmd.exe"))
        .args(["/c", "start", "", "explorer.exe"])
        .spawn()
        .context("重启 Explorer 失败。")?;
    Ok(())
}

fn open_hklm_read(path: &str) -> Result<RegKey> {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(path, KEY_READ | KEY_WOW64_64KEY)
        .with_context(|| format!("没能读取注册表：{path}"))
}

fn open_hkcu_read(path: &str) -> Result<RegKey> {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(path, KEY_READ)
        .with_context(|| format!("没能读取当前用户注册表：{path}"))
}

fn open_hklm_write(path: &str) -> Result<RegKey> {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .create_subkey_with_flags(path, KEY_WRITE | KEY_WOW64_64KEY)
        .map(|(key, _)| key)
        .with_context(|| format!("没能写入注册表：{path}"))
}

fn system32_path(file_name: &str) -> PathBuf {
    PathBuf::from(env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string()))
        .join("System32")
        .join(file_name)
}

fn ensure_admin() -> Result<()> {
    if is_admin() {
        Ok(())
    } else {
        bail!("请用管理员权限启动 2.0，再来改系统字体。");
    }
}

fn is_admin() -> bool {
    unsafe { IsUserAnAdmin().as_bool() }
}

fn is_shift_pressed() -> bool {
    unsafe { GetAsyncKeyState(VK_SHIFT.0.into()) < 0 }
}

fn run_self_elevated(arg: &str) -> Result<()> {
    let current_exe = env::current_exe().context("没能定位当前程序。")?;
    shell_execute_runas(&current_exe, arg)
}

fn shell_execute_runas(executable: &Path, arguments: &str) -> Result<()> {
    let verb = wide("runas");
    let file = wide(executable.as_os_str());
    let args = wide(arguments);

    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(verb.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR(args.as_ptr()),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };

    if (result.0 as isize) <= 32 {
        bail!("系统拒绝了管理员提权。");
    }

    Ok(())
}

fn wide(value: impl AsRef<OsStr>) -> Vec<u16> {
    value
        .as_ref()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
