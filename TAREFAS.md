# Tarefas do Capivara Booster

## Em andamento
- Validar a compilação Rust/Tauri v2 e o instalador NSIS no GitHub Actions.
- Auditar a paridade funcional com a versão WPF v1.1.4 antes de publicar a v2.0.0.
- Executar testes de integração em Windows para Registro, DPAPI, ponto de restauração, PowerShell, Defender, HPET e instalador.
- Testar atualização real de uma instalação v1.1.4 para o instalador Rust, usando o mesmo contrato `version.json` e os dois nomes de asset.
- Definir assinatura de código e política do SmartScreen antes da distribuição pública.

## Após paridade e lançamento Rust
- Remover os resquícios do projeto WPF/.NET: solução e fontes C#, XAML, workflow .NET/Inno Setup, scripts antigos e configuração obsoleta.
- Manter assets, documentação e dados de compatibilidade ainda usados pelo instalador/updater; só excluir depois de confirmar referências e usuários de v1.x.
- Centralizar o projeto em uma única árvore oficial no GitHub depois da migração Rust: comparar `main` e `rust`, integrar a implementação final na branch escolhida, atualizar o branch padrão e remover a branch redundante somente após confirmar que o histórico e o CI estão preservados.

## Depois do lançamento da v2.0.0
- Aplicar a nova identidade visual do Capivara Booster na UI Rust.
  - Paleta: Obsidian Slate `#0D1117`, Carbon Surface `#161B22`, Steel Line `#262F3D`.
  - Acentos: Cyber Mint `#00E5AB`, Volt Cyan `#00B8FF`.
  - Texto: Pure Frost `#F0F6FC`, Muted Silver `#8B949E`.
  - Atualizar logo, ícones de módulos, cards, botões e landing visual.
  - Preservar o conceito: frieza máxima, latência mínima.
  - Validar contraste, estados de foco, responsividade e PT-BR/EN-US.

## Dependências
- **Depende do Victor:** fornecer certificado de assinatura, se desejar reduzir o aviso do SmartScreen; disponibilizar uma máquina Windows de teste com instalação v1.1.4 para o upgrade real.
- **Depende do Claude:** concluir auditoria, validação e lançamento seguro da v2.0.0; integrar as branches e remover o WPF depois da aprovação técnica.
- **Depende de outra pessoa:** nenhuma além da autoridade certificadora, caso seja usada assinatura de código.
