param(
    [string]$Source = "Assets\logo.png",
    [string]$Output = "bin\Release\net8.0-windows\app.ico"
)

Write-Host "Generating multi-resolution .ico from: $Source" -ForegroundColor Cyan

# Check if source exists
if (-not (Test-Path $Source)) {
    Write-Error "Source file not found: $Source"
    exit 1
}

# Add scripts folder to path for System.Drawing
$dotnetSdk = (Get-ChildItem -Path "C:\Program Files\dotnet" -ErrorAction SilentlyContinue | Sort-Object Version -Desc | Select-Object -First 1).FullName
if (-not $dotnetSdk) {
    Write-Error "dotnet SDK not found"
    exit 1
}

# Use .NET to generate the icon
$script = @"
using System.Drawing;
using System.Drawing.Imaging;

namespace IconGen {
    public static class Generator {
        public static void MakeIcon(string src, string out) {
            using var img = Image.FromFile(src);
            int[] resls = { 16, 32, 48, 128, 256 };
            using var icon = new Icon(1);
            for (int i = 0; i < resls.Length; i++) {
                int w = resls[i], h = resls[i];
                using var bmp = new Bitmap(w, h);
                using var g = Graphics.FromImage(bmp);
                g.CompositingQuality = System.Drawing.Drawing2D.CompositingQuality.HighQuality;
                g.SmoothingMode = System.Drawing.Drawing2D.SmoothingMode.HighQuality;
                g.InterpolationMode = System.Drawing.Drawing2D.InterpolationMode.HighQualityBicubic;
                g.Clear(Color.Transparent);
                int x = (w - img.Width) / 2;
                int y = (h - img.Height) / 2;
                g.DrawImage(img, new Rectangle(x, y, img.Width, img.Height));
                icon.Frames[i] = IconFrame.FromBitmap(bmp);
            }
            icon.Save(out);
        }
    }
}
using System;
using System.Reflection;
using System.IO;

string execPath = Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location);
string sourcePath = Path.Combine(execPath, "$Source");
string outputPath = Path.Combine(execPath, "$Output");

if (!File.Exists(sourcePath)) {
    Console.Error.WriteLine("Source not found: " + sourcePath);
    Environment.ExitCode = 1;
    return;
}

IconGen.Generator.MakeIcon(sourcePath, outputPath);
Console.WriteLine("Icon generated: " + outputPath);
"@

$dotnetExe = "$env:HOME\dotnet"
if (-not (Test-Path $dotnetExe)) {
    $dotnetExe = (Get-ChildItem -Path "C:\Program Files\dotnet" -ErrorAction SilentlyContinue | Sort-Object Version -Desc | Select-Object -First 1).FullName
}

if (-not $dotnetExe) {
    Write-Error "Cannot find dotnet executable"
    exit 1
}

Write-Host "Running .NET script to generate icon..." -ForegroundColor Yellow

$tempCsx = Path.Combine($env:TEMP, "GenerateIcon.csx")
File.WriteAllLines($tempCsx, $script)

$args = ""
& $dotnetExe script $tempCsx $args

Write-Host "Icon generation complete!" -ForegroundColor Green