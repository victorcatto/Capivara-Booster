import { invoke } from '../tauri';
import { t } from '../i18n';
import { addLog, logCompletion, el } from '../shell';

interface GameDetection { game_id: string; found: boolean; executable_path: string; }
interface LiveTelemetry {
  cpu_name: string; cpu_load_percent: number;
  ram_total_gb: number; ram_used_gb: number;
  gpu_name: string; power_plan: string; uptime_hours: number;
}

export async function renderDashboard(host: HTMLElement): Promise<void> {
  host.innerHTML = `
    <h1>${t('Dashboard_Title', 'Visão Geral do Sistema')}</h1>
    <p class="subtitle">${t('Dashboard_Subtitle', 'Telemetria ao vivo do hardware e aceleração do sistema em um clique.')}</p>

    <div class="status-grid">
      <div class="status-card"><h4>${t('Dash_Cpu', 'Processador')}</h4><strong id="tm-cpu" class="text-muted">—</strong><small id="tm-cpu-load">—</small></div>
      <div class="status-card"><h4>${t('Dash_Ram', 'Memória')}</h4><strong id="tm-ram" class="text-muted">—</strong><div class="mini-bar"><div id="tm-ram-bar"></div></div><small id="tm-ram-pct">—</small></div>
      <div class="status-card"><h4>${t('Dash_Gpu', 'Placa de Vídeo')}</h4><strong id="tm-gpu" class="text-muted">—</strong><small id="tm-uptime">—</small></div>
      <div class="status-card"><h4>${t('Dash_Power', 'Plano de Energia')}</h4><strong id="tm-plan" class="text-muted">—</strong><small id="hpet-status">—</small></div>
    </div>

    <div class="status-grid" style="grid-template-columns:repeat(2,minmax(0,1fr));">
      <div class="status-card"><h4>${t('Dash_Latency_Badge', 'Latência')}</h4><strong id="latency-badge" class="text-muted">—</strong><small>HPET / Dynamic Tick</small></div>
      <div class="status-card"><h4>${t('Dash_Games_Card', 'Esports Suite')}</h4><strong id="games-summary">—</strong><small>CS2 / Valorant / LoL / PoE</small></div>
    </div>

    <div class="hero">
      <h2>${t('Dashboard_QuickBoost_Title', 'Aceleração Mestra em 1 Clique')}</h2>
      <p class="subtitle">${t('Dashboard_QuickBoost_Desc', 'Cria ponto de restauração, ativa Desempenho Máximo, otimiza rede e limpa caches.')}</p>
      <div class="progress"><div class="progress-bar" id="boost-progress"></div></div>
      <div class="actions">
        <button class="btn btn-primary" id="btn-quick-boost">${t('Dashboard_QuickBoost_Button', 'Executar Master Boost')}</button>
        <button class="btn btn-secondary" id="btn-standby">${t('Dash_Purge_Standby', 'Liberar RAM (Standby)')}</button>
        <button class="btn btn-secondary" id="btn-report">${t('Report_Button', 'Relatório Completo')}</button>
        <button class="btn btn-secondary" id="btn-refresh">${t('Dashboard_Refresh_Button', 'Atualizar')}</button>
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
    // Telemetria ao vivo: uma consulta só, sem travar a UI.
    try {
      const tm = await invoke<LiveTelemetry>('get_live_telemetry');
      const set = (id: string, v: string) => { const n = el<HTMLElement>(id); if (n) n.textContent = v; };
      set('#tm-cpu', tm.cpu_name || '—');
      set('#tm-cpu-load', t('Dash_Cpu_Load', 'Carga atual') + `: ${tm.cpu_load_percent}%`);
      set('#tm-ram', `${tm.ram_used_gb.toFixed(1)} / ${tm.ram_total_gb.toFixed(1)} GB`);
      const pct = tm.ram_total_gb > 0 ? Math.round((tm.ram_used_gb / tm.ram_total_gb) * 100) : 0;
      const bar = el<HTMLDivElement>('#tm-ram-bar');
      if (bar) bar.style.width = `${Math.min(pct, 100)}%`;
      set('#tm-ram-pct', `${pct}% ${t('Dash_Ram_Used', 'em uso')}`);
      set('#tm-gpu', tm.gpu_name || '—');
      set('#tm-uptime', t('Dash_Uptime', 'Ligado há') + ` ${tm.uptime_hours}h`);
      set('#tm-plan', tm.power_plan || '—');
    } catch { /* telemetria falhou: cards ficam com — */ }

    const hpetStatus = el<HTMLSpanElement>('#hpet-status');
    if (hpetStatus) {
      try {
        const disabled = await invoke<boolean>('get_hpet_status');
        hpetStatus.textContent = disabled ? 'HPET: Dynamic Tick ON' : 'HPET: padrão';
        hpetStatus.className = disabled ? 'text-success' : '';
      } catch { hpetStatus.textContent = 'HPET: N/A'; }
    }
    const latBadge = el<HTMLSpanElement>('#latency-badge');
    if (latBadge) {
      try {
        const disabled = await invoke<boolean>('get_hpet_status');
        latBadge.textContent = disabled ? t('Dash_Latency_Optimized', 'Otimizada') : t('Dash_Latency_Default', 'Padrão do Windows');
        latBadge.className = disabled ? 'text-success' : 'text-muted';
      } catch { latBadge.textContent = '—'; }
    }

    const games = ['cs2', 'valorant', 'lol'];
    const results = await Promise.all(games.map((g) => invoke<GameDetection>('detect_game', { gameId: g })));
    const poe = await invoke<Array<{ game: string; found: boolean }>>('detect_poe');
    const found = results.filter((r) => r.found).length + poe.filter((p) => p.found).length;
    const total = results.length + poe.length;
    const mark = (ok: boolean) => (ok ? '✔' : '✘');
    const summary = el<HTMLSpanElement>('#games-summary');
    if (summary) {
      summary.textContent = `${found}/${total} (CS2: ${mark(results[0].found)}, Valorant: ${mark(results[1].found)}, LoL: ${mark(results[2].found)}, PoE: ${mark(poe.some((p) => p.found))})`;
    }
  };
  refresh().catch(() => {});

  el<HTMLButtonElement>('#btn-refresh')?.addEventListener('click', () => refresh().catch(() => {}));

  el<HTMLButtonElement>('#btn-standby')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    try {
      await invoke('purge_standby_list');
      logCompletion(t('Dash_Standby_Done', 'RAM em cache liberada'));
      await refresh();
    } catch (err) { addLog('❌ ' + String(err)); }
    finally { btn.disabled = false; }
  });

  el<HTMLButtonElement>('#btn-report')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    btn.textContent = t('Report_Generating', 'Gerando relatório...');
    try {
      const path = await invoke<string>('generate_full_report');
      logCompletion(`${t('Report_Done', 'Relatório salvo na Área de Trabalho')}: ${path}`);
      btn.textContent = t('Report_Button', 'Relatório Completo');
    } catch (err) {
      addLog('❌ ' + String(err));
      btn.textContent = t('Report_Button', 'Relatório Completo');
    } finally { btn.disabled = false; }
  });

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
