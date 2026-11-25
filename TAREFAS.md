# Tarefas do Capivara Booster

## Concluído em 2026-11-24
- Unificação das árvores: `main` agora contém a edição Rust (Tauri v2) e a branch `rust` foi removida.
- Remoção do legado WPF/.NET: solução, fontes C#, XAML, workflow Inno Setup e scripts antigos.
- Pipeline de release Rust: tags `v2.*` compilam Tauri + NSIS, geram SHA-256 e publicam o release.
- Remoção do e-mail confidencial de administrador de todos os arquivos (agora só hash interno).
- READMEs profissionalizados e atualizados para a edição Rust unificada.
- Publicação da tag v2.0.0 disparando o build de release no GitHub Actions.

## Em andamento
- Acompanhar o build de release v2.0.0 e confirmar os assets publicados.
- Testes reais em Windows (Registro, DPAPI, ponto de restauração, HPET, Defender) ficam com o Victor nesta máquina.

## Depois do lançamento da v2.0.0
- Aplicar a nova identidade visual do Capivara Booster na UI Rust.
  - Paleta: Obsidian Slate `#0D1117`, Carbon Surface `#161B22`, Steel Line `#262F3D`.
  - Acentos: Cyber Mint `#00E5AB`, Volt Cyan `#00B8FF`.
  - Texto: Pure Frost `#F0F6FC`, Muted Silver `#8B949E`.
  - Atualizar logo, ícones de módulos, cards, botões e landing visual.
  - Preservar o conceito: frieza máxima, latência mínima.
  - Validar contraste, estados de foco, responsividade e PT-BR/EN-US.

## Dependências
- **Depende do Victor:** testes locais do instalador v2 e feedback funcional; certificado de assinatura, se desejar reduzir o aviso do SmartScreen.
- **Depende do Claude:** confirmar o release, manter CI verde e evoluir a UI após o lançamento.
- **Depende de outra pessoa:** autoridade certificadora, caso seja usada assinatura de código.

## Em andamento (v2.1.2)
- Corrigir versão travada em 2.0.0 no canto superior esquerdo (deve acompanhar tauri.conf.json).
- Menu lateral exibindo chaves cruas ([Nav_Poe], [Nav_Drivers], [Nav_LowLatency]): i18n deve carregar antes da primeira renderização.
- Erro de acesso negado (os error 5) em GameDVR e IFEO do CS2: garantir elevação real (manifest requireAdministrator embutido via build.rs).
- Banner de atualização no dashboard quando há versão nova (console mostra, UI não oferece botão).
- Driver Booster completo: comparar data do driver local com o último do site do fabricante; se o do site for mais novo, oferecer link de download e instalação; se o local for atual, mostrar "Updated".
- Revisar e melhorar os scripts de otimização com foco em Path of Exile 2 e Counter-Strike 2.
- Melhorar o layout da página do Path of Exile e ajustar os menus do app (organização, espaçamento e hierarquia).
- Pesquisar na internet por novos scripts de otimização testados e validar cada um antes de adicionar: disponibilizar o máximo possível com confirmação de funcionamento.
- Revisão gramatical completa (PT-BR e EN-US) da landing page e do README principal do GitHub: texto profissional, bem editado, com as informações mais importantes em destaque.
- Aumentar o tamanho dos ícones do menu esquerdo e revisar o layout para responsividade máxima.
- Mais funções de otimização focadas em Path of Exile 2.
