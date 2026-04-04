using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Windows.Forms;

namespace WindowsFontTuner
{
    internal static class UiPalette
    {
        public static readonly Color WindowBackground = Color.FromArgb(240, 245, 251);
        public static readonly Color CardBackground = Color.FromArgb(232, 252, 253, 255);
        public static readonly Color Border = Color.FromArgb(214, 225, 239);
        public static readonly Color Accent = Color.FromArgb(27, 108, 236);
        public static readonly Color AccentHover = Color.FromArgb(18, 95, 217);
        public static readonly Color AccentPressed = Color.FromArgb(12, 80, 191);
        public static readonly Color AccentSoft = Color.FromArgb(228, 239, 255);
        public static readonly Color Secondary = Color.FromArgb(245, 248, 252);
        public static readonly Color SecondaryHover = Color.FromArgb(236, 242, 250);
        public static readonly Color SecondaryPressed = Color.FromArgb(226, 235, 247);
        public static readonly Color TextPrimary = Color.FromArgb(26, 33, 52);
        public static readonly Color TextSecondary = Color.FromArgb(84, 96, 122);
        public static readonly Color TextMuted = Color.FromArgb(113, 125, 148);
        public static readonly Color Success = Color.FromArgb(14, 136, 95);
        public static readonly Color Warning = Color.FromArgb(191, 112, 16);
        public static readonly Color Danger = Color.FromArgb(184, 64, 64);
        public static readonly Color HeroStart = Color.FromArgb(235, 244, 255);
        public static readonly Color HeroEnd = Color.FromArgb(249, 252, 255);
    }

    internal static class UiGeometry
    {
        public static GraphicsPath CreateRoundedRectangle(Rectangle bounds, int radius)
        {
            GraphicsPath path = new GraphicsPath();

            if (radius <= 0)
            {
                path.AddRectangle(bounds);
                path.CloseFigure();
                return path;
            }

            int diameter = radius * 2;
            Rectangle arc = new Rectangle(bounds.Location, new Size(diameter, diameter));

            path.AddArc(arc, 180, 90);
            arc.X = bounds.Right - diameter;
            path.AddArc(arc, 270, 90);
            arc.Y = bounds.Bottom - diameter;
            path.AddArc(arc, 0, 90);
            arc.X = bounds.Left;
            path.AddArc(arc, 90, 90);
            path.CloseFigure();
            return path;
        }
    }

    public sealed class ModernCardPanel : Panel
    {
        private int _cornerRadius = 26;
        private bool _useGradient;
        private bool _showGlow;
        private bool _showShadow = true;
        private Color _fillColor = UiPalette.CardBackground;
        private Color _gradientStartColor = UiPalette.HeroStart;
        private Color _gradientEndColor = UiPalette.HeroEnd;
        private Color _borderColor = UiPalette.Border;

        public ModernCardPanel()
        {
            DoubleBuffered = true;
            ResizeRedraw = true;
            BackColor = Color.Transparent;
            Padding = new Padding(20);
        }

        public int CornerRadius
        {
            get { return _cornerRadius; }
            set
            {
                _cornerRadius = value;
                UpdateRegion();
                Invalidate();
            }
        }

        public bool UseGradient
        {
            get { return _useGradient; }
            set
            {
                _useGradient = value;
                Invalidate();
            }
        }

        public bool ShowGlow
        {
            get { return _showGlow; }
            set
            {
                _showGlow = value;
                Invalidate();
            }
        }

        public bool ShowShadow
        {
            get { return _showShadow; }
            set
            {
                _showShadow = value;
                Invalidate();
            }
        }

        public Color FillColor
        {
            get { return _fillColor; }
            set
            {
                _fillColor = value;
                Invalidate();
            }
        }

        public Color GradientStartColor
        {
            get { return _gradientStartColor; }
            set
            {
                _gradientStartColor = value;
                Invalidate();
            }
        }

        public Color GradientEndColor
        {
            get { return _gradientEndColor; }
            set
            {
                _gradientEndColor = value;
                Invalidate();
            }
        }

        public Color BorderColor
        {
            get { return _borderColor; }
            set
            {
                _borderColor = value;
                Invalidate();
            }
        }

        protected override void OnResize(EventArgs eventargs)
        {
            base.OnResize(eventargs);
            UpdateRegion();
        }

