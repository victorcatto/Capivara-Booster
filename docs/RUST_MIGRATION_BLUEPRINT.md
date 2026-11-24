# Capivara Booster — Blueprint de Migração para Rust (Tauri v2)

> **Versão:** 1.0 · **Data:** 2026-11-24 · **Status:** Proposta aprovada (stack: Tauri v2)
> **Base atual:** WPF .NET 8 · v1.1.4 · `victorcatto/Capivara-Booster`

---

## 1. Sumário Executivo

Reescrever o Capivara Booster em **Rust + Tauri v2** (backend Rust, UI web dark-mode embutida) mantendo **paridade total de funcionalidades** na v2.0.0. A migração é incremental, em branch própria (`rust`), sem quebrar a base de usuários v1.x nem o pipeline de auto-update existente.

**Ganho imediato para o usuário final:**

| Métrica | WPF .NET 8 (hoje) | Tauri v2 (alvo) |
|---|---|---|
| Dependência de runtime | .NET 8 Desktop Runtime obrigatório | **Nenhuma** (binário autossuficiente) |
| Instalador | ~3,9 MB (framework-dependent) | ~5–8 MB (tudo incluído) |
| RAM em repouso | ~80–150 MB (WPF + CLR) | **~30–60 MB** (WebView2 compartilhado) |
| Binário principal | CapivaraBooster.dll + runtime | **um único .exe** |
| Acesso a Win32/Registry | P/Invoke + shells para PowerShell/CMD | **nativo** (`windows-rs`, `winreg`) |
| Tempo de inicialização fria | ~1–2 s (JIT) | **< 300 ms** (nativo) |

> Nota: o build atual usa `--self-contained false`, ou seja, o instalador v1.x exige que o usuário tenha o .NET Desktop Runtime 8 instalado. A versão Rust elimina esse requisito — é o argumento nº 1 de marketing para a v2.0.

---

## 2. Stack Recomendada (e por quê)

### 2.1 Decisão: **Tauri v2** (Rust backend + frontend web)

- **UI moderna sem reescrever o tema**: o visual cyberpunk (`#0F1015` / `#181A22` / `#00E5FF` / `#9D00FF`) vira CSS puro — mais fácil de iterar que XAML.
- **i18n preservada 100%**: os dicionários `Localization/pt-BR.json` e `en-US.json` são **reutilizados como estão** — só muda quem os carrega.
- **Auto-update oficial**: `tauri-plugin-updater` com assinatura integrada; adaptamos ao nosso `version.json` na fase 3.
- **Instalador integrado**: bundler gera NSIS/MSI + assinatura; o Inno Setup atual pode ser aposentado ou mantido em paralelo.
- **WebView2**: já vem com o Windows 10/11 moderno; o Tauri usa o do sistema, sem empacotar Chromium (diferença crucial vs. Electron).

### 2.2 Crates Rust por domínio

