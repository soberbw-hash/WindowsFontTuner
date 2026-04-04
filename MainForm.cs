using System;
using System.Collections.Generic;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Web.Script.Serialization;
using System.Windows.Forms;

namespace WindowsFontTuner
{
    public sealed class MainForm : Form
    {
        private readonly FontTunerService _service;
        private readonly ComboBox _presetComboBox;
        private readonly Label _descriptionLabel;
        private readonly Label _fontHintLabel;
        private readonly Label _adminLabel;
        private readonly CheckBox _rebuildCacheCheckBox;
        private readonly CheckBox _restartExplorerCheckBox;
        private readonly TextBox _logTextBox;
        private List<FontPreset> _presets;

        public MainForm()
        {
            _service = new FontTunerService();
            _presets = new List<FontPreset>();

            Text = "Windows字体调谐器";
            Width = 920;
            Height = 700;
            MinimumSize = new Size(760, 560);
            StartPosition = FormStartPosition.CenterScreen;

            TableLayoutPanel root = new TableLayoutPanel();
            root.Dock = DockStyle.Fill;
            root.Padding = new Padding(16);
            root.ColumnCount = 1;
            root.RowCount = 8;
            root.RowStyles.Add(new RowStyle(SizeType.AutoSize));
            root.RowStyles.Add(new RowStyle(SizeType.AutoSize));
            root.RowStyles.Add(new RowStyle(SizeType.AutoSize));
            root.RowStyles.Add(new RowStyle(SizeType.AutoSize));
            root.RowStyles.Add(new RowStyle(SizeType.AutoSize));
            root.RowStyles.Add(new RowStyle(SizeType.AutoSize));
            root.RowStyles.Add(new RowStyle(SizeType.AutoSize));
            root.RowStyles.Add(new RowStyle(SizeType.Percent, 100f));
            Controls.Add(root);

            Label titleLabel = new Label();
            titleLabel.AutoSize = true;
            titleLabel.Font = new Font(Font, FontStyle.Bold);
            titleLabel.Text = "开源可扩展的 Windows 字体调谐工具";
            root.Controls.Add(titleLabel, 0, 0);

            _adminLabel = new Label();
            _adminLabel.AutoSize = true;
            _adminLabel.Margin = new Padding(0, 8, 0, 0);
            root.Controls.Add(_adminLabel, 0, 1);

            Panel presetPanel = new Panel();
            presetPanel.Height = 36;
            presetPanel.Dock = DockStyle.Top;
            presetPanel.Margin = new Padding(0, 12, 0, 0);
            root.Controls.Add(presetPanel, 0, 2);

            Label presetLabel = new Label();
            presetLabel.Text = "预设：";
            presetLabel.AutoSize = true;
            presetLabel.Location = new Point(0, 10);
            presetPanel.Controls.Add(presetLabel);

            _presetComboBox = new ComboBox();
            _presetComboBox.DropDownStyle = ComboBoxStyle.DropDownList;
            _presetComboBox.Width = 340;
            _presetComboBox.Location = new Point(56, 6);
            _presetComboBox.SelectedIndexChanged += PresetComboBox_SelectedIndexChanged;
            presetPanel.Controls.Add(_presetComboBox);

            FlowLayoutPanel actions = new FlowLayoutPanel();
            actions.AutoSize = true;
            actions.WrapContents = true;
            actions.Margin = new Padding(0, 12, 0, 0);
            root.Controls.Add(actions, 0, 3);

            actions.Controls.Add(BuildButton("应用当前预设", ApplyButton_Click));
            actions.Controls.Add(BuildButton("创建备份", BackupButton_Click));
            actions.Controls.Add(BuildButton("恢复最近备份", RestoreButton_Click));
            actions.Controls.Add(BuildButton("打开备份目录", OpenBackupsButton_Click));
            actions.Controls.Add(BuildButton("打开预设目录", OpenPresetsButton_Click));

            FlowLayoutPanel optionsPanel = new FlowLayoutPanel();
            optionsPanel.AutoSize = true;
            optionsPanel.WrapContents = true;
            optionsPanel.Margin = new Padding(0, 12, 0, 0);
            root.Controls.Add(optionsPanel, 0, 4);

            _rebuildCacheCheckBox = new CheckBox();
            _rebuildCacheCheckBox.Text = "重建字体缓存";
            _rebuildCacheCheckBox.Checked = true;
            _rebuildCacheCheckBox.AutoSize = true;
            optionsPanel.Controls.Add(_rebuildCacheCheckBox);

            _restartExplorerCheckBox = new CheckBox();
            _restartExplorerCheckBox.Text = "应用后重启资源管理器";
            _restartExplorerCheckBox.Checked = true;
            _restartExplorerCheckBox.AutoSize = true;
            optionsPanel.Controls.Add(_restartExplorerCheckBox);

            _descriptionLabel = new Label();
            _descriptionLabel.AutoSize = true;
            _descriptionLabel.MaximumSize = new Size(860, 0);
            _descriptionLabel.Margin = new Padding(0, 12, 0, 0);
            root.Controls.Add(_descriptionLabel, 0, 5);

            _fontHintLabel = new Label();
            _fontHintLabel.AutoSize = true;
            _fontHintLabel.MaximumSize = new Size(860, 0);
            _fontHintLabel.Margin = new Padding(0, 8, 0, 0);
            root.Controls.Add(_fontHintLabel, 0, 6);

            _logTextBox = new TextBox();
            _logTextBox.Multiline = true;
            _logTextBox.ScrollBars = ScrollBars.Vertical;
            _logTextBox.Dock = DockStyle.Fill;
            _logTextBox.Margin = new Padding(0, 12, 0, 0);
            _logTextBox.ReadOnly = true;
            _logTextBox.BackColor = Color.White;
            root.Controls.Add(_logTextBox, 0, 7);

            Load += MainForm_Load;
        }