        protected override void OnPaint(PaintEventArgs e)
        {
            base.OnPaint(e);

            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            Rectangle bounds = new Rectangle(0, 0, Width - 1, Height - 1);

            using (GraphicsPath path = UiGeometry.CreateRoundedRectangle(bounds, _cornerRadius))
            {
                if (_showShadow)
                {
                    Rectangle shadowBounds = new Rectangle(0, 6, Width - 1, Height - 7);
                    using (GraphicsPath shadowPath = UiGeometry.CreateRoundedRectangle(shadowBounds, _cornerRadius))
                    using (SolidBrush shadowBrush = new SolidBrush(Color.FromArgb(18, 36, 53, 79)))
                    {
                        e.Graphics.FillPath(shadowBrush, shadowPath);
                    }
                }

                if (_useGradient)
                {
                    using (LinearGradientBrush brush = new LinearGradientBrush(bounds, _gradientStartColor, _gradientEndColor, LinearGradientMode.ForwardDiagonal))
                    {
                        e.Graphics.FillPath(brush, path);
                    }
                }
                else
                {
                    using (SolidBrush brush = new SolidBrush(_fillColor))
                    {
                        e.Graphics.FillPath(brush, path);
                    }
                }

                if (_showGlow)
                {
                    using (SolidBrush glow = new SolidBrush(Color.FromArgb(70, 255, 255, 255)))
                    {
                        e.Graphics.FillEllipse(glow, new Rectangle(Width - 180, -30, 210, 210));
                        e.Graphics.FillEllipse(glow, new Rectangle(Width - 110, 18, 120, 120));
                    }

                    using (SolidBrush accent = new SolidBrush(Color.FromArgb(44, UiPalette.Accent)))
                    {
                        e.Graphics.FillEllipse(accent, new Rectangle(-38, Height - 110, 145, 145));
                    }
                }

                using (Pen border = new Pen(_borderColor))
                {
                    e.Graphics.DrawPath(border, path);
                }
            }
        }

        private void UpdateRegion()
        {
            if (Width <= 0 || Height <= 0)
            {
                return;
            }

            using (GraphicsPath path = UiGeometry.CreateRoundedRectangle(new Rectangle(0, 0, Width, Height), _cornerRadius))
            {
                Region = new Region(path);
            }
        }
    }

    public enum ModernButtonStyle
    {
        Primary,
        Secondary,
        Ghost
    }

    public sealed class ModernButton : Button
    {
        private ModernButtonStyle _buttonStyle;
        private bool _hovered;
        private bool _pressed;

        public ModernButton()
        {
            FlatStyle = FlatStyle.Flat;
            FlatAppearance.BorderSize = 0;
            TabStop = true;
            Cursor = Cursors.Hand;
            DoubleBuffered = true;
            Height = 40;
            Width = 132;
            Font = new Font(SystemFonts.MessageBoxFont.FontFamily, 9.25f, FontStyle.Bold);
            ForeColor = UiPalette.TextPrimary;
            BackColor = Color.Transparent;
        }

        public int CornerRadius { get; set; }

        public ModernButtonStyle ButtonStyle
        {
            get { return _buttonStyle; }
            set
            {
                _buttonStyle = value;
                Invalidate();
            }
        }

        protected override void OnMouseEnter(EventArgs e)
        {
            base.OnMouseEnter(e);
            _hovered = true;
            Invalidate();
        }

        protected override void OnMouseLeave(EventArgs e)
        {
            base.OnMouseLeave(e);
            _hovered = false;
            _pressed = false;
            Invalidate();
        }

        protected override void OnMouseDown(MouseEventArgs mevent)
        {
            base.OnMouseDown(mevent);
            _pressed = true;
            Invalidate();
        }

        protected override void OnMouseUp(MouseEventArgs mevent)
        {
            base.OnMouseUp(mevent);
            _pressed = false;
            Invalidate();
        }

        protected override void OnPaint(PaintEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            e.Graphics.Clear(Parent == null ? UiPalette.WindowBackground : Parent.BackColor);

            Rectangle bounds = new Rectangle(0, 0, Width - 1, Height - 1);
            Color fill;
            Color border;
            Color textColor;

            ResolveColors(out fill, out border, out textColor);

            using (GraphicsPath path = UiGeometry.CreateRoundedRectangle(bounds, CornerRadius <= 0 ? Height / 2 : CornerRadius))
            using (SolidBrush brush = new SolidBrush(fill))
            using (Pen pen = new Pen(border))
            {
                e.Graphics.FillPath(brush, path);
                e.Graphics.DrawPath(pen, path);
            }

            TextRenderer.DrawText(
                e.Graphics,
                Text,
                Font,
                bounds,
                textColor,
                TextFormatFlags.HorizontalCenter | TextFormatFlags.VerticalCenter | TextFormatFlags.EndEllipsis);
        }

        private void ResolveColors(out Color fill, out Color border, out Color textColor)
        {
            if (!Enabled)
            {
                fill = Color.FromArgb(236, 240, 246);
                border = Color.FromArgb(220, 226, 236);
                textColor = UiPalette.TextMuted;
                return;
            }

            if (_buttonStyle == ModernButtonStyle.Primary)
            {
                fill = _pressed ? UiPalette.AccentPressed : (_hovered ? UiPalette.AccentHover : UiPalette.Accent);
                border = fill;
                textColor = Color.White;
                return;
            }

            if (_buttonStyle == ModernButtonStyle.Ghost)
            {
                fill = _pressed ? Color.FromArgb(224, 233, 245) : (_hovered ? Color.FromArgb(237, 243, 251) : Color.Transparent);
                border = _hovered || _pressed ? UiPalette.Border : Color.Transparent;
                textColor = UiPalette.TextSecondary;
                return;
            }

            fill = _pressed ? UiPalette.SecondaryPressed : (_hovered ? UiPalette.SecondaryHover : UiPalette.Secondary);
            border = UiPalette.Border;
            textColor = UiPalette.TextPrimary;
        }
    }
}