| Domínio atual (C#) | Crate Rust | Papel |
|---|---|---|
| `ProcessExecutor` (cmd/PowerShell) | `tokio` + `tokio::process::Command` | execução assíncrona com stream de stdout/stderr em tempo real |
| `RegistryService` | `winreg` | leitura/escrita de registro com tipos seguros |
| `AuthService` (DPAPI + PBKDF2) | `windows` (Win32.Security.Cryptography) + `ring` / `pbkdf2` | DPAPI via `CryptProtectData`, hash PBKDF2-SHA256 100k iter |
| `UpdateService` | `reqwest` + `sha2` + `tauri-plugin-updater` | download com progresso, verificação SHA-256 |
| `CleanerService` | `walkdir` + `rayon` | varredura paralela de pastas (temp, prefetch, caches GPU) |
| `SystemBoostService` (HPET, plano de energia) | `windows` (bcdedit via cmd) + `powercfg` | mesmos comandos, orquestração nativa |
| `GameOptimizerService` (CS2/Valorant/LoL) | `winreg` + `serde_json` | IFEO, `AppCompatFlags`, `autoexec.cfg`, `game.cfg` |
| `DebloatService` (UWP) | PowerShell `Remove-AppxPackage` via `tokio::process` | pipeline em lote idêntico ao atual |
| `LocalizationService` | `rust-i18n` ou `fluent` | carrega os mesmos JSONs pt-BR/en-US |
| `RestoreService` (restore point + rollback) | PowerShell `Checkpoint-Computer` + `winreg` backup JSON | mesmo fluxo, armazenado em `%AppData%\CapivaraBooster` |

### 2.3 Frontend

- **Framework**: escolha livre — recomendo **vanilla TS + Vite** ou **Svelte 5** (leve, reativo, sem overhead de runtime). React também funciona.
- **Tema**: portar `Styles/DarkTheme.xaml` → `styles/theme.css` com variáveis CSS (`--bg`, `--card`, `--accent-cyan`, `--accent-purple`).
- **Console em tempo real**: eventos Tauri (`emit` do backend → `listen` no front) alimentam um `<virtualized-list>` — mesmo comportamento do ListBox virtualizado atual.

---

## 3. Estrutura de Projeto Proposta (workspace)

```
capivara-booster/
├── src-tauri/                    # backend Rust
│   ├── Cargo.toml
│   ├── tauri.conf.json           # config v2 (janela, bundle, updater)
│   ├── capabilities/             # permissões IPC granulares (v2)
│   ├── icons/                    # app.ico multi-resolução (reaproveitado)
│   └── src/
│       ├── main.rs               # entrypoint Tauri
│       ├── commands/             # comandos IPC expostos ao front
│       │   ├── mod.rs
│       │   ├── dashboard.rs      # status HPET, jogos detectados, quick boost
│       │   ├── debloat.rs        # remoção UWP, telemetria
│       │   ├── cleaner.rs        # scan/clean de caches
│       │   ├── boost.rs          # HPET, power plan, network throttling
│       │   ├── games.rs          # CS2 / Valorant / LoL
│       │   ├── auth.rs           # login/registro DPAPI
│       │   ├── restore.rs        # restore point + rollback
│       │   └── updater.rs        # checagem de versão / download
│       ├── services/             # lógica pura (sem Tauri), espelhando os Services C#
│       │   ├── process_executor.rs
│       │   ├── registry.rs
│       │   ├── cleaner.rs
│       │   ├── boost.rs
│       │   ├── games/{cs2,valo,lol}.rs
│       │   ├── auth.rs
│       │   └── update.rs
│       └── localization/         # pt-BR.json, en-US.json (movidos ou embarcados)
├── src/                          # frontend web
│   ├── index.html
│   ├── main.ts
│   ├── styles/{theme.css,controls.css}
│   ├── views/{Dashboard,Cleaner,Debloat,SystemBoost,GameOptimizer,BackupRestore}.ts
│   └── i18n.ts                   # carrega JSONs, troca em runtime
├── scripts/
│   ├── Build-Ico.ps1             # já existente — reaproveitado
│   └── publish-update.ps1        # adaptado para cargo-tauri
└── .github/workflows/release.yml # adaptado: rust toolchain + tauri-action
```

**Durante a migração**, o layout fica híbrido: `src-tauri/` convive com `src/CapivaraBooster/` (WPF) até a paridade — depois o WPF é removido.

---

## 4. Roadmap Incremental (5 fases)

### Fase 0 — Fundação (1–2 dias)
- [ ] Criar branch `rust`.
- [ ] Scaffold `cargo tauri init` + Vite; janela dark-mode 1280×870, ícone atual.
- [ ] CI: workflow novo `release-rust.yml` (build em `windows-latest`, `dtolnay/rust-toolchain@stable` + `tauri-apps/tauri-action`).
- [ ] **Marco**: janela abre, tema carregado, release de teste publica instalador assinado (dev).

### Fase 1 — Núcleo de sistema (3–5 dias)
- [ ] Portar `ProcessExecutor` (tokio) + `RegistryService` (winreg) com testes unitários.
- [ ] Portar `SystemBoostService`: HPET (`bcdedit`), plano Ultimate Performance, Game DVR, Network Throttling Index.
- [ ] Portar `CleanerService` (walkdir + rayon) com os 8 alvos atuais.
- [ ] Comandos IPC + UI do Dashboard e System Boost.
- [ ] **Marco**: Quick Boost funcional de ponta a ponta (restore point → tweaks → limpeza → refresh).

### Fase 2 — Debloat + Jogos (3–5 dias)
- [ ] Portar `DebloatService` (pipeline UWP em lote) + `RestoreService` (rollback de registro em JSON imutável).
- [ ] Portar `GameOptimizerService`: CS2 (IFEO, autoexec.cfg), Valorant (prioridades, exclusões Defender), LoL (`game.cfg`, LowSpecMode).
- [ ] Views Debloat, Cleaner, Game Optimizer, Backup & Restore.
- [ ] **Marco**: paridade funcional dos 6 módulos (checklist item a item contra o app v1.1.4).

### Fase 3 — Auth + Updater (2–3 dias)
- [ ] Portar `AuthService`: vault DPAPI (`CryptProtectData`), PBKDF2-100k, regra de admin único `victorcatto93@gmail.com`, domínios permitidos.
- [ ] Integrar `tauri-plugin-updater` apontando para o `version.json` do branch `main` (mesmo formato; campo extra `signature` se usar assinatura Tauri).
- [ ] **Marco**: login/registro + auto-update funcional no formato atual.

### Fase 4 — Release v2.0.0 (2–3 dias)
- [ ] Auditoria completa (mesma lista de itens do release v1.1.4).
- [ ] i18n completa PT-BR/EN-US nos novos views.
- [ ] Substituir workflow antigo; atualizar READMEs (badges, links, requisitos de sistema).
- [ ] Publicar **v2.0.0** com changelog de migração; o updater da v1.1.4 oferece a v2.0.0 naturalmente (version bump no `version.json`).
- [ ] **Marco**: usuários v1.x atualizam para a v2.0 Rust pelo próprio app.

### Compatibilidade durante a transição
- O `version.json` continua sendo a fonte da verdade; o app Rust lê o mesmo contrato (`version`, `installerUrl`, `sha256`, `changelog`).
- Enquanto houver usuários v1.x, manter o campo `installerUrl` apontando para o instalador NSIS da v2 (o app WPF baixa e executa qualquer .exe — funciona).
- As chaves de registro e arquivos em `%AppData%\CapivaraBooster` usam os mesmos nomes → configurações de idioma e sessão sobrevivem à migração.

---

## 5. Riscos e Mitigações

| Risco | Impacto | Mitigação |
|---|---|---|
| WebView2 ausente (Win10 LTSC antigo) | app não abre | detectar e oferecer download do Evergreen Runtime no instalador NSIS |
| `bcdedit`/`powercfg` exigem elevação | falha silenciosa | manifest do NSIS pede admin; janela Tauri roda elevada como hoje (app.manifest) |
| PowerShell execution policy | debloat falha | manter `-NoProfile -NonInteractive -ExecutionPolicy Bypass` (já feito no ProcessExecutor) |
| Assinatura de código ausente | SmartScreen bloqueia | igual hoje — instruções "Mais informações → Executar assim mesmo" no README |
| Paridade de comportamento do console | logs diferentes | eventos Tauri emitem o mesmo formato `FormattedMessage` + `Level` |
| Regressão de regress de registro | sistema do usuário | portar o `RestoreService` ANTES de qualquer tweak (mesma ordem da Fase 1) |

---

## 6. Métricas de Sucesso da v2.0.0

1. **Paridade**: todos os 6 módulos + auth + updater funcionando identicamente ao v1.1.4 (checklist de auditoria 100%).
2. **Footprint**: instalador ≤ 8 MB; RAM em repouso ≤ 60 MB; startup < 300 ms.
3. **Zero dependência**: roda em Windows 10 1809+ limpo (só WebView2, pré-instalado em 95%+ das máquinas).
4. **Migração transparente**: usuário v1.1.4 recebe v2.0.0 pelo auto-update sem perda de configurações.
5. **CI verde**: release assinada, `sha256` preenchido no manifest, download direto funcional.

---

## 7. Próximo Passo Imediato

Autorizado o início, a primeira ação concreta é a **Fase 0**: criar o branch `rust`, scaffold do Tauri v2 com o tema atual portado, e um release de teste no GitHub para validar o pipeline completo antes de portar qualquer lógica.
