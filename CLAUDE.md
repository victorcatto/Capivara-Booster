# Projetos de Victor

Pasta com todos os projetos dele. Cada projeto tem o próprio `CLAUDE.md`,
`TAREFAS.md` e afins; aqui fica só o que vale para todos. Língua, jeito de
responder, autonomia, economia de tokens, navegador, processos visíveis e
método ficam em `~/.claude/CLAUDE.md` e não se repetem aqui.

- **Antes de mexer num projeto:** abrir o `CLAUDE.md` dele (quando existir).
- Manter este arquivo curto: detalhe de um projeto só vai para o `CLAUDE.md`
  daquele projeto.

## Concorrente: nunca mencionar

Em nenhum texto de nenhum projeto (patch note, interface, erro, site, post,
e-mail, commit, comentário, documento): nada de nome de produto do mesmo ramo,
nem "como no X", "melhor que o X" ou "feche o X". Descrever o que o nosso faz,
sem comparar. Antes de publicar qualquer texto, procurar nome de concorrente
nele. Os nomes a barrar ficam na linha `CONCORRENTE` do
`~/.claude/hooks/texto-proibido.js`.

## Interface: nunca

Badge com bolinha piscando ("● Ao vivo", "● Online"), texto piscando, glow
pulsando em loop, emoji fazendo papel de ícone em interface séria. Tem cara de
coisa feita por IA e passa falta de profissionalismo.

## Web pública

Todo site, landing ou página de venda aberta ao público segue a skill
**`entrega-web`** (SEO, AEO, leitura por IA, og:image, desempenho 90+,
celular, acessibilidade, Lighthouse antes de entregar, Search Console depois),
sem ele pedir. Página com login fica só com desempenho, celular e
acessibilidade.

## Tarefas e bloqueios

- `TAREFAS.md` (ou equivalente) guarda **só o que falta**. Concluiu, a tarefa
  sai no mesmo passe e vai para `TAREFAS-FINALIZADAS.md`, na mesma pasta:
  agrupada por data, a mais recente em cima, uma linha. Observação de campo
  (só dá para ver quando acontecer) vai para o documento de estado do projeto.
- O classificador do **modo auto** barrou uma ação: anotar em
  `Projetos/bloqueado.md` (data, projeto, motivo, ferramenta, o que se
  queria), nunca o segredo, só o alvo. Quando ele trocar para **edit
  automatically** ou pedir para retomar, a primeira coisa é ler o
  `bloqueado.md` e oferecer o que ficou parado. Resolvido (ou que não vale
  mais) sai dele e vai para o topo do `Projetos/bloqueado-finalizado.md`, com
  a data e como se resolveu. Nada de `[x]` no `bloqueado.md`; nada se apaga.

## Credenciais e o PC dele

- Credencial fica no `.env` da pasta raiz do projeto, uma chave por
  credencial. Faltou uma: criar a chave vazia e dizer só "preencha o `.env`
  com a credencial X".
- **Conta Google: uma só, sempre pelo botão "Entrar com o Google"**, nunca
  e-mail e senha do próprio site. `GOOGLE_EMAIL` e `GOOGLE_SENHA` ficam só no
  `Projetos/.env`: não repetir em `.env` de projeto.
- Código de 2 fatores, captcha ou aprovação no app do banco: pedir no momento
  exato, com tudo pronto para seguir.
- Mouse e teclado reais do PC dele: pode, pedindo **só na hora de usar**, com
  tudo pronto, dizendo o que vai fazer e por quanto tempo. Recebeu o OK,
  começar na hora; terminou, avisar na hora que liberou.

## CPU e processos

- **Fila de CPU:** o hook `~/.claude/hooks/fila-cpu.js` (PreToolUse em Bash,
  PowerShell, Agent e Workflow) segura o comando até a CPU ficar abaixo de
  99% e solta uma sessão por vez, com 15 s entre elas; depois de 15 min,
  recusa. Esperar e tentar de novo, nunca contornar. Mudar o limite só por
  ordem dele. Motivo: com a CPU em 100%, o PC trava o que já está rodando.
- **Processo esquecido:** no começo de toda tarefa, quando a fila segurar um
  comando ou quando o PC estiver pesado, medir 10 s de CPU por processo
  (comando abaixo) e **fechar sem perguntar** o que sobrou de sessão do
  Claude: `bash`, `python`, `node`, `grep`, `tail`, `sleep` cuja cadeia de
  pais não chega a um `claude.exe`, `Code.exe` ou `explorer.exe` vivo;
  navegador de teste; servidor de dev sem uso.
- **Nunca fechar:** sessão do Claude viva e programa que o Victor abriu
  (Discord, VS Code, o Chrome dele, jogo); se pesar, dizer numa linha quanto
  gasta.
- No modo auto, o classificador barra o `Stop-Process`: anotar no
  `bloqueado.md` e passar nome e PID ao Victor para ele fechar.

```powershell
# CPU gasta por processo em 10 s (os 12 que mais gastam)
$a = Get-Process | Select-Object Id, ProcessName, CPU; Start-Sleep 10; Get-Process | ForEach-Object { $p = $_; $o = $a | Where-Object Id -eq $p.Id; if ($o -and $p.CPU) { [pscustomobject]@{Id=$p.Id; Nome=$p.ProcessName; Seg=[math]::Round($p.CPU - $o.CPU, 1)} } } | Sort-Object Seg -Descending | Select-Object -First 12

# Candidatos a órfão: processo de sessão cujo pai já morreu (conferir a linha antes de fechar)
$vivos = @{}; Get-Process | ForEach-Object { $vivos[$_.Id] = 1 }; Get-CimInstance Win32_Process | Where-Object { $_.Name -match '^(bash|python|node|grep|tail|sleep)\.exe$' -and -not $vivos[[int]$_.ParentProcessId] } | Select-Object ProcessId, Name, CreationDate, CommandLine
```

## O que fica no PC

- Saída de build e lixo de ferramenta (log velho, pasta temporária) se apagam.
- **Só no GitHub:** projeto parado ou site de cliente sem alteração pendente
  pode ficar só no repo. Para mexer, trazer o código de volta dentro da pasta,
  sem tocar no que ficou fora do git:
  `git clone --no-checkout https://github.com/<dono>/<repo>.git .tmp && mv .tmp/.git . && rmdir .tmp && git reset --hard`
- No Desktop ficam só esta pasta e atalhos.

## Git e GitHub

- Conta do GitHub dele, com o `gh` logado.
- Todo repo é **privado**: é backup, não vitrine. Credencial e documento
  interno sobem junto; fica fora só dado de pessoa real (lead, e-mail de
  cliente, dado de saúde, senha de cliente) e arquivo acima de 100 MB. Se um
  repo for virar público, reescrever o histórico antes.

## SSH

Em hospedagem compartilhada, rajada de conexões derruba a sessão: agrupar os
comandos numa conexão só. O apelido de cada servidor é o `Host` do
`~/.ssh/config` (`ssh <apelido>`).
