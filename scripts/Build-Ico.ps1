param(
    [string]$Source = "C:\Users\User\Documents\PROJETO2\src\CapivaraBooster\Assets\logo.png",
    [string]$Output = "C:\Users\User\Documents\PROJETO2\src\CapivaraBooster\Assets\app.ico"
)

Add-Type -AssemblyName System.Drawing

$img = [System.Drawing.Image]::FromFile($Source)

try {
    $sizes = @(16, 24, 32, 48, 64, 128, 256)

    # PNG-compressed 256px frame + BMP frames for smaller sizes, packed into a real ICO container
    $ms = New-Object System.IO.MemoryStream

    # ICO header: reserved(2) type(2) count(2)
    $writer = New-Object System.IO.BinaryWriter($ms)
    $writer.Write([UInt16]0)      # reserved
    $writer.Write([UInt16]1)      # type = icon
    $writer.Write([UInt16]$sizes.Count)

    $pngFrames = @{}
    foreach ($s in $sizes) {
        $bmp = New-Object System.Drawing.Bitmap($s, $s)
        $g = [System.Drawing.Graphics]::FromImage($bmp)
        $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
        $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
        $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $g.Clear([System.Drawing.Color]::Transparent)

        # Cover-fit: scale to fill, center crop (square source assumed)
        $scale = [Math]::Max($s / $img.Width, $s / $img.Height)
        $w = [int]($img.Width * $scale)
        $h = [int]($img.Height * $scale)
        $x = [int](($s - $w) / 2)
        $y = [int](($s - $h) / 2)
        $g.DrawImage($img, $x, $y, $w, $h)
        $g.Dispose()

        $frameMs = New-Object System.IO.MemoryStream
        # PNG-embedded frames (Vista+): aceito por tauri-build/RC sem "old DIB"
        $bmp.Save($frameMs, [System.Drawing.Imaging.ImageFormat]::Png)
        $pngFrames[$s] = $frameMs.ToArray()
        $bmp.Dispose()
    }

    # Directory entries (16 bytes each)
    $offset = 6 + (16 * $sizes.Count)
    foreach ($s in $sizes) {
        $data = $pngFrames[$s]
        $dim = if ($s -ge 256) { 0 } else { $s }
        $writer.Write([Byte]$dim)          # width (0 = 256)
        $writer.Write([Byte]$dim)          # height
        $writer.Write([Byte]0)             # color palette
        $writer.Write([Byte]0)             # reserved
        $writer.Write([UInt16]1)           # color planes
        $writer.Write([UInt16]32)          # bits per pixel
        $writer.Write([UInt32]$data.Length)
        $writer.Write([UInt32]$offset)
        $offset += $data.Length
    }

    # Frame data
    foreach ($s in $sizes) {
        $writer.Write($pngFrames[$s])
    }

    $writer.Flush()
    [System.IO.File]::WriteAllBytes($Output, $ms.ToArray())
    $writer.Dispose()
    $ms.Dispose()
    Write-Host "ICO gerado com $($sizes.Count) resolucoes: $Output"
}
finally {
    $img.Dispose()
}
