Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
$iconsDir = Join-Path $projectRoot "src-tauri\icons"
$assetsDir = Join-Path $projectRoot "src\assets"

function New-RoundedRectPath {
  param(
    [System.Drawing.RectangleF]$Rect,
    [single]$Radius
  )

  $path = New-Object System.Drawing.Drawing2D.GraphicsPath
  $diameter = $Radius * 2

  $path.AddArc($Rect.X, $Rect.Y, $diameter, $diameter, 180, 90)
  $path.AddArc($Rect.Right - $diameter, $Rect.Y, $diameter, $diameter, 270, 90)
  $path.AddArc($Rect.Right - $diameter, $Rect.Bottom - $diameter, $diameter, $diameter, 0, 90)
  $path.AddArc($Rect.X, $Rect.Bottom - $diameter, $diameter, $diameter, 90, 90)
  $path.CloseFigure()

  return $path
}

function New-AppIconBitmap {
  param(
    [int]$Size
  )

  $bitmap = New-Object System.Drawing.Bitmap($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $graphics.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
  $graphics.Clear([System.Drawing.Color]::Transparent)

  $shadowRect = New-Object System.Drawing.RectangleF(($Size * 0.08), ($Size * 0.1), ($Size * 0.84), ($Size * 0.84))
  $iconRect = New-Object System.Drawing.RectangleF(($Size * 0.07), ($Size * 0.06), ($Size * 0.86), ($Size * 0.86))
  $radius = [single]($Size * 0.23)

  $shadowPath = New-RoundedRectPath -Rect $shadowRect -Radius $radius
  $shadowBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(34, 36, 75, 146))
  $graphics.FillPath($shadowBrush, $shadowPath)

  $iconPath = New-RoundedRectPath -Rect $iconRect -Radius $radius
  $mainBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    ([System.Drawing.PointF]::new($iconRect.Left, $iconRect.Top)),
    ([System.Drawing.PointF]::new($iconRect.Right, $iconRect.Bottom)),
    [System.Drawing.Color]::FromArgb(255, 75, 124, 247),
    [System.Drawing.Color]::FromArgb(255, 128, 186, 255)
  )
  $graphics.FillPath($mainBrush, $iconPath)

  $sheenPath = New-RoundedRectPath -Rect ([System.Drawing.RectangleF]::new($iconRect.X, $iconRect.Y, $iconRect.Width, $iconRect.Height * 0.58)) -Radius $radius
  $sheenBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    ([System.Drawing.PointF]::new($iconRect.Left, $iconRect.Top)),
    ([System.Drawing.PointF]::new($iconRect.Left, $iconRect.Top + ($iconRect.Height * 0.58))),
    [System.Drawing.Color]::FromArgb(82, 255, 255, 255),
    [System.Drawing.Color]::FromArgb(8, 255, 255, 255)
  )
  $graphics.FillPath($sheenBrush, $sheenPath)

  $glowBrush = New-Object System.Drawing.Drawing2D.PathGradientBrush($iconPath)
  $glowBrush.CenterColor = [System.Drawing.Color]::FromArgb(0, 255, 255, 255)
  $glowBrush.SurroundColors = @([System.Drawing.Color]::FromArgb(22, 7, 28, 82))
  $graphics.FillPath($glowBrush, $iconPath)

  $borderPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(92, 255, 255, 255), [single]($Size * 0.016))
  $graphics.DrawPath($borderPen, $iconPath)

  $innerRect = New-Object System.Drawing.RectangleF(($Size * 0.13), ($Size * 0.12), ($Size * 0.74), ($Size * 0.74))
  $highlightPath = New-RoundedRectPath -Rect $innerRect -Radius ([single]($Size * 0.18))
  $highlightPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(28, 255, 255, 255), [single]($Size * 0.008))
  $graphics.DrawPath($highlightPen, $highlightPath)

  $letterShadowBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(40, 16, 36, 78))
  $letterBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(248, 255, 255, 255))
  $fontSize = [single]($Size * 0.34)
  $font = New-Object System.Drawing.Font("Segoe UI", $fontSize, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
  $format = New-Object System.Drawing.StringFormat
  $format.Alignment = [System.Drawing.StringAlignment]::Center
  $format.LineAlignment = [System.Drawing.StringAlignment]::Center

  $textRect = [System.Drawing.RectangleF]::new($Size * 0.09, $Size * 0.08, $Size * 0.72, $Size * 0.72)
  $shadowRectText = [System.Drawing.RectangleF]::new($textRect.X + ($Size * 0.012), $textRect.Y + ($Size * 0.012), $textRect.Width, $textRect.Height)
  $graphics.DrawString("Aa", $font, $letterShadowBrush, $shadowRectText, $format)
  $graphics.DrawString("Aa", $font, $letterBrush, $textRect, $format)

  $sparkBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(240, 255, 246, 197))
  $graphics.FillEllipse($sparkBrush, $Size * 0.72, $Size * 0.2, $Size * 0.07, $Size * 0.07)
  $graphics.FillEllipse($sparkBrush, $Size * 0.78, $Size * 0.16, $Size * 0.03, $Size * 0.03)

  $sparkPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(210, 255, 247, 214), [single]($Size * 0.008))
  $graphics.DrawLine($sparkPen, $Size * 0.76, $Size * 0.14, $Size * 0.76, $Size * 0.26)
  $graphics.DrawLine($sparkPen, $Size * 0.7, $Size * 0.2, $Size * 0.82, $Size * 0.2)

  $sparkPen.Dispose()
  $sparkBrush.Dispose()
  $letterBrush.Dispose()
  $letterShadowBrush.Dispose()
  $font.Dispose()
  $format.Dispose()
  $highlightPen.Dispose()
  $glowBrush.Dispose()
  $borderPen.Dispose()
  $sheenBrush.Dispose()
  $mainBrush.Dispose()
  $shadowBrush.Dispose()
  $highlightPath.Dispose()
  $iconPath.Dispose()
  $shadowPath.Dispose()
  $graphics.Dispose()

  return $bitmap
}

