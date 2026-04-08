use crate::models::{
    ActionResult, BootstrapPayload, DesktopTextSettings, DisplayProfile, FontDownload,
    InstallMode, PresetDefinition, PresetStatus, RenderingSettings, RuntimePresetState,
};
use anyhow::{anyhow, bail, Context, Result};
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
use ttf_parser::{name_id, Face};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{AddFontResourceExW, FONT_RESOURCE_CHARACTERISTICS};
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_SHIFT};
use windows::Win32::UI::Shell::{IsUserAnAdmin, ShellExecuteW};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, SM_CXSCREEN,
    SM_CYSCREEN, SW_SHOWNORMAL, WM_FONTCHANGE, WM_SETTINGCHANGE,
};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY, KEY_WRITE};
use winreg::RegKey;

const FONT_SUBSTITUTES_PATH: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\FontSubstitutes";
const FONTS_PATH: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts";
const FONT_LINK_PATH: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\FontLink\SystemLink";
const DESKTOP_PATH: &str = r"Control Panel\Desktop";
const AVALON_GRAPHICS_PATH: &str = r"Software\Microsoft\Avalon.Graphics";
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

pub fn load_bootstrap(presets: &[PresetDefinition]) -> Result<BootstrapPayload> {
    let display = sniff_display_profile();
    let active_preset_id = detect_active_preset_id(presets)?;
    let active_font_label = resolve_active_font_label(presets, active_preset_id.as_deref())?;
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
        backup_count: list_backup_directories().len(),
        backup_dir: backup_dir.to_string_lossy().to_string(),
        display,
        presets,
    })
}

pub fn apply_preset(preset_id: &str, presets: &[PresetDefinition]) -> Result<ActionResult> {
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
    write_desktop_settings(resolve_desktop_settings(preset.desktop, &display))?;
    write_rendering_settings(resolve_rendering_settings(preset.rendering, &display))?;
    notify_system();

    Ok(ActionResult {
        message: "✨ 风格已应用，注销电脑后即可感受。".to_string(),
        backup_path: Some(backup_path.to_string_lossy().to_string()),
        active_preset_id: Some(preset.id.to_string()),
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
    })
}

pub fn restore_windows_default(presets: &[PresetDefinition]) -> Result<ActionResult> {
    ensure_admin()?;
    let backup_path = restore_windows_default_internal(presets)?;
    Ok(ActionResult {
        message: "已经恢复到 Windows 原生设定。".to_string(),
        backup_path: Some(backup_path.to_string_lossy().to_string()),
        active_preset_id: None,
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
    notify_system();
    Ok(backup_path)
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

    DisplayProfile {
        width,
        height,
        scale_percent,
        resolution_label,
        matrix_profile,
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
    display: &DisplayProfile,
) -> DesktopTextSettings {
    if display.width >= 3800 || display.scale_percent >= 175 {
        DesktopTextSettings {
            font_smoothing_gamma: 1800,
            ..base
        }
    } else if display.width >= 2500 || display.scale_percent >= 125 {
        base
    } else {
        DesktopTextSettings {
            font_smoothing_type: 2,
            font_smoothing_gamma: 2000,
            ..base
        }
    }
}

fn resolve_rendering_settings(
    base: RenderingSettings,
    display: &DisplayProfile,
) -> RenderingSettings {
    if display.width >= 3800 || display.scale_percent >= 175 {
        RenderingSettings {
            pixel_structure: 0,
            gamma_level: 1800,
            clear_type_level: 0,
            text_contrast_level: 4,
            ..base
        }
    } else if display.width >= 2500 || display.scale_percent >= 125 {
        base
    } else {
        RenderingSettings {
            pixel_structure: 1,
            gamma_level: 2000,
            clear_type_level: 100,
            text_contrast_level: 6,
            ..base
        }
    }
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
