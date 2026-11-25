# Gera os frames do vídeo de apresentação do Capivara Booster (1920x1080, paleta oficial).
# Uso: powershell -File make-frames.ps1
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$out = Join-Path $PSScriptRoot 'frames'
New-Item -ItemType Directory -Force $out | Out-Null

$bg       = [System.Drawing.Color]::FromArgb(13, 17, 23)      # Obsidian Slate
$card     = [System.Drawing.Color]::FromArgb(22, 27, 34)      # Carbon Surface
$border   = [System.Drawing.Color]::FromArgb(38, 47, 61)      # Steel Line
$mint     = [System.Drawing.Color]::FromArgb(0, 229, 171)     # Cyber Mint
$volt     = [System.Drawing.Color]::FromArgb(0, 184, 255)     # Volt Cyan
$frost    = [System.Drawing.Color]::FromArgb(240, 246, 252)   # Pure Frost
$silver   = [System.Drawing.Color]::FromArgb(139, 148, 158)   # Muted Silver

$assets = 'C:\Users\User\Documents\PROJETO2\docs\site'
$logo   = Join-Path $assets 'logo.png'
$banner = Join-Path $assets 'banner-poe.png'
$heroIl = Join-Path $assets 'hero-capivara.png'

function New-Frame([string]$nome, [scriptblock]$desenho) {
  $bmp = New-Object System.Drawing.Bitmap(1920, 1080)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = 'AntiAlias'
  $g.TextRenderingHint = 'AntiAliasGridFit'
  $g.Clear($bg)
  & $desenho $g
  $g.Dispose()
  $bmp.Save((Join-Path $out "$nome.png"), [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  Write-Host "frame: $nome"
}

function Draw-Brand([System.Drawing.Graphics]$g, [int]$x, [int]$y, [int]$tam) {
  if (Test-Path $logo) {
    $img = [System.Drawing.Image]::FromFile($logo)
    $g.DrawImage($img, $x, $y, $tam, $tam)
    $img.Dispose()
  }
  $font  = New-Object System.Drawing.Font('Segoe UI', 30, [System.Drawing.FontStyle]::Bold)
  $font2 = New-Object System.Drawing.Font('Segoe UI', 30, [System.Drawing.FontStyle]::Regular)
  $g.DrawString('CAPIVARA ', $font,  (New-Object System.Drawing.SolidBrush($frost)), ($x + $tam + 18), ($y + 4))
  $med = $g.MeasureString('CAPIVARA ', $font)
  $g.DrawString('BOOSTER', $font2, (New-Object System.Drawing.SolidBrush($mint)), ($x + $tam + 18 + $med.Width), ($y + 4))
}

function Draw-Card([System.Drawing.Graphics]$g, [int]$x, [int]$y, [int]$w, [int]$h, [string]$titulo, [string]$desc) {
  $pen = New-Object System.Drawing.Pen($border, 2)
  $br  = New-Object System.Drawing.SolidBrush($card)
  $g.FillRectangle($br, $x, $y, $w, $h)
  $g.DrawRectangle($pen, $x, $y, $w, $h)
  $ft = New-Object System.Drawing.Font('Segoe UI', 22, [System.Drawing.FontStyle]::Bold)
  $fd = New-Object System.Drawing.Font('Segoe UI', 16)
  $g.DrawString($titulo, $ft, (New-Object System.Drawing.SolidBrush($mint)), ($x + 24), ($y + 18))
  $rect = New-Object System.Drawing.RectangleF(($x + 24), ($y + 64), ($w - 48), ($h - 80))
  $g.DrawString($desc, $fd, (New-Object System.Drawing.SolidBrush($silver)), $rect)
}

function Draw-Selo([System.Drawing.Graphics]$g, [string]$texto, [int]$x, [int]$y, [System.Drawing.Color]$cor) {
  $f = New-Object System.Drawing.Font('Segoe UI', 15, [System.Drawing.FontStyle]::Bold)
  $med = $g.MeasureString($texto, $f)
  $w = [int]($med.Width + 36); $h = 38
  $pen = New-Object System.Drawing.Pen($cor, 2)
  $g.DrawRectangle($pen, $x, $y, $w, $h)
  $g.DrawString($texto, $f, (New-Object System.Drawing.SolidBrush($cor)), ($x + 18), ($y + 6))
}

# 1. Abertura: logo + slogan
New-Frame '01-abertura' { param($g)
  if (Test-Path $heroIl) {
    $img = [System.Drawing.Image]::FromFile($heroIl)
    $g.DrawImage($img, 710, 90, 500, 500)
    $img.Dispose()
  }
  $f1 = New-Object System.Drawing.Font('Segoe UI', 56, [System.Drawing.FontStyle]::Bold)
  $med = $g.MeasureString('Frieza máxima,', $f1)
  $x1 = (1920 - $med.Width) / 2
  $g.DrawString('Frieza máxima,', $f1, (New-Object System.Drawing.SolidBrush($frost)), $x1, 620)
  $med2 = $g.MeasureString('latência mínima.', $f1)
  $x2 = (1920 - $med2.Width) / 2
  $g.DrawString('latência mínima.', $f1, (New-Object System.Drawing.SolidBrush($mint)), $x2, 700)
  $f2 = New-Object System.Drawing.Font('Segoe UI', 22)
  $sub = 'Ferramenta gratuita de otimização para jogadores competitivos'
  $meds = $g.MeasureString($sub, $f2)
  $g.DrawString($sub, $f2, (New-Object System.Drawing.SolidBrush($silver)), ((1920 - $meds.Width) / 2), 820)
  Draw-Brand $g 40 40 56
}

# 2. Banner PoE
New-Frame '02-banner-poe' { param($g)
  Draw-Brand $g 40 40 56
  if (Test-Path $banner) {
    $img = [System.Drawing.Image]::FromFile($banner)
    $g.DrawImage($img, 210, 170, 1500, 750)
    $img.Dispose()
  }
  Draw-Selo $g 'NOVO  •  PATH OF EXILE 1 & 2' 40 1000 $volt
}

# 3. Motor de baixa latência
New-Frame '03-latencia' { param($g)
  Draw-Brand $g 40 40 56
  $ft = New-Object System.Drawing.Font('Segoe UI', 44, [System.Drawing.FontStyle]::Bold)
  $g.DrawString('Motor de baixa latência', $ft, (New-Object System.Drawing.SolidBrush($frost)), 60, 150)
  $itens = @(
    @('Dynamic Tick off', 'Kernel sem ticks ociosos: menos jitter de frame'),
    @('MMCSS + Win32PrioritySeparation', 'CPU prioriza o jogo em primeiro plano'),
    @('USB e filas de mouse/teclado', 'Polling estável, buffering reduzido'),
    @('Nagle off + HAGS + PCIe ASPM off', 'Rede sem agrupamento, GPU sempre acordada')
  )
  for ($i = 0; $i -lt $itens.Count; $i++) {
    $y = 280 + $i * 170
    Draw-Card $g 60 $y 900 150 $itens[$i][0] $itens[$i][1]
  }
  Draw-Card $g 1020 280 840 670 '11 tweaks reversíveis', "Cada ajuste tem tooltip explicativo e pode ser revertido ao padrão do Windows com um clique.`n`nPerfil recomendado aplica tudo de uma vez, com ponto de restauração automático."
  Draw-Selo $g 'Requer reinício nos tweaks de BCD' 1020 980 $volt
}

# 4. Jogos competitivos
New-Frame '04-jogos' { param($g)
  Draw-Brand $g 40 40 56
  $ft = New-Object System.Drawing.Font('Segoe UI', 44, [System.Drawing.FontStyle]::Bold)
  $g.DrawString('Perfis competitivos', $ft, (New-Object System.Drawing.SolidBrush($frost)), 60, 150)
  Draw-Card $g 60 280 580 420 'Counter-Strike 2', "autoexec.cfg competitivo`nPrioridade e afinidade por IFEO`nExclusões no Defender`nLimpeza de shader cache`nBoost do processo ao vivo"
  Draw-Card $g 670 280 580 420 'VALORANT', "Jogo em prioridade alta`nRiot Client em prioridade baixa`nExclusões do Vanguard no Defender`nLimpeza de logs e webcache"
  Draw-Card $g 1280 280 580 420 'League of Legends', "Low Spec Mode`nClose Client`nVSync off`nBackup do game.cfg`nExclusões no Defender"
  Draw-Card $g 60 740 1800 280 'Path of Exile 1 & 2', "Iluminação global off: 30 a 75% de FPS a mais em mapas densos.  Canais de som reduzidos, Lockstep/Predictive, GPU de alto desempenho, Nagle off e limpeza segura de shader cache.  Backup automático do .ini antes de qualquer mudança."
}

# 5. Limpeza e debloat
New-Frame '05-limpeza' { param($g)
  Draw-Brand $g 40 40 56
  $ft = New-Object System.Drawing.Font('Segoe UI', 44, [System.Drawing.FontStyle]::Bold)
  $g.DrawString('Limpeza profunda e debloat', $ft, (New-Object System.Drawing.SolidBrush($frost)), 60, 150)
  $itens = @(
    @('15 pacotes UWP removíveis', 'Cortana, Xbox, Weather, Maps e mais, com seleção individual'),
    @('Caches de shader', 'DirectX D3DSCache, NVIDIA DX/GLCache e AMD DxCache'),
    @('Arquivos do Windows', 'Temp, Prefetch, cache do Windows Update, Lixeira e DNS')
  )
  for ($i = 0; $i -lt $itens.Count; $i++) {
    Draw-Card $g 60 (280 + $i * 220) 1150 200 $itens[$i][0] $itens[$i][1]
  }
  Draw-Card $g 1270 280 590 640 'Segurança em primeiro lugar', "Ponto de restauração do Windows antes das operações críticas.`n`nTodo arquivo alterado ganha backup automático, restaurável em um clique.`n`nCada chave do registro é rastreada com o valor original."
}

# 6. Console ao vivo + login
New-Frame '06-console' { param($g)
  Draw-Brand $g 40 40 56
  $ft = New-Object System.Drawing.Font('Segoe UI', 44, [System.Drawing.FontStyle]::Bold)
  $g.DrawString('Console ao vivo, você vê tudo', $ft, (New-Object System.Drawing.SolidBrush($frost)), 60, 150)
  $pen = New-Object System.Drawing.Pen($border, 2)
  $br  = New-Object System.Drawing.SolidBrush($card)
  $g.FillRectangle($br, 60, 280, 1800, 480)
  $g.DrawRectangle($pen, 60, 280, 1800, 480)
  $fc = New-Object System.Drawing.Font('Consolas', 20)
  $linhas = @(
    @('14:32:01 [Cleaner] nvidia_shader: 1.248 arquivos, 0 erros', $silver),
    @('14:32:07 [Latency] dynamic_tick aplicado', $silver),
    @('14:32:15 [PoE] Perfil poe2 aplicado: production Config', $silver),
    @('14:32:15 [OK] Perfil PoE aplicado com sucesso', $mint)
  )
  for ($i = 0; $i -lt $linhas.Count; $i++) {
    $g.DrawString($linhas[$i][0], $fc, (New-Object System.Drawing.SolidBrush($linhas[$i][1])), 100, (330 + $i * 70))
  }
  Draw-Card $g 60 800 1800 200 'Login obrigatório, dados protegidos', "Cofre de contas com DPAPI do Windows e senhas em PBKDF2-HMAC-SHA256. Cadastro restrito a Gmail, Outlook, Yahoo e Yahoo Brasil."
}

# 7. Fechamento: download
New-Frame '07-download' { param($g)
  if (Test-Path $heroIl) {
    $img = [System.Drawing.Image]::FromFile($heroIl)
    $g.DrawImage($img, 760, 80, 400, 400)
    $img.Dispose()
  }
  $f1 = New-Object System.Drawing.Font('Segoe UI', 48, [System.Drawing.FontStyle]::Bold)
  $texto = 'Baixe o Capivara Booster'
  $med = $g.MeasureString($texto, $f1)
  $g.DrawString($texto, $f1, (New-Object System.Drawing.SolidBrush($frost)), ((1920 - $med.Width) / 2), 540)
  $f2 = New-Object System.Drawing.Font('Segoe UI', 30, [System.Drawing.FontStyle]::Bold)
  $cta = 'github.com/victorcatto/Capivara-Booster'
  $med2 = $g.MeasureString($cta, $f2)
  $x = (1920 - $med2.Width) / 2
  $pen = New-Object System.Drawing.Pen($mint, 3)
  $g.DrawRectangle($pen, ($x - 30), 640, ($med2.Width + 60), 90)
  $g.DrawString($cta, $f2, (New-Object System.Drawing.SolidBrush($mint)), $x, 660)
  $f3 = New-Object System.Drawing.Font('Segoe UI', 20)
  $nota = 'Grátis • Open source • Windows 10/11 • 100% reversível'
  $med3 = $g.MeasureString($nota, $f3)
  $g.DrawString($nota, $f3, (New-Object System.Drawing.SolidBrush($silver)), ((1920 - $med3.Width) / 2), 790)
  Draw-Brand $g 40 40 56
}

Write-Host 'frames prontos em docs/video/frames'
