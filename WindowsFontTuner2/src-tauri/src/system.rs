use crate::models::{ActionResult, BootstrapPayload, DisplayProfile, PresetDefinition, PresetStatus, RuntimePresetState};
use anyhow::{anyhow, bail, Context, Result};
use dirs::data_local_dir;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
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

pub fn should_run_headless_recovery() -> bool {
    env::args().any(|arg| arg == "--emergency-reset") || is_shift_pressed()
}

pub fn run_headless_recovery(presets: &[PresetDefinition]) -> Result<()> {
    if !is_admin() {
        run_self_elevated("--emergency-reset")?;
        return Ok(());
    }

    let _ = restore_windows_default_internal(presets)?;
    Ok(())
}

pub fn load_bootstrap(presets: &[PresetDefinition]) -> Result<BootstrapPayload> {
    let display = sniff_display_profile();
    let active_preset_id = detect_active_preset_id(presets)?;
    let active_font_label = resolve_active_font_label(presets, active_preset_id.as_deref())?;
    let runtime_presets = presets
        .iter()
        .map(|preset| RuntimePresetState {
            id: preset.id.to_string(),
            status: preset.status,
            available: preset.status == PresetStatus::Ready && preset.required_fonts.iter().all(|name| font_family_exists(name)),
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
        presets: runtime_presets,
    })
}

pub fn apply_preset(preset_id: &str, presets: &[PresetDefinition]) -> Result<ActionResult> {
    ensure_admin()?;

    let preset = presets
        .iter()
        .find(|preset| preset.id == preset_id)
        .cloned()
        .ok_or_else(|| anyhow!("没找到这套风格。"))?;

    if preset.status != PresetStatus::Ready {
        bail!("这套风格还在打磨，暂时不能直接应用。");
    }

    if !preset.required_fonts.iter().all(|name| font_family_exists(name)) {
        bail!("对应字体包还没就位，下一步我会接上静默下载。");
    }

    let backup_path = create_backup()?;
    write_font_substitutes(&preset)?;
    write_desktop_settings(&preset)?;
    write_rendering_settings(&preset)?;
    notify_system();

    Ok(ActionResult {
        message: "✨ 风格已应用，注销电脑后即可感受。".to_string(),
        backup_path: Some(backup_path.to_string_lossy().to_string()),
        active_preset_id: Some(preset.id.to_string()),
    })
}

pub fn restore_windows_default(presets: &[PresetDefinition]) -> Result<ActionResult> {
    ensure_admin()?;

    let backup_path = restore_windows_default_internal(presets)?;
    Ok(ActionResult {
        message: "已经恢复回 Windows 默认字体链路。".to_string(),
        backup_path: Some(backup_path.to_string_lossy().to_string()),
        active_preset_id: None,
    })
}

pub fn launch_system_font_repair() -> Result<()> {
    let cmd = PathBuf::from(env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string()))
        .join("System32")
        .join("cmd.exe");
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
    reset_desktop_settings()?;
    reset_rendering_settings()?;
    notify_system();
    Ok(backup_path)
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

    let matrix_profile = if width >= 3800 {
        "4K 轻锐矩阵"
    } else if width >= 2500 {
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

fn write_desktop_settings(preset: &PresetDefinition) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .create_subkey(DESKTOP_PATH)
        .context("没能打开桌面字体渲染设置。")?
        .0;

    key.set_value("FontSmoothing", &preset.desktop.font_smoothing)
        .context("写入 FontSmoothing 失败。")?;
    key.set_value("FontSmoothingType", &preset.desktop.font_smoothing_type)
        .context("写入 FontSmoothingType 失败。")?;
    key.set_value("FontSmoothingGamma", &preset.desktop.font_smoothing_gamma)
        .context("写入 FontSmoothingGamma 失败。")?;
    key.set_value(
        "FontSmoothingOrientation",
        &preset.desktop.font_smoothing_orientation,
    )
    .context("写入 FontSmoothingOrientation 失败。")?;

    Ok(())
}

fn reset_desktop_settings() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .create_subkey(DESKTOP_PATH)
        .context("没能打开桌面字体渲染设置。")?
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

fn write_rendering_settings(preset: &PresetDefinition) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let root = match hkcu.open_subkey_with_flags(AVALON_GRAPHICS_PATH, KEY_READ | KEY_WRITE) {
        Ok(root) => root,
        Err(_) => return Ok(()),
    };

    for display_key_name in root.enum_keys().flatten() {
        if let Ok(display_key) = root.open_subkey_with_flags(&display_key_name, KEY_READ | KEY_WRITE) {
            let _ = display_key.set_value("PixelStructure", &preset.rendering.pixel_structure);
            let _ = display_key.set_value("GammaLevel", &preset.rendering.gamma_level);
            let _ = display_key.set_value("ClearTypeLevel", &preset.rendering.clear_type_level);
            let _ = display_key.set_value("TextContrastLevel", &preset.rendering.text_contrast_level);
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
        if let Ok(display_key) = root.open_subkey_with_flags(&display_key_name, KEY_READ | KEY_WRITE) {
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
    let key = match open_hklm_read(FONTS_PATH) {
        Ok(key) => key,
        Err(_) => return false,
    };

    let family_name_lower = family_name.to_lowercase();
    key.enum_values().flatten().any(|(name, value)| {
        let value_text = String::from_utf8_lossy(&value.bytes).to_lowercase();
        name.to_lowercase().contains(&family_name_lower) || value_text.contains(&family_name_lower)
    })
}

fn backup_root() -> PathBuf {
    data_local_dir()
        .unwrap_or_else(env::temp_dir)
        .join("WindowsFontTuner")
        .join("Backups")
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
        r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts",
        target.join("Fonts.reg"),
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

fn open_hklm_read(path: &str) -> Result<RegKey> {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(path, KEY_READ | KEY_WOW64_64KEY)
        .with_context(|| format!("没能读取注册表：{path}"))
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
        bail!("请用管理员权限启动 2.0，再来动系统字体。");
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
    value.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}
