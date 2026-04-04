[CmdletBinding()]
param(
    [string]$IconPath,
    [string]$PreviewPath
)

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

Add-Type -AssemblyName System.Drawing

$root = Split-Path -Parent $PSScriptRoot
$IconPath = if ($IconPath) { $IconPath } else { Join-Path $root 'Assets\AppIcon.ico' }
$PreviewPath = if ($PreviewPath) { $PreviewPath } else { Join-Path $root 'artifacts\AppIcon-preview.png' }
$iconDirectory = Split-Path -Parent $IconPath
$previewDirectory = Split-Path -Parent $PreviewPath

New-Item -ItemType Directory -Path $iconDirectory -Force | Out-Null
New-Item -ItemType Directory -Path $previewDirectory -Force | Out-Null

function New-RoundedPath {
    param(
        [System.Drawing.RectangleF]$Bounds,
        [float]$Radius
    )

    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $diameter = $Radius * 2

    $path.AddArc($Bounds.X, $Bounds.Y, $diameter, $diameter, 180, 90)
    $path.AddArc($Bounds.Right - $diameter, $Bounds.Y, $diameter, $diameter, 270, 90)
    $path.AddArc($Bounds.Right - $diameter, $Bounds.Bottom - $diameter, $diameter, $diameter, 0, 90)
    $path.AddArc($Bounds.X, $Bounds.Bottom - $diameter, $diameter, $diameter, 90, 90)
    $path.CloseFigure()
    return $path
}

function New-IconBitmap {
    param([int]$Size)

    $bitmap = New-Object System.Drawing.Bitmap $Size, $Size
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $graphics.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
    $graphics.Clear([System.Drawing.Color]::Transparent)

    $pad = [Math]::Max(1, [int]($Size * 0.08))
    $radius = [Math]::Max(3, $Size * 0.24)
    $rect = [System.Drawing.RectangleF]::new([single]$pad, [single]$pad, [single]($Size - ($pad * 2)), [single]($Size - ($pad * 2)))

    $bgPath = New-RoundedPath -Bounds $rect -Radius $radius
    $startColor = [System.Drawing.Color]::FromArgb(255, 43, 108, 246)
    $endColor = [System.Drawing.Color]::FromArgb(255, 59, 210, 255)
    $brush = [System.Drawing.Drawing2D.LinearGradientBrush]::new($rect, $startColor, $endColor, 45.0)
    $graphics.FillPath($brush, $bgPath)

    $glowColor = [System.Drawing.Color]::FromArgb(80, 255, 255, 255)
    $glowBrush = New-Object System.Drawing.SolidBrush($glowColor)
    $graphics.FillEllipse($glowBrush, $Size * 0.54, $Size * 0.02, $Size * 0.34, $Size * 0.24)

    $lineBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 255, 255, 255))
    $fontFamily = New-Object System.Drawing.FontFamily('Segoe UI')
    $fontSize = if ($Size -lt 24) { $Size * 0.54 } elseif ($Size -lt 48) { $Size * 0.5 } else { $Size * 0.46 }
    $font = New-Object System.Drawing.Font($fontFamily, $fontSize, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)

    $format = New-Object System.Drawing.StringFormat
    $format.Alignment = [System.Drawing.StringAlignment]::Center
    $format.LineAlignment = [System.Drawing.StringAlignment]::Center

    $text = if ($Size -lt 32) { 'A' } else { 'Aa' }
    $textRect = [System.Drawing.RectangleF]::new([single]($Size * 0.08), [single]($Size * 0.02), [single]($Size * 0.84), [single]($Size * 0.82))
    $graphics.DrawString($text, $font, $lineBrush, $textRect, $format)

    $barHeight = [Math]::Max(2, [int]($Size * 0.075))
    $barWidth = [Math]::Max(8, [int]($Size * 0.42))
    $barRect = [System.Drawing.RectangleF]::new([single](($Size - $barWidth) / 2.0), [single]($Size * 0.74), [single]$barWidth, [single]$barHeight)
    $barPath = New-RoundedPath -Bounds $barRect -Radius ($barHeight / 2.0)
    $barBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(235, 233, 247, 255))
    $graphics.FillPath($barBrush, $barPath)

    $barBrush.Dispose()
    $lineBrush.Dispose()
    $format.Dispose()
    $font.Dispose()
    $fontFamily.Dispose()
    $glowBrush.Dispose()
    $brush.Dispose()
    $bgPath.Dispose()
    $barPath.Dispose()
    $graphics.Dispose()

    return $bitmap
}

$sizes = @(16, 24, 32, 48, 64, 128, 256)
$pngEntries = @()

foreach ($size in $sizes) {
    $bitmap = New-IconBitmap -Size $size
    $memory = New-Object System.IO.MemoryStream
    $bitmap.Save($memory, [System.Drawing.Imaging.ImageFormat]::Png)
    $pngEntries += [PSCustomObject]@{
        Size = $size
        Bytes = $memory.ToArray()
    }

    if ($size -eq 256) {
        $bitmap.Save($PreviewPath, [System.Drawing.Imaging.ImageFormat]::Png)
    }

    $memory.Dispose()
    $bitmap.Dispose()
}

$iconStream = New-Object System.IO.FileStream($IconPath, [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write)
$writer = New-Object System.IO.BinaryWriter($iconStream)

$writer.Write([UInt16]0)
$writer.Write([UInt16]1)
$writer.Write([UInt16]$pngEntries.Count)

$offset = 6 + (16 * $pngEntries.Count)
foreach ($entry in $pngEntries) {
    $sizeByte = if ($entry.Size -ge 256) { 0 } else { [byte]$entry.Size }
    $writer.Write([byte]$sizeByte)
    $writer.Write([byte]$sizeByte)
    $writer.Write([byte]0)
    $writer.Write([byte]0)
    $writer.Write([UInt16]1)
    $writer.Write([UInt16]32)
    $writer.Write([UInt32]$entry.Bytes.Length)
    $writer.Write([UInt32]$offset)
    $offset += $entry.Bytes.Length
}

foreach ($entry in $pngEntries) {
    $writer.Write($entry.Bytes)
}

$writer.Flush()
$writer.Dispose()
$iconStream.Dispose()

[PSCustomObject]@{
    IconPath = $IconPath
    PreviewPath = $PreviewPath
    Sizes = ($sizes -join ', ')
} | Format-List





