## Added

- Live dashboard telemetry: CPU model with current load %, RAM usage bar, GPU name, active power plan and uptime, all from a single PowerShell query; new "Free RAM (Standby)" action via NtSetSystemInformation
- Cache & Junk Cleaner redesigned: summary hero (total found, selected, freeable now), professional table with category and real on-disk sizes, Select All / Recommended presets, per-item progress and safely skipped in-use files reported at the end
- Update completion feedback: typed DownloadOutcome (bytes, path, launched) with a localized success banner; Windows Update driver installs now surface reboot-required explicitly
- CI/CD: every release now syncs READMEs, defines the repo homepage as the landing page and regenerates releases-data.json without BOM

## Changed

- All PowerShell/CMD scripts remain windowless (CREATE_NO_WINDOW) and log exact [OK] results per module

## Fixed

- In-app updater stuck without completion signal after download
- Driver status row not updating after a successful one-click update

## Performance

- Dashboard telemetry consolidated into one WMI/PowerShell round-trip (was several)

## PT-BR

- Telemetria ao vivo no painel: CPU com carga, barra de RAM, GPU, plano de energia e uptime em uma consulta; botão Liberar RAM (Standby)
- Limpar Cache repaginado: resumo (total, selecionados, liberável agora), tabela com categoria e tamanhos reais, Selecionar tudo/Recomendados, progresso por item e arquivos em uso pulados com segurança
- Confirmação de conclusão do atualizador com banner localizado; atualização de driver agora avisa reinício obrigatório
- CI/CD: cada release sincroniza READMEs, define a homepage do repo e regenera o JSON da landing sem BOM
