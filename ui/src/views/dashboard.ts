import { invoke } from '../tauri';
import { t } from '../i18n';
import { addLog, logCompletion, el } from '../shell';

interface GameDetection { game_id: string; found: boolean; executable_path: string; }

export async function renderDashboard(host: HTMLElement): Promise<void> {
  host.innerHTML = `
    <h1>${t('Dashboard_Title', 'Visão Geral do Sistema')}</h1>
    <p class="subtitle">${t('Dashboard_Subtitle', 'Diagnósticos de timers de hardware e aceleração do sistema.')}</p>

    <div class="status-grid">
      <div class="status-card"><h4>${t('Dashboard_OsCard_Title', 'Sistema')}</h4><strong id="os-info">Windows 10/11 (64-bit)</strong><small style="color:var(--accent-cyan);">${t('Dashboard_OsCard_Target', 'Alvo: Windows 10/11')}</small></div>
      <div class="status-card"><h4>${t('Dashboard_HpetCard_Title', 'HPET')}</h4><strong id="hpet-status" class="text-muted">${t('Checking', 'Verificando...')}</strong><small>${t('Dashboard_HpetCard_Desc', 'Latência DPC')}</small></div>
      <div class="status-card"><h4>${t('Dashboard_PowerCard_Title', 'Plano de Energia')}</h4><strong class="text-success">Ultimate Performance</strong><small>${t('Dashboard_PowerCard_Desc', 'Clock travado')}</small></div>
      <div class="status-card"><h4>${t('Dashboard_GamesCard_Title', 'Jogos')}</h4><strong id="games-summary">${t('Checking', 'Verificando...')}</strong><small>CS2 / Valorant / LoL</small></div>
    </div>

    <div class="hero">
      <h2>${t('Dashboard_QuickBoost_Title', 'Aceleração Mestra em 1 Clique')}</h2>
      <p class="subtitle">${t('Dashboard_QuickBoost_Desc', 'Cria ponto de restauração, ativa Desempenho Máximo, otimiza rede e limpa caches.')}</p>
      <div class="progress"><div class="progress-bar" id="boost-progress"></div></div>
      <div class="actions">
        <button class="btn btn-primary" id="btn-quick-boost">${t('Dashboard_QuickBoost_Button', 'Executar Master Boost')}</button>
        <button class="btn btn-secondary" id="btn-refresh">${t('Dashboard_Refresh_Button', 'Atualizar Diagnósticos')}</button>
      </div>
    </div>

    <div class="card">
      <h3>🛡️ ${t('Dashboard_Safety_Title', 'Garantia de Segurança')}</h3>
      <p class="subtitle">${t('Dashboard_Safety_Desc', 'Ponto de restauração automático antes de qualquer alteração.')}</p>
      <div class="actions">
        <button class="btn btn-secondary" id="btn-restore-point">${t('Restore_Create_Point', 'Criar Ponto de Restauração')}</button>
        <button class="btn btn-danger" id="btn-revert-registry">${t('Backup_RevertRegistry', 'Reverter Tweaks de Registro')}</button>
      </div>
    </div>`;

  const refresh = async () => {
    const games = ['cs2', 'valorant', 'lol'];
    const results = await Promise.all(games.map((g) => invoke<GameDetection>('detect_game', { gameId: g })));
    const found = results.filter((r) => r.found).length;
    const summary = el<HTMLSpanElement>('#games-summary');
    if (summary) summary.textContent = `${found}/3 (CS2: ${results[0].found ? '✔' : '✘'}, Valorant: ${results[1].found ? '✔' : '✘'}, LoL: ${results[2].found ? '✔' : '✘'})`;
  };
  refresh().catch(() => {});

  el<HTMLButtonElement>('#btn-refresh')?.addEventListener('click', () => refresh().catch(() => {}));

  el<HTMLButtonElement>('#btn-quick-boost')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    const bar = el<HTMLDivElement>('#boost-progress');
    try {
      addLog('⚡ ' + t('Boost_Starting', 'Iniciando Master Boost...'));
      if (bar) bar.style.width = '20%';
      await invoke('create_restore_point', { description: 'CapivaraBooster_QuickBoost' });
      if (bar) bar.style.width = '45%';
      await invoke('apply_system_optimizations');
      if (bar) bar.style.width = '80%';
      await refresh();
      if (bar) bar.style.width = '100%';
      logCompletion(t('Boost_Done', 'Master Boost concluído'));
    } catch (err) {
      addLog('❌ ' + String(err));
    } finally {
      btn.disabled = false;
      setTimeout(() => { if (bar) bar.style.width = '0'; }, 1500);
    }
  });

  el<HTMLButtonElement>('#btn-restore-point')?.addEventListener('click', async () => {
    try {
      await invoke('create_restore_point', { description: 'CapivaraBooster_Manual' });
      addLog('🛡️ ' + t('Restore_Point_Created', 'Ponto de restauração criado'));
    } catch (err) { addLog('❌ ' + String(err)); }
  });

  el<HTMLButtonElement>('#btn-revert-registry')?.addEventListener('click', async () => {
    try {
      const n = await invoke<number>('revert_registry');
      addLog(`↩️ ${n} ${t('Registry_Reverted', 'valores de registro restaurados')}`);
    } catch (err) { addLog('❌ ' + String(err)); }
  });
}
