using Microsoft.Win32;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing.Text;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Security.Principal;
using System.ServiceProcess;

namespace WindowsFontTuner
{
    public sealed class FontTunerService
    {
        private const string FontSubstitutesPath = @"SOFTWARE\Microsoft\Windows NT\CurrentVersion\FontSubstitutes";
        private const string DesktopPath = @"Control Panel\Desktop";
        private const string AvalonGraphicsPath = @"Software\Microsoft\Avalon.Graphics";
        private readonly string _backupRoot;

        public FontTunerService()
        {
            _backupRoot = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "WindowsFontTuner",
                "Backups");
        }

        public string BackupRoot
        {
            get { return _backupRoot; }
        }

        public bool IsAdministrator()
        {
            WindowsIdentity identity = WindowsIdentity.GetCurrent();
            WindowsPrincipal principal = new WindowsPrincipal(identity);
            return principal.IsInRole(WindowsBuiltInRole.Administrator);
        }

        public string CreateBackup()
        {
            EnsureAdmin();

            string directory = Path.Combine(_backupRoot, DateTime.Now.ToString("yyyyMMdd-HHmmss"));
            Directory.CreateDirectory(directory);

            ExportRegistry(@"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\FontSubstitutes", Path.Combine(directory, "FontSubstitutes.reg"));
            ExportRegistry(@"HKCU\Control Panel\Desktop", Path.Combine(directory, "Desktop.reg"));
            ExportRegistry(@"HKCU\Software\Microsoft\Avalon.Graphics", Path.Combine(directory, "Avalon.Graphics.reg"));
            ExportRegistry(@"HKCU\Control Panel\Desktop\WindowMetrics", Path.Combine(directory, "WindowMetrics.reg"));

            return directory;
        }

        public string ApplyPreset(FontPreset preset, bool rebuildFontCache, bool restartExplorer)
        {
            if (preset == null)
            {
                throw new ArgumentNullException("preset", "未提供要应用的预设。");
            }

            EnsureAdmin();

            string backupPath = CreateBackup();
            ApplyFontSubstitutes(preset.FontSubstitutes);
            ApplyDesktopTextSettings(preset.DesktopTextSettings);
            ApplyRenderingSettings(preset.Rendering);
            ApplyWindowMetrics(preset.WindowMetrics);
            RefreshSystemState(rebuildFontCache, restartExplorer);
            return backupPath;
        }

        public string RestoreLatestBackup(bool rebuildFontCache, bool restartExplorer)
        {
            EnsureAdmin();

            if (!Directory.Exists(_backupRoot))
            {
                throw new InvalidOperationException("还没有可恢复的备份。");
            }

            DirectoryInfo latest = new DirectoryInfo(_backupRoot)
                .GetDirectories()
                .OrderByDescending(function => function.CreationTimeUtc)
                .FirstOrDefault();

            if (latest == null)
            {
                throw new InvalidOperationException("未找到备份目录。");
            }

            ImportRegistry(Path.Combine(latest.FullName, "FontSubstitutes.reg"));
            ImportRegistry(Path.Combine(latest.FullName, "Desktop.reg"));
            ImportRegistry(Path.Combine(latest.FullName, "Avalon.Graphics.reg"));
            ImportRegistry(Path.Combine(latest.FullName, "WindowMetrics.reg"));
            RefreshSystemState(rebuildFontCache, restartExplorer);
            return latest.FullName;
        }

        public IList<string> GetInstalledFamilies()
        {
            InstalledFontCollection collection = new InstalledFontCollection();
            return collection.Families.Select(function => function.Name).OrderBy(function => function).ToList();
        }

        public IList<string> GetMissingFonts(FontPreset preset)
        {
            if (preset == null || preset.RequiredFonts == null || preset.RequiredFonts.Count == 0)
            {
                return new List<string>();
            }

            HashSet<string> installed = new HashSet<string>(GetInstalledFamilies(), StringComparer.OrdinalIgnoreCase);
            return preset.RequiredFonts
                .Where(function => !installed.Contains(function))
                .OrderBy(function => function)
                .ToList();
        }