        private static Button BuildButton(string text, EventHandler onClick)
        {
            Button button = new Button();
            button.Text = text;
            button.AutoSize = true;
            button.Padding = new Padding(10, 4, 10, 4);
            button.Click += onClick;
            return button;
        }

        private void MainForm_Load(object sender, EventArgs e)
        {
            _adminLabel.Text = _service.IsAdministrator()
                ? "管理员模式：已启用"
                : "管理员模式：未启用。应用修改前请右键以管理员身份运行。";

            LoadPresets();
            Log("备份目录：" + _service.BackupRoot);
            Log("本工具不内置字体，请先安装官方字体，再应用预设。");
        }

        private void LoadPresets()
        {
            string presetDirectory = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Presets");
            List<FontPreset> presets = new List<FontPreset>();
            JavaScriptSerializer serializer = new JavaScriptSerializer();

            if (Directory.Exists(presetDirectory))
            {
                foreach (string file in Directory.GetFiles(presetDirectory, "*.json"))
                {
                    FontPreset preset = serializer.Deserialize<FontPreset>(File.ReadAllText(file));
                    preset.SourcePath = file;
                    presets.Add(preset);
                }
            }

            _presets = presets.OrderBy(function => function.Name).ToList();
            _presetComboBox.Items.Clear();

            foreach (FontPreset preset in _presets)
            {
                _presetComboBox.Items.Add(preset);
            }

            if (_presetComboBox.Items.Count > 0)
            {
                _presetComboBox.SelectedIndex = 0;
            }
        }

        private FontPreset GetSelectedPreset()
        {
            return _presetComboBox.SelectedItem as FontPreset;
        }

        private void PresetComboBox_SelectedIndexChanged(object sender, EventArgs e)
        {
            FontPreset preset = GetSelectedPreset();

            if (preset == null)
            {
                _descriptionLabel.Text = "未选择预设。";
                _fontHintLabel.Text = string.Empty;
                return;
            }

            _descriptionLabel.Text = preset.Description ?? string.Empty;

            IList<string> missing = _service.GetMissingFonts(preset);
            if (missing.Count == 0)
            {
                _fontHintLabel.Text = "已检测到所需字体：" + string.Join(", ", preset.RequiredFonts ?? new List<string>());
            }
            else
            {
                _fontHintLabel.Text = "缺少所需字体：" + string.Join(", ", missing);
            }
        }

        private void ApplyButton_Click(object sender, EventArgs e)
        {
            FontPreset preset = GetSelectedPreset();

            if (preset == null)
            {
                MessageBox.Show(this, "请先选择一个预设。", "Windows字体调谐器", MessageBoxButtons.OK, MessageBoxIcon.Information);
                return;
            }

            IList<string> missing = _service.GetMissingFonts(preset);
            if (missing.Count > 0)
            {
                DialogResult proceed = MessageBox.Show(
                    this,
                    "检测到以下字体未安装：\r\n\r\n" + string.Join("\r\n", missing) + "\r\n\r\n仍然继续吗？",
                    "缺少字体",
                    MessageBoxButtons.YesNo,
                    MessageBoxIcon.Warning);

                if (proceed != DialogResult.Yes)
                {
                    return;
                }
            }

            RunAction("应用预设", delegate
            {
                string backup = _service.ApplyPreset(
                    preset,
                    _rebuildCacheCheckBox.Checked,
                    _restartExplorerCheckBox.Checked);

                Log("已应用预设：" + preset.Name);
                Log("已创建备份：" + backup);
            });
        }

        private void BackupButton_Click(object sender, EventArgs e)
        {
            RunAction("创建备份", delegate
            {
                string backup = _service.CreateBackup();
                Log("已创建备份：" + backup);
            });
        }

        private void RestoreButton_Click(object sender, EventArgs e)
        {
            DialogResult confirm = MessageBox.Show(
                this,
                "要恢复最近一次备份并刷新资源管理器吗？",
                "恢复最近备份",
                MessageBoxButtons.YesNo,
                MessageBoxIcon.Question);

            if (confirm != DialogResult.Yes)
            {
                return;
            }

            RunAction("恢复最近备份", delegate
            {
                string restored = _service.RestoreLatestBackup(
                    _rebuildCacheCheckBox.Checked,
                    _restartExplorerCheckBox.Checked);

                Log("已恢复备份：" + restored);
            });
        }

        private void OpenBackupsButton_Click(object sender, EventArgs e)
        {
            _service.OpenDirectory(_service.BackupRoot);
        }

        private void OpenPresetsButton_Click(object sender, EventArgs e)
        {
            _service.OpenDirectory(Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Presets"));
        }

        private void RunAction(string actionName, Action action)
        {
            Cursor previous = Cursor.Current;

            try
            {
                Cursor.Current = Cursors.WaitCursor;
                Enabled = false;
                Application.DoEvents();
                action();
                Log(actionName + "完成。");
            }
            catch (Exception ex)
            {
                Log(actionName + "失败：" + ex.Message);
                MessageBox.Show(this, ex.Message, actionName, MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
            finally
            {
                Enabled = true;
                Cursor.Current = previous;
            }
        }

        private void Log(string message)
        {
            string line = DateTime.Now.ToString("HH:mm:ss") + "  " + message;
            _logTextBox.AppendText(line + Environment.NewLine);
        }
    }
}
