## Added

- Driver source pool with link verification: each vendor now has a pool of official sources in preference order (Intel DSA detect + Download Center 28425, NVIDIA app direct + auto-detect, AMD drivers pages, Realtek downloads). Every link is verified with an HTTP HEAD request (10s timeout) before reaching the user; dead links are skipped automatically and the first working source wins
- "Scan cache" button in the Cache Cleaner: rescans the disk live and refreshes table sizes without losing your selections, with a Scanning state on the button

## Changed

- Cleaner measurement engine parallelized (rayon): all 9 disk targets are now measured simultaneously instead of one by one, several times faster rescans
- New rescan_cleaner command runs off the main thread: the UI no longer freezes while the disk is measured

## Fixed

- Driver Booster never opens an unverified page again: if every pool source is down at that moment, the app reports exactly what was tried instead of opening a 404

## PT-BR

- Pool de fontes de driver com verificação de link: cada fabricante tem fontes oficiais em ordem de preferência (Intel DSA + Download Center 28425, NVIDIA app direto + auto-detect, páginas da AMD, downloads da Realtek). Todo link é testado com HTTP HEAD (10s) antes de chegar ao usuário; links mortos são pulados e a primeira fonte funcional ganha
- Botão "Procurar cache" na limpeza: re-escaneia o disco ao vivo e atualiza os tamanhos da tabela sem perder suas marcações
- Motor de medição em paralelo (rayon): os 9 alvos de disco são medidos ao mesmo tempo, re-escanso várias vezes mais rápido
- Driver Booster nunca mais abre página não verificada: se todas as fontes estiverem fora no momento, o app informa o que foi tentado em vez de abrir um 404