        public void OpenDirectory(string path)
        {
            if (string.IsNullOrWhiteSpace(path))
            {
                return;
            }

            if (!Directory.Exists(path))
            {
                Directory.CreateDirectory(path);
            }

            Process.Start("explorer.exe", "\"" + path + "\"");
        }

        private void ApplyFontSubstitutes(Dictionary<string, string> values)
        {
            if (values == null)
            {
                return;
            }

            using (RegistryKey key = Registry.LocalMachine.CreateSubKey(FontSubstitutesPath))
            {
                foreach (KeyValuePair<string, string> pair in values)
                {
                    key.SetValue(pair.Key, pair.Value, RegistryValueKind.String);
                }
            }
        }

        private void ApplyDesktopTextSettings(DesktopTextSettings settings)
        {
            if (settings == null)
            {
                return;
            }

            using (RegistryKey key = Registry.CurrentUser.CreateSubKey(DesktopPath))
            {
                key.SetValue("FontSmoothing", string.IsNullOrWhiteSpace(settings.FontSmoothing) ? "2" : settings.FontSmoothing, RegistryValueKind.String);
                key.SetValue("FontSmoothingType", settings.FontSmoothingType, RegistryValueKind.DWord);
                key.SetValue("FontSmoothingGamma", settings.FontSmoothingGamma, RegistryValueKind.DWord);
                key.SetValue("FontSmoothingOrientation", settings.FontSmoothingOrientation, RegistryValueKind.DWord);
            }
        }

        private void ApplyRenderingSettings(RenderingSettings settings)
        {
            if (settings == null)
            {
                return;
            }

            using (RegistryKey root = Registry.CurrentUser.OpenSubKey(AvalonGraphicsPath, true))
            {
                if (root == null)
                {
                    return;
                }

                foreach (string name in root.GetSubKeyNames())
                {
                    using (RegistryKey displayKey = root.OpenSubKey(name, true))
                    {
                        if (displayKey == null)
                        {
                            continue;
                        }

                        displayKey.SetValue("PixelStructure", settings.PixelStructure, RegistryValueKind.DWord);
                        displayKey.SetValue("GammaLevel", settings.GammaLevel, RegistryValueKind.DWord);
                        displayKey.SetValue("ClearTypeLevel", settings.ClearTypeLevel, RegistryValueKind.DWord);
                        displayKey.SetValue("TextContrastLevel", settings.TextContrastLevel, RegistryValueKind.DWord);
                    }
                }
            }
        }

        private void ApplyWindowMetrics(WindowMetricsSettings settings)
        {
            if (settings == null || string.IsNullOrWhiteSpace(settings.FaceName))
            {
                return;
            }

            NativeMethods.LOGFONT iconFont = new NativeMethods.LOGFONT();
            bool gotIconFont = NativeMethods.SystemParametersInfo(
                NativeMethods.SPI_GETICONTITLELOGFONT,
                (uint)Marshal.SizeOf(typeof(NativeMethods.LOGFONT)),
                ref iconFont,
                0);

            if (!gotIconFont)
            {
                throw new InvalidOperationException("无法读取桌面图标字体设置。");
            }

            iconFont.lfFaceName = settings.FaceName;
            iconFont.lfWeight = settings.Weight;
            iconFont.lfQuality = (byte)settings.Quality;

            bool setIconFont = NativeMethods.SystemParametersInfo(
                NativeMethods.SPI_SETICONTITLELOGFONT,
                (uint)Marshal.SizeOf(typeof(NativeMethods.LOGFONT)),
                ref iconFont,
                NativeMethods.SPIF_SENDCHANGE | NativeMethods.SPIF_UPDATEINIFILE);

            if (!setIconFont)
            {
                throw new InvalidOperationException("无法更新桌面图标字体设置。");
            }

            NativeMethods.NONCLIENTMETRICS metrics = new NativeMethods.NONCLIENTMETRICS();
            metrics.cbSize = (uint)Marshal.SizeOf(typeof(NativeMethods.NONCLIENTMETRICS));

            bool gotMetrics = NativeMethods.SystemParametersInfo(
                NativeMethods.SPI_GETNONCLIENTMETRICS,
                metrics.cbSize,
                ref metrics,
                0);

            if (!gotMetrics)
            {
                throw new InvalidOperationException("无法读取窗口字体参数。");
            }

            UpdateLogFont(ref metrics.lfCaptionFont, settings);
            UpdateLogFont(ref metrics.lfSmCaptionFont, settings);
            UpdateLogFont(ref metrics.lfMenuFont, settings);
            UpdateLogFont(ref metrics.lfStatusFont, settings);
            UpdateLogFont(ref metrics.lfMessageFont, settings);

            bool setMetrics = NativeMethods.SystemParametersInfo(
                NativeMethods.SPI_SETNONCLIENTMETRICS,
                metrics.cbSize,
                ref metrics,
                NativeMethods.SPIF_SENDCHANGE | NativeMethods.SPIF_UPDATEINIFILE);

            if (!setMetrics)
            {
                throw new InvalidOperationException("无法更新窗口字体参数。");
            }
        }

