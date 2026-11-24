import { invoke } from '../tauri';
import { t } from '../i18n';
import { addLog, el } from '../shell';

interface StartupItem { name: string; path: string; location: string; }

export async function renderBoost(host: HTMLElement): Promise<void> {
  host.innerHTML = `
    <h1>${t('Boost_Title', 'Latência do Sistema')}</h1>
    <p class="subtitle">${t('Boost_Subtitle', 'HPET, plano de energia, Game DVR e rede.')}</p>

    <div class="card section">
      <h3>⚡ ${t('Boost_Tweaks_Title', 'Otimizações Globais de Latência')}</h3>
      <p class="subtitle">${t('Boost_Tweaks_Desc', 'Game DVR off, Network Throttling off, prioridade de jogos e apps em segundo plano.')}</p>
      <div class="actions">
        <button class="btn btn-primary" id="btn-apply">${t('Boost_Apply', 'Aplicar Otimizações')}</button>
        <button class="btn btn-secondary" id="btn-revert">${t('Boost_Revert', 'Reverter ao Padrão')}</button>
      </div>
    </div>

    <div class="section">
      <h3>🚀 ${t('Startup_Header', 'Itens de Inicialização')}</h3>
      <div id="startup-list"></div>
    </div>`;

  const startup = await invoke<StartupItem[]>('get_startup_items').catch(() => []);
  const list = el<HTMLDivElement>('#startup-list');
  if (list) {
    list.innerHTML = startup.length === 0
      ? `<p class="text-muted">${t('Startup_Empty', 'Nenhum item de inicialização encontrado.')}</p>`
      : startup.map((s) => `
        <div class="list-row">
          <span></span>
          <div><div class="name">${s.name}</div><div class="desc mono">${s.path}</div><div class="desc text-muted">${s.location}</div></div>
          <span></span>
        </div>`).join('');
  }

  el<HTMLButtonElement>('#btn-apply')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    try {
      await invoke('apply_system_optimizations');
      addLog('✅ ' + t('Boost_Applied', 'Otimizações de latência aplicadas'));
    } catch (err) {
      addLog('❌ ' + String(err));
    } finally {
      btn.disabled = false;
    }
  });

  el<HTMLButtonElement>('#btn-revert')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    try {
      await invoke('revert_system_optimizations');
      addLog('↩️ ' + t('Boost_Reverted', 'Ajustes revertidos ao padrão'));
    } catch (err) {
      addLog('❌ ' + String(err));
    } finally {
      btn.disabled = false;
    }
  });
}
