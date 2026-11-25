import { invoke } from '../tauri';
import { t } from '../i18n';
import { addLog, logCompletion, el } from '../shell';

interface PoeInstall {
  game: 'poe1' | 'poe2';
  found: boolean;
  config_path: string;
  source: 'Standalone' | 'Steam' | 'Epic' | 'Unknown';
}

interface PoeOptions {
  global_illumination_off: boolean;
  shadows_min: boolean;
  screen_shake_off: boolean;
  dynamic_culling: boolean;
  dynamic_resolution: boolean;
  renderer_dx12: boolean;
  sound_channels_low: boolean;
  mute_effects_keep_loot: boolean;
  lockstep: boolean;
}

const OPTION_LABELS: Array<{ key: keyof PoeOptions; pt: string; en: string; tip: string }> = [
  { key: 'global_illumination_off', pt: 'Desativar iluminação global', en: 'Disable global illumination', tip: 'Principal gargalo de CPU em mapas densos: ganho de 30 a 75% de FPS.' },
  { key: 'shadows_min', pt: 'Sombras no mínimo', en: 'Minimum shadows', tip: 'Reduz draw-calls da CPU.' },
  { key: 'screen_shake_off', pt: 'Desativar screen shake', en: 'Disable screen shake', tip: 'Estabiliza o frame-time em explosões de partículas.' },
  { key: 'dynamic_culling', pt: 'Culling dinâmico', en: 'Dynamic culling', tip: 'Mantém o FPS estável com dezenas de habilidades simultâneas.' },
  { key: 'dynamic_resolution', pt: 'Resolução dinâmica', en: 'Dynamic resolution', tip: 'Escala a resolução para segurar o frame-time.' },
  { key: 'renderer_dx12', pt: 'Renderer DirectX 12 (desmarcado: Vulkan)', en: 'DirectX 12 renderer (unchecked: Vulkan)', tip: 'DX12 para GPUs modernas; Vulkan costuma ir melhor em GPUs AMD.' },
  { key: 'sound_channels_low', pt: 'Canais de som no baixo', en: 'Low sound channels', tip: 'O áudio do PoE consome muita CPU em combate intenso.' },
  { key: 'mute_effects_keep_loot', pt: 'Mutar efeitos mantendo filtro de loot', en: 'Mute effects keeping loot filter', tip: 'Avançado: elimina o processamento de áudio de habilidades, mantendo o som do filtro de loot.' },
  { key: 'lockstep', pt: 'Lockstep (desmarcado: Predictive)', en: 'Lockstep (unchecked: Predictive)', tip: 'Lockstep para ping < 60ms e Hardcore; Predictive para rotas instáveis.' },
];

