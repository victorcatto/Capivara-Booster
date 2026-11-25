import { invoke } from '../tauri';
import { t } from '../i18n';
import { addLog, logCompletion, el } from '../shell';

interface Cs2Install { found: boolean; exe_path: string; cfg_dir: string; video_txt: string; }
interface Cs2TweakStatus { id: string; applied: boolean; }

const TWEAK_LABEL: Record<string, string> = {
  mouse_accel_off: 'Aceleração do mouse off',
  mmcss_games: 'MMCSS: GPU Priority 8 e agendamento High',
  fso_gamedvr_off: 'Tela cheia exclusiva + Game DVR off',
  gpu_high: 'GPU de alto desempenho para cs2.exe',
  app_compat: 'AppCompatFlags: FSE exclusiva + DPI aware',
  ifeo_priority: 'Prioridade alta + núcleo 0 livre (IFEO)',
  power_ultimate: 'Plano de energia Desempenho Máximo',
  usb_suspend_off: 'Suspensão seletiva USB off',
  hpet_off: 'HPET/Dynamic Tick otimizado',
  nagle_off: 'Nagle/TCPNoDelay desativado',
  timer_05ms: 'Timer resolution 0,5 ms',
};

export async function renderCs2(host: HTMLElement): Promise<void> {
  const install: Cs2Install = await invoke('detect_cs2');

  host.innerHTML = `
    <h1>Counter-Strike 2</h1>
    <p class="subtitle">${t('Cs2_Subtitle', 'Perfil competitivo completo: registro, energia, timers, shaders e configs do jogo. Tudo com backup e reversão em um clique.')}</p>

    <div class="status-grid" style="grid-template-columns:repeat(2,minmax(0,1fr));">
      <div class="status-card"><h4>${t('Cs2_Install', 'Instalação')}</h4><strong id="cs2-found">${install.found ? t('Cs2_Found', 'Detectada') : t('Cs2_NotFound', 'Não encontrada')}</strong><small class="mono">${install.exe_path || 'Steam'}</small></div>
      <div class="status-card"><h4>${t('Cs2_Cfg', 'Pasta de configs')}</h4><strong class="mono" style="font-size:12px;">${install.cfg_dir || '—'}</strong><small class="mono">${install.video_txt ? 'cs2_video.txt OK' : 'cs2_video.txt ausente'}</small></div>
    </div>

    <div class="hero">
      <h2>${t('Cs2_Profile_Title', 'Perfil competitivo (1 clique)')}</h2>
      <p class="subtitle">${t('Cs2_Profile_Desc', 'Mouse sem aceleração, MMCSS no topo, tela cheia exclusiva, Game DVR off, GPU alta, prioridade de processo, energia, HPET, Nagle e timer 0,5 ms.')}</p>
      <div class="actions">
        <button class="btn btn-primary" id="btn-cs2-profile" ${install.found ? '' : 'disabled'}>${t('Cs2_ApplyProfile', 'Aplicar perfil completo')}</button>
        <button class="btn btn-secondary" id="btn-cs2-configs" ${install.cfg_dir ? '' : 'disabled'}>${t('Cs2_ApplyConfigs', 'Aplicar configs do jogo')}</button>
        <button class="btn btn-secondary" id="btn-cs2-cache" ${install.found ? '' : 'disabled'}>${t('Cs2_ClearCache', 'Limpar shader cache')}</button>
      </div>
      <p class="hint">${t('Cs2_SteamClosed_Note', 'Feche a Steam antes de aplicar as configs: o jogo sobrescreve os arquivos ao encerrar.')}</p>
    </div>

    <div class="card section">
      <h3>${t('Cs2_Status_Header', 'Status dos tweaks')}</h3>
      <div id="cs2-status" class="text-muted">${t('Cs2_Status_Empty', 'Aplique o perfil para ver o resultado de cada tweak aqui.')}</div>
    </div>`;

  el<HTMLButtonElement>('#btn-cs2-profile')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    addLog('🎯 ' + t('Cs2_Applying', 'Aplicando perfil CS2...'));
    try {
      const status: Cs2TweakStatus[] = await invoke('apply_cs2_profile');
      const host2 = el<HTMLDivElement>('#cs2-status');
      if (host2) {
        host2.innerHTML = status.map((s) => `
          <div class="list-row">
            <div class="name">${TWEAK_LABEL[s.id] || s.id}</div>
            <span class="drv-status ${s.applied ? 'drv-ok' : 'drv-warn'}">${s.applied ? t('Cs2_Applied', 'Aplicado') : t('Cs2_Failed', 'Falhou')}</span>
          </div>`).join('');
      }
      const ok = status.filter((s) => s.applied).length;
      logCompletion(`${t('Cs2_Profile_Done', 'Perfil CS2 aplicado')}: ${ok}/${status.length}`);
    } catch (err) {
      addLog('❌ ' + String(err));
    } finally {
      btn.disabled = false;
    }
  });

  el<HTMLButtonElement>('#btn-cs2-configs')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    try {
      const rate = Number((navigator as unknown as { devicePixelRatio: number }).devicePixelRatio) > 0 ? 240 : 240;
      const results: string[] = await invoke('apply_cs2_configs', { refreshRate: rate });
      results.forEach((r) => addLog(`✅ ${r}`));
      logCompletion(t('Cs2_Configs_Done', 'cs2_video.txt, autoexec.cfg e launch options aplicados'));
    } catch (err) {
      addLog('❌ ' + String(err));
    } finally {
      btn.disabled = false;
    }
  });

  el<HTMLButtonElement>('#btn-cs2-cache')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    try {
      const [bytes, files] = await invoke<[number, number]>('clear_cs2_caches');
      logCompletion(`${t('Cs2_Cache_Done', 'Shader cache limpo')}: ${(bytes / 1_048_576).toFixed(1)} MB / ${files} ${t('Cleaner_Files', 'arquivos')}`);
    } catch (err) {
      addLog('❌ ' + String(err));
    } finally {
      btn.disabled = false;
    }
  });
}
