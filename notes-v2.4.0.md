## Added

- Full System Report: new dashboard button collects live telemetry (CPU model, load %, temperature via ACPI sensor, RAM usage, GPU driver and date, per-disk free space, active power plan, timer state, motherboard with exact model, BIOS and Windows build) and generates a branded PDF on the user's real Desktop (registry path, OneDrive-safe)
- Report PDF generated in pure Rust (PDF 1.4 written by hand, no external crates): Capivara Booster branded header, organized sections and repository footer

## Changed

- CI/CD build time: tauri-cli is now cached (was recompiled every run, 3-5 minutes), npm install with node_modules cache, rust-cache covering all crates and the cargo bin directory, release profile with codegen-units 16 and thin LTO (much faster linking)

## Fixed

- Release workflow failing on the manifest commit step (local Cargo.toml change blocked the checkout of main)
- Release workflow failing on npm ci (repository has no package-lock.json)

## PT-BR

- Relatório Completo: novo botão no painel coleta telemetria ao vivo (CPU com nome, carga e temperatura via sensor ACPI, RAM, driver e data da GPU, espaço livre por disco, plano de energia, estado dos timers, placa-mãe exata, BIOS e build do Windows) e gera um PDF com a marca Capivara Booster na Área de Trabalho real (caminho via registro, compatível com OneDrive)
- PDF gerado em Rust puro (PDF 1.4 escrito à mão, sem crates externas): cabeçalho com faixa Cyber Mint, seções organizadas e rodapé com o repositório
- Tempo de build no CI: tauri-cli agora é cacheado (antes recompilava a cada run, 3 a 5 minutos), npm com cache de node_modules, rust-cache cobrindo todas as crates e o cargo bin, perfil release com codegen-units 16 e LTO thin
- Fim do erro do workflow de release no commit do manifest e no npm ci
