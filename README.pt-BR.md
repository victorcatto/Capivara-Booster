<div align="center">

# CAPIVARA BOOSTER

### Otimização do Windows 10/11 para jogos competitivos

[![Última versão](https://img.shields.io/github/v/release/victorcatto/Capivara-Booster?style=for-the-badge&color=00E5AB)](https://github.com/victorcatto/Capivara-Booster/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/victorcatto/Capivara-Booster/release.yml?branch=main&style=for-the-badge&label=build%20de%20release)](https://github.com/victorcatto/Capivara-Booster/actions)
[![Windows 10/11](https://img.shields.io/badge/Windows-10%20%7C%2011%20x64-00B8FF?style=for-the-badge&logo=windows)](https://github.com/victorcatto/Capivara-Booster)
[![Licença](https://img.shields.io/badge/licença-MIT-8B949E?style=for-the-badge)](LICENSE)

[English version](README.md)

</div>

## O que é o Capivara Booster?

O Capivara Booster é uma ferramenta open source para Windows, criada para jogadores competitivos e usuários avançados. Ela reúne limpeza do sistema, controles de privacidade, ajustes de latência, perfis específicos para jogos, backups de configuração e reversão em um único aplicativo com controle de administrador.

A versão estável atual é a edição .NET 8/WPF. A edição Rust/Tauri v2 está sendo desenvolvida separadamente na branch `rust` e só substituirá a versão estável depois que a paridade funcional e a validação do instalador forem concluídas.

## Download

**[Baixar o instalador oficial mais recente](https://github.com/victorcatto/Capivara-Booster/releases/latest/download/CapivaraBooster-Setup.exe)**

Consulte o [histórico de releases](https://github.com/victorcatto/Capivara-Booster/releases) para hashes, changelogs e instaladores versionados.

### Requisitos

- Windows 10 ou Windows 11, 64 bits
- Permissão de administrador para alterações de sistema e registro
- .NET 8 Desktop Runtime para a edição WPF estável atual

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

### Conta e atualizações

- Troca de idioma PT-BR e EN-US em tempo real
- Cofre de contas local protegido pelo Windows DPAPI
- Derivação de senha PBKDF2-HMAC-SHA256 com salt aleatório
- Cadastro restrito a Gmail, Outlook, Yahoo e Yahoo Brasil
- Identidade única de administrador protegida internamente e não exposta na interface
- Manifesto de atualização no GitHub com verificação SHA-256
- Console de execução oculto por padrão e disponível por botão

## Modelo de segurança

O Capivara Booster altera configurações do Windows somente depois de uma ação explícita do usuário. Pontos de restauração e backups de arquivos são criados antes das operações destrutivas compatíveis. Os valores de registro alterados pelo aplicativo são rastreados para reversão em Backup & Restore.

Revise cada otimização antes de aplicá-la. Hardware, edição do Windows, drivers, política do antivírus e atualizações dos jogos podem alterar o resultado de um ajuste.

## Compilar a partir do código-fonte

```powershell
git clone https://github.com/victorcatto/Capivara-Booster.git
Set-Location Capivara-Booster

# Edição WPF estável
dotnet restore CapivaraBooster.sln
dotnet build CapivaraBooster.sln -c Release

# Migração Rust/Tauri v2
Set-Location src-tauri
cargo check
cargo test
```

A branch Rust exige Rust stable, ferramentas de compilação Windows MSVC, Node.js e Tauri CLI. O CI roda em `windows-latest` e valida frontend, compilação Rust, testes e bundle NSIS.

## Arquitetura

- Edição estável: C#/.NET 8, WPF, MVVM, Registro do Windows e integrações PowerShell
- Edição em migração: Rust, Tauri v2, frontend TypeScript, `winreg`, `windows-rs`, Tokio, DPAPI e compatibilidade com o atualizador do GitHub
- Pipeline de release: GitHub Actions com runners Windows e empacotamento NSIS

Leia o [blueprint da migração Rust](docs/RUST_MIGRATION_BLUEPRINT.md) para conhecer o roadmap.

## Status do projeto

- `main`: edição WPF estável e pipeline de produção
- `rust`: migração ativa, paridade funcional, revisão de segurança e validação do instalador
- A v2.0.0 só será publicada depois da checklist de paridade, da atualização limpa da instalação e da verificação ponta a ponta

## Licença

O Capivara Booster é distribuído sob a licença MIT. Veja [LICENSE](LICENSE).