function Save-Png {
  param(
    [System.Drawing.Bitmap]$Bitmap,
    [string]$Path
  )

  $Bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
}

function New-ResizedBitmap {
  param(
    [System.Drawing.Bitmap]$Source,
    [int]$Size
  )

  $bmp = New-Object System.Drawing.Bitmap($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $g.DrawImage($Source, 0, 0, $Size, $Size)
  $g.Dispose()
  return $bmp
}

function New-IcoFromPngs {
  param(
    [string[]]$PngPaths,
    [string]$OutputPath
  )

  $fileStream = [System.IO.File]::Open($OutputPath, [System.IO.FileMode]::Create)
  $writer = New-Object System.IO.BinaryWriter($fileStream)

  try {
    $writer.Write([UInt16]0)
    $writer.Write([UInt16]1)
    $writer.Write([UInt16]$PngPaths.Count)

    $imageBytes = @()
    foreach ($path in $PngPaths) {
      $bytes = [System.IO.File]::ReadAllBytes($path)
      $imageBytes += ,$bytes
    }

    $offset = 6 + (16 * $PngPaths.Count)
    for ($i = 0; $i -lt $PngPaths.Count; $i++) {
      $pngPath = $PngPaths[$i]
      $bytes = $imageBytes[$i]
      $img = [System.Drawing.Image]::FromFile($pngPath)
      $width = if ($img.Width -ge 256) { 0 } else { [byte]$img.Width }
      $height = if ($img.Height -ge 256) { 0 } else { [byte]$img.Height }
      $img.Dispose()

      $writer.Write([byte]$width)
      $writer.Write([byte]$height)
      $writer.Write([byte]0)
      $writer.Write([byte]0)
      $writer.Write([UInt16]1)
      $writer.Write([UInt16]32)
      $writer.Write([UInt32]$bytes.Length)
      $writer.Write([UInt32]$offset)
      $offset += $bytes.Length
    }

    foreach ($bytes in $imageBytes) {
      $writer.Write($bytes)
    }
  }
  finally {
    $writer.Dispose()
    $fileStream.Dispose()
  }
}

$baseBitmap = New-AppIconBitmap -Size 1024

$pngMap = @{
  "icon.png"             = 1024
  "128x128.png"          = 128
  "128x128@2x.png"       = 256
  "32x32.png"            = 32
  "Square30x30Logo.png"  = 30
  "Square44x44Logo.png"  = 44
  "Square71x71Logo.png"  = 71
  "Square89x89Logo.png"  = 89
  "Square107x107Logo.png"= 107
  "Square142x142Logo.png"= 142
  "Square150x150Logo.png"= 150
  "Square284x284Logo.png"= 284
  "Square310x310Logo.png"= 310
  "StoreLogo.png"        = 50
}

foreach ($item in $pngMap.GetEnumerator()) {
  $bmp = if ($item.Value -eq 1024) { $baseBitmap } else { New-ResizedBitmap -Source $baseBitmap -Size $item.Value }
  Save-Png -Bitmap $bmp -Path (Join-Path $iconsDir $item.Key)
  if ($item.Value -ne 1024) {
    $bmp.Dispose()
  }
}

$uiIcon = New-ResizedBitmap -Source $baseBitmap -Size 256
Save-Png -Bitmap $uiIcon -Path (Join-Path $assetsDir "app-icon.png")
$uiIcon.Dispose()

$icoSizes = 16, 24, 32, 48, 64, 128, 256
$tempPngs = @()
foreach ($size in $icoSizes) {
  $bmp = New-ResizedBitmap -Source $baseBitmap -Size $size
  $path = Join-Path $env:TEMP ("windowsfonttuner2-icon-$size.png")
  Save-Png -Bitmap $bmp -Path $path
  $bmp.Dispose()
  $tempPngs += $path
}

New-IcoFromPngs -PngPaths $tempPngs -OutputPath (Join-Path $iconsDir "icon.ico")

foreach ($path in $tempPngs) {
  Remove-Item $path -Force -ErrorAction SilentlyContinue
}

$baseBitmap.Dispose()

Write-Host "Generated icon set in $iconsDir"
