# Windows Font Tuner

`Windows Font Tuner` is a small WinForms utility for backing up, applying, and restoring Windows font tuning presets.

It is designed for workflows like:

- backing up `FontSubstitutes`, `Desktop`, `Avalon.Graphics`, and `WindowMetrics`
- switching `Segoe UI` / `Segoe UI Variable` aliases to a preferred installed family
- applying grayscale-style text rendering tweaks
- updating desktop icon and legacy UI font metrics
- rebuilding the font cache and restarting Explorer

## What This Tool Does

- loads presets from the `Presets` folder
- warns if a preset needs font families that are not installed
- creates timestamped backups under `%LOCALAPPDATA%\WindowsFontTuner\Backups`
- applies font substitution, rendering, and window metrics settings
- restores the latest backup with one click

## What This Tool Does Not Do

- it does not bundle third-party fonts
- it does not patch private Explorer resources or XAML assets
- it does not guarantee that every Windows 11 shell surface will honor the same settings

## Required Fonts

The included `HarmonyOS Unified` preset expects these font families to already be installed:

- `HarmonyOS Sans SC`
- `HarmonyOS Sans SC Medium`

Official source:

- [HarmonyOS Sans official repository](https://github.com/huawei-fonts/HarmonyOS-Sans)

## Build

This project targets `.NET Framework 4.8` and uses the MSBuild that ships with Windows.

Run:

```bat
build.bat
```

The executable will be created at:

```text
bin\Release\WindowsFontTuner.exe
```

## Run

Run the built exe **as Administrator**.

The app manifest requests elevation automatically because writing `HKLM\...\FontSubstitutes` and rebuilding the font cache require admin privileges.

## Editing Presets

Presets are plain JSON files in the `Presets` folder.

Fields:

- `Name`: label shown in the UI
- `Description`: short explanation
- `RequiredFonts`: installed font families the preset expects
- `FontSubstitutes`: aliases written under `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\FontSubstitutes`
- `DesktopTextSettings`: values written under `HKCU\Control Panel\Desktop`
- `Rendering`: values written to each `HKCU\Software\Microsoft\Avalon.Graphics\DISPLAY*` key
- `WindowMetrics`: face name, weight, and quality applied through `SystemParametersInfo`

## Suggested Open-Source Workflow

1. Create a Git repository in this folder.
2. Add screenshots and more presets.
3. Document official font download links instead of redistributing proprietary font files.
4. Tag releases with the built exe if you want a simple download for non-technical users.

## Notes

- Windows 11 shell typography is split across multiple systems. Some surfaces honor these settings, some only partially do.
- Before applying a preset, the app automatically exports the relevant registry keys into a timestamped backup folder.
