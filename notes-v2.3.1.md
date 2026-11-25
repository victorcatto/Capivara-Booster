## Added

- Dedicated Counter-Strike 2 menu: full competitive profile in one click with per-tweak status (mouse accel off, MMCSS GPU Priority 8, fullscreen exclusive + Game DVR off, high-performance GPU, IFEO priority/affinity, Ultimate power plan, USB selective suspend off, HPET, Nagle/TCPNoDelay, 0.5 ms timer resolution)
- CS2 game configs with automatic backup: competitive cs2_video.txt (fullscreen exclusive, VSync off, 2X MSAA, Reflex on, boost player contrast), autoexec.cfg FPS Boost (sky, water, decals, bullet trails and death animation off) and launch options (-nojoy -freq <hz> +fps_max 0) injected into localconfig.vdf
- CS2 auto-detection via Steam registry and libraryfolders.vdf; shader cache cleanup for AppID 730 plus D3DSCache/NVIDIA/AMD
- Path of Exile 2 community tweaks: DLSS upscaler on the Quality preset, ReBAR/autoHDR forced in UserGpuPreferences

## Fixed

- Sidebar showing raw i18n keys ([Nav_Poe], [Nav_Drivers]) inside the installed app: localization dictionaries are now embedded in the binary (include_str), no longer dependent on files next to the executable
- Cleaner button wording: "Run Cleanup" instead of purge (PT-BR and EN-US)

## PT-BR

- Menu dedicado do Counter-Strike 2: perfil competitivo em 1 clique com status de cada tweak (mouse sem aceleração, MMCSS, tela cheia exclusiva + Game DVR off, GPU alta, IFEO prioridade/afinidade, energia Ultimate, USB suspend off, HPET, Nagle e timer 0,5 ms)
- Configs do CS2 com backup automático: cs2_video.txt competitivo, autoexec.cfg FPS Boost (céu, água, decals, rastros de tiro e animação pós-morte off) e launch options injetadas no localconfig.vdf
- Detecção automática da instalação via Steam; limpeza do shader cache (AppID 730 + D3DSCache/NVIDIA/AMD)
- Path of Exile 2: DLSS no preset Quality e ReBAR/autoHDR forçados
- Fim definitivo das chaves cruas no menu: dicionários de tradução embutidos no executável
- Botão da limpeza com texto profissional "Executar Limpeza"
