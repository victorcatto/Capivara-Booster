<div align="center">

# CAPIVARA BOOSTER

### Otimização do Windows 10/11 para jogos competitivos

[![Última versão](https://img.shields.io/github/v/release/victorcatto/Capivara-Booster?style=for-the-badge&color=00E5AB)](https://github.com/victorcatto/Capivara-Booster/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/victorcatto/Capivara-Booster/release.yml?branch=main&style=for-the-badge&label=build%20de%20release)](https://github.com/victorcatto/Capivara-Booster/actions)
[![Windows 10/11](https://img.shields.io/badge/Windows-10%20%7C%2011%20x64-00B8FF?style=for-the-badge&logo=windows)](https://github.com/victorcatto/Capivara-Booster)
[![Licença](https://img.shields.io/badge/licença-MIT-8B949E?style=for-the-badge)](LICENSE)

[English version](README.md)

**[Landing page oficial](https://victorcatto.github.io/Capivara-Booster/)**: versão atual, download direto e as últimas novidades.

</div>

## O que é o Capivara Booster?

O Capivara Booster é uma ferramenta open source para Windows, criada para jogadores competitivos e usuários avançados. Ela reúne limpeza do sistema, controles de privacidade, ajustes de latência, perfis específicos para jogos, backups de configuração e reversão em um único aplicativo com controle de administrador.

O aplicativo é construído nativamente em **Rust** com **Tauri v2**: um único executável leve, sem dependência do runtime .NET, acesso direto ao Registro do Windows e interface web moderna em modo escuro.

## Download

**[Baixar o instalador oficial mais recente](https://github.com/victorcatto/Capivara-Booster/releases/latest/download/CapivaraBooster-Setup.exe)**

Consulte o [histórico de releases](https://github.com/victorcatto/Capivara-Booster/releases) para hashes, changelogs e instaladores versionados.

### Requisitos

- Windows 10 ou Windows 11, 64 bits
- Permissão de administrador para alterações de sistema e registro
- WebView2 Runtime (pré-instalado no Windows 10/11 atualizado)

O Windows SmartScreen pode exibir um aviso porque o instalador open source não possui assinatura de código. Confira o asset da release e o hash SHA-256 antes de executá-lo.

## Recursos

### Sistema e privacidade

- Ponto de restauração antes de operações destrutivas
- Rastreamento e reversão de alterações do registro
- Debloat de pacotes UWP selecionados
- Controle de serviços e tarefas agendadas de telemetria
- Inspeção de itens de inicialização

### Limpeza

- Arquivos temporários do usuário e do sistema
- Prefetch e cache de downloads do Windows Update
- Lixeira e cache de resolução DNS
- Caches de shaders DirectX, NVIDIA e AMD
- Varredura paralela com seleção por item

### Latência e desempenho

- Controle do HPET e dos temporizadores sintéticos
- Plano de energia Desempenho Máximo
- Ajustes do Game DVR e otimizações de tela cheia
- Network Throttling Index e SystemResponsiveness
- Perfil de agendamento multimídia para jogos

### Perfis de jogos

- **Counter-Strike 2:** prioridade/afinidade por IFEO, AppCompat, cache de shaders e `autoexec.cfg` competitivo
- **VALORANT:** prioridade do jogo e Riot Client, AppCompat, logs e exclusões do Defender
- **League of Legends:** modo de baixo consumo, fechamento do cliente, logs, backups e exclusões do Defender
- **Path of Exile 1 & 2:** perfil competitivo da engine GGG
  - Iluminação global desativada (ganho de 30 a 75% de FPS em mapas densos), sombras no mínimo, screen shake off
  - Culling dinâmico e resolução dinâmica para estabilidade em endgame
  - Canais de som reduzidos (o áudio do PoE é um gargalo real de CPU) e opção de mutar efeitos mantendo o filtro de loot
  - Alternador Lockstep/Predictive com aviso de uso (Lockstep para ping baixo e Hardcore)
  - Preferência de GPU de alto desempenho, Nagle off e limpeza segura de shader cache
  - Backup automático do `.ini` original antes de qualquer alteração

### Conta e atualizações

- Troca de idioma PT-BR e EN-US em tempo real
- Cofre de contas local protegido pelo Windows DPAPI
- Derivação de senha PBKDF2-HMAC-SHA256 com salt aleatório
- Cadastro restrito a Gmail, Outlook, Yahoo e Yahoo Brasil
- Identidade única de administrador protegida internamente e não exposta na interface
- Manifesto de atualização no GitHub com verificação SHA-256
- Console de execução oculto por padrão e disponível por botão

## Segurança & Transparência

O Capivara Booster altera configurações do Windows somente depois de uma ação explícita do usuário.

- **Ponto de restauração do Windows** criado antes das operações destrutivas compatíveis
- **Backup automático de arquivos** antes de modificar qualquer configuração de jogo (`game.cfg` do LoL, `autoexec.cfg` do CS2, `.ini` do Path of Exile), restaurável em um clique
- **Rastreamento de registro**: cada chave alterada pelo aplicativo é registrada com o valor original, permitindo rollback total em Backup & Restore
- **Open source**: todo o código está neste repositório, auditável antes de instalar
- **Atualizações verificadas**: o auto-updater valida o SHA-256 do instalador antes de executá-lo

Revise cada otimização antes de aplicá-la. Hardware, edição do Windows, drivers, política do antivírus e atualizações dos jogos podem alterar o resultado de um ajuste.

## Compilar a partir do código-fonte

```powershell
git clone https://github.com/victorcatto/Capivara-Booster.git
Set-Location Capivara-Booster

# Frontend
Set-Location ui
npm install
npm run build

# Backend Rust + instalador NSIS
Set-Location ..\src-tauri
cargo check
cargo test
cargo tauri build --bundles nsis
```

A compilação exige Rust stable, ferramentas de compilação Windows MSVC, Node.js e Tauri CLI. O CI roda em `windows-latest` e valida frontend, compilação Rust, testes e bundle NSIS.

## Arquitetura

- Backend: Rust com Tokio, `winreg`, `windows-rs`, DPAPI (cofre/sessão), verificação SHA-256 de atualizações e `serde_json`
- Frontend: TypeScript com Vite, tema cyberpunk escuro e troca PT-BR/EN-US em tempo de execução
- Empacotamento: instalador NSIS do Tauri v2, modo de instalação por máquina e manifesto requireAdministrator
- Pipeline de release: GitHub Actions com runners Windows

## Status do projeto

- Árvore única de desenvolvimento: todo o trabalho segue em `main`
- Tags de release `v2.*` e posteriores compilam a edição Rust via GitHub Actions
- Os instaladores v1.x (WPF) permanecem disponíveis na página de Releases para instalações existentes, e o atualizador Rust mantém o mesmo contrato `version.json`, permitindo que clientes antigos continuem atualizando

## Licença

O Capivara Booster é distribuído sob a licença MIT. Veja [LICENSE](LICENSE).