export async function renderPoe(host: HTMLElement): Promise<void> {
  const installs: PoeInstall[] = await invoke('detect_poe');
  const options: PoeOptions = {
    global_illumination_off: true,
    shadows_min: true,
    screen_shake_off: true,
    dynamic_culling: true,
    dynamic_resolution: true,
    renderer_dx12: true,
    sound_channels_low: true,
    mute_effects_keep_loot: false,
    lockstep: true,
  };

  const poe1 = installs.find((i) => i.game === 'poe1');
  const poe2 = installs.find((i) => i.game === 'poe2');

  host.innerHTML = `
    <h1>${t('Poe_Title', 'Path of Exile (PoE 1 & PoE 2)')}</h1>
    <p class="subtitle">${t('Poe_Subtitle', 'Perfil competitivo da engine GGG: gráficos, áudio, shaders, rede e CPU com backup automático.')}</p>

    <div class="status-grid">
      <div class="status-card">
        <h4>PoE 1</h4>
        <strong class="${poe1?.found ? 'text-success' : 'text-muted'}">${poe1?.found ? t('Poe_Found', 'Instalado') : t('Poe_NotFound', 'Não encontrado')}</strong>
        ${poe1?.found ? `<small class="mono">${poe1.source}</small>` : ''}
      </div>
      <div class="status-card">
        <h4>PoE 2</h4>
        <strong class="${poe2?.found ? 'text-success' : 'text-muted'}">${poe2?.found ? t('Poe_Found', 'Instalado') : t('Poe_NotFound', 'Não encontrado')}</strong>
        ${poe2?.found ? `<small class="mono">${poe2.source}</small>` : ''}
      </div>
    </div>

    <div class="card section">
      <h3>⚙️ ${t('Poe_Options_Header', 'Otimizações')}</h3>
      <div id="poe-options"></div>
      <div class="actions">
        <button class="btn btn-primary" id="btn-poe-apply-poe1" ${poe1?.found ? '' : 'disabled'}>${t('Poe_Apply_Poe1', 'Aplicar Perfil Competitivo: PoE 1')}</button>
        <button class="btn btn-primary" id="btn-poe-apply-poe2" ${poe2?.found ? '' : 'disabled'}>${t('Poe_Apply_Poe2', 'Aplicar Perfil Competitivo: PoE 2')}</button>
        <button class="btn btn-secondary" id="btn-poe-cache">${t('Poe_Clear_Cache', 'Limpar Shader Cache')}</button>
      </div>
      <p class="hint">${t('Poe_Backup_Note', 'Um backup do .ini original é criado automaticamente antes de qualquer alteração.')}</p>
    </div>`;

  const list = el<HTMLDivElement>('#poe-options');
  if (list) {
    list.innerHTML = OPTION_LABELS.map((option, idx) => `
      <div class="list-row">
        <input type="checkbox" id="poe-opt-${idx}" ${options[option.key] ? 'checked' : ''} />
        <div>
          <div class="name">${t(`Poe_Opt_${option.key}`, option.pt)}</div>
          <div class="desc text-muted">${option.tip}</div>
        </div>
        <span></span>
      </div>`).join('');
    OPTION_LABELS.forEach((option, idx) => {
      el<HTMLInputElement>(`#poe-opt-${idx}`)?.addEventListener('change', (e) => {
        options[option.key] = (e.target as HTMLInputElement).checked;
      });
    });
  }

  const apply = async (game: 'poe1' | 'poe2') => {
    try {
      // Backup automático antes de qualquer alteração
      await invoke('create_restore_point', { description: `CapivaraBooster_PoE_${game === 'poe1' ? '1' : '2'}_Backup` });
      addLog(`🛡️ Backup criado: PoE ${game === 'poe1' ? '1' : '2'} config`);

      // Aplicar perfil competitivo
      const path = await invoke<string>('apply_poe_profile', { game, options });
      addLog(`✅ ${path}`);

      // Boost ao vivo: se o jogo já está aberto, prioridade alta + timer 0.5ms agora.
      try {
        await invoke('boost_live_poe', { game });
        addLog('🎮 ' + t('Poe_LiveBoost', 'Jogo em execução detectado: prioridade alta e timer 0.5ms aplicados'));
      } catch { /* jogo fechado: boost vale na próxima abertura via IFEO/GPU pref */ }

      logCompletion(t('Poe_Applied', 'Perfil PoE aplicado com sucesso'));
    } catch (err) {
      addLog('❌ ' + String(err));
    }
  };

  el<HTMLButtonElement>('#btn-poe-apply-poe1')?.addEventListener('click', () => apply('poe1').catch((err) => addLog('❌ ' + String(err))));
  el<HTMLButtonElement>('#btn-poe-apply-poe2')?.addEventListener('click', () => apply('poe2').catch((err) => addLog('❌ ' + String(err))));

  el<HTMLButtonElement>('#btn-poe-cache')?.addEventListener('click', async () => {
    try {
      const cleared = await invoke<[number, number]>('clear_poe_shader_cache');
      addLog(`🧹 Shader cache: ${(cleared[0] / 1024 / 1024).toFixed(1)} MB em ${cleared[1]} arquivos`);
      logCompletion(t('Poe_Cache_Cleared', 'Shader cache do PoE limpo'));
    } catch (err) {
      addLog('❌ ' + String(err));
    }
  });
}