        private static void UpdateLogFont(ref NativeMethods.LOGFONT logFont, WindowMetricsSettings settings)
        {
            logFont.lfFaceName = settings.FaceName;
            logFont.lfWeight = settings.Weight;
            logFont.lfQuality = (byte)settings.Quality;
        }

        private void RefreshSystemState(bool rebuildFontCache, bool restartExplorer)
        {
            BroadcastSettingsChanged();

            if (rebuildFontCache)
            {
                RebuildFontCache();
            }

            if (restartExplorer)
            {
                RestartExplorer();
            }

            BroadcastSettingsChanged();
        }

        private static void RebuildFontCache()
        {
            try
            {
                using (ServiceController controller = new ServiceController("FontCache"))
                {
                    if (controller.Status != ServiceControllerStatus.Stopped &&
                        controller.Status != ServiceControllerStatus.StopPending)
                    {
                        controller.Stop();
                        controller.WaitForStatus(ServiceControllerStatus.Stopped, TimeSpan.FromSeconds(10));
                    }
                }
            }
            catch
            {
            }

            DeleteFiles(@"C:\Windows\ServiceProfiles\LocalService\AppData\Local\FontCache");
            DeleteFile(@"C:\Windows\System32\FNTCACHE.DAT");

            try
            {
                using (ServiceController controller = new ServiceController("FontCache"))
                {
                    if (controller.Status != ServiceControllerStatus.Running &&
                        controller.Status != ServiceControllerStatus.StartPending)
                    {
                        controller.Start();
                        controller.WaitForStatus(ServiceControllerStatus.Running, TimeSpan.FromSeconds(10));
                    }
                }
            }
            catch
            {
            }
        }

        private static void DeleteFiles(string directory)
        {
            if (!Directory.Exists(directory))
            {
                return;
            }

            foreach (string file in Directory.GetFiles(directory))
            {
                DeleteFile(file);
            }
        }

        private static void DeleteFile(string file)
        {
            try
            {
                if (File.Exists(file))
                {
                    File.SetAttributes(file, FileAttributes.Normal);
                    File.Delete(file);
                }
            }
            catch
            {
            }
        }

        private static void RestartExplorer()
        {
            foreach (Process process in Process.GetProcessesByName("explorer"))
            {
                try
                {
                    process.Kill();
                    process.WaitForExit(5000);
                }
                catch
                {
                }
            }

            Process.Start("explorer.exe");
        }

        private static void BroadcastSettingsChanged()
        {
            UIntPtr result = UIntPtr.Zero;
            NativeMethods.SendMessageTimeout(
                NativeMethods.HWND_BROADCAST,
                NativeMethods.WM_SETTINGCHANGE,
                UIntPtr.Zero,
                "Windows",
                NativeMethods.SMTO_ABORTIFHUNG,
                5000,
                out result);
        }

        private static void ExportRegistry(string registryPath, string destinationFile)
        {
            RunProcess(Path.Combine(Environment.SystemDirectory, "reg.exe"), string.Format("export \"{0}\" \"{1}\" /y", registryPath, destinationFile));
        }

        private static void ImportRegistry(string file)
        {
            if (!File.Exists(file))
            {
                return;
            }

            RunProcess(Path.Combine(Environment.SystemDirectory, "reg.exe"), string.Format("import \"{0}\"", file));
        }

        private static void RunProcess(string fileName, string arguments)
        {
            ProcessStartInfo startInfo = new ProcessStartInfo();
            startInfo.FileName = fileName;
            startInfo.Arguments = arguments;
            startInfo.CreateNoWindow = true;
            startInfo.UseShellExecute = false;
            startInfo.RedirectStandardOutput = true;
            startInfo.RedirectStandardError = true;

            using (Process process = Process.Start(startInfo))
            {
                string output = process.StandardOutput.ReadToEnd();
                string error = process.StandardError.ReadToEnd();
                process.WaitForExit();

                if (process.ExitCode != 0)
                {
                    string message = string.IsNullOrWhiteSpace(error) ? output : error;
                    throw new InvalidOperationException(message.Trim());
                }
            }
        }

        private void EnsureAdmin()
        {
            if (!IsAdministrator())
            {
                throw new InvalidOperationException("请右键以管理员身份运行本工具。");
            }
        }

        internal static class NativeMethods
        {
            public const uint SPI_GETICONTITLELOGFONT = 0x001F;
            public const uint SPI_SETICONTITLELOGFONT = 0x0022;
            public const uint SPI_GETNONCLIENTMETRICS = 0x0029;
            public const uint SPI_SETNONCLIENTMETRICS = 0x002A;
            public const uint SPIF_UPDATEINIFILE = 0x0001;
            public const uint SPIF_SENDCHANGE = 0x0002;
            public const uint WM_SETTINGCHANGE = 0x001A;
            public const uint SMTO_ABORTIFHUNG = 0x0002;
            public static readonly IntPtr HWND_BROADCAST = new IntPtr(0xffff);

            [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
            public struct LOGFONT
            {
                public int lfHeight;
                public int lfWidth;
                public int lfEscapement;
                public int lfOrientation;
                public int lfWeight;
                public byte lfItalic;
                public byte lfUnderline;
                public byte lfStrikeOut;
                public byte lfCharSet;
                public byte lfOutPrecision;
                public byte lfClipPrecision;
                public byte lfQuality;
                public byte lfPitchAndFamily;

                [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
                public string lfFaceName;
            }

            [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
            public struct NONCLIENTMETRICS
            {
                public uint cbSize;
                public int iBorderWidth;
                public int iScrollWidth;
                public int iScrollHeight;
                public int iCaptionWidth;
                public int iCaptionHeight;
                public LOGFONT lfCaptionFont;
                public int iSmCaptionWidth;
                public int iSmCaptionHeight;
                public LOGFONT lfSmCaptionFont;
                public int iMenuWidth;
                public int iMenuHeight;
                public LOGFONT lfMenuFont;
                public LOGFONT lfStatusFont;
                public LOGFONT lfMessageFont;
                public int iPaddedBorderWidth;
            }

            [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
            public static extern bool SystemParametersInfo(uint uiAction, uint uiParam, ref LOGFONT pvParam, uint fWinIni);

            [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
            public static extern bool SystemParametersInfo(uint uiAction, uint uiParam, ref NONCLIENTMETRICS pvParam, uint fWinIni);

            [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
            public static extern IntPtr SendMessageTimeout(
                IntPtr hWnd,
                uint msg,
                UIntPtr wParam,
                string lParam,
                uint fuFlags,
                uint uTimeout,
                out UIntPtr lpdwResult);
        }
    }
}
