import { invoke } from '../tauri';
import { t } from '../i18n';
import { addLog, logCompletion, el } from '../shell';

interface BloatwareItem { id: string; name: string; package_pattern: string; description: string; category: string; is_selected: boolean; is_removed: boolean; }

export async function renderDebloat(host: HTMLElement): Promise<void> {
  const items: BloatwareItem[] = await invoke('get_bloatware_items');

  host.innerHTML = `
    <h1>${t('Debloat_Title', 'Windows Debloater & Privacidade')}</h1>
    <p class="subtitle">${t('Debloat_Subtitle', 'Remoção segura de bloatware UWP e desativação de telemetria. Estado real verificado no Windows; removível e restaurável.')}</p>

    <div class="card section">
      <div class="drv-toolbar">
        <h3>🔒 ${t('Telemetry_Title', 'Telemetria e Rastreamento')}</h3>
        <button class="btn btn-primary" id="btn-disable-telemetry">${t('Telemetry_Disable', 'Desativar Telemetria')}</button>
      </div>
      <p class="hint">${t('Telemetry_Desc', 'Desativa DiagTrack, dmwappushservice, 10 tarefas agendadas de telemetria (CEIP, Appraiser, Feedback) e chaves de anúncios.')}</p>
    </div>

    <div class="card section">
      <div class="drv-toolbar">
        <h3>🧹 ${t('Bloatware_Header', 'Bloatware UWP')}</h3>
        <div class="actions" style="margin:0;">
          <button class="btn btn-secondary btn-sm" id="btn-select-recommended">${t('Debloat_Recommended', 'Recomendados')}</button>
          <button class="btn btn-secondary btn-sm" id="btn-select-all">${t('Cleaner_SelectAll', 'Selecionar tudo')}</button>
          <button class="btn btn-secondary btn-sm" id="btn-deselect-all">${t('Cleaner_DeselectAll', 'Desmarcar tudo')}</button>
          <button class="btn btn-primary btn-sm" id="btn-remove">${t('Debloat_Remove_Selected', 'Remover selecionados')}</button>
        </div>
      </div>
      <div id="bloat-summary" class="drv-summary" style="display:none;"></div>
      <div class="drv-table-wrap">
        <table class="drv-table">
          <thead><tr>
            <th style="width:36px;"></th>
            <th>${t('Bloat_Col_Package', 'Pacote')}</th>
            <th>${t('Bloat_Col_Desc', 'Descrição')}</th>
            <th>${t('Bloat_Col_Status', 'Status')}</th>
            <th style="text-align:right;">${t('Cleaner_Col_Action', 'Ação')}</th>
          </tr></thead>
          <tbody id="bloat-tbody"></tbody>
        </table>
      </div>
    </div>`;

  const tbody = el<HTMLElement>('#bloat-tbody');
  const summary = el<HTMLDivElement>('#bloat-summary');

  // Estado real do Windows: marca "Instalado" ou "Não presente".
  const markRealState = async () => {
    try {
      const installed: string[] = await invoke('get_installed_bloatware');
      items.forEach((i) => { i.is_removed = !installed.includes(i.package_pattern); });
    } catch { /* sem verificação: usa o estado em memória */ }
  };

  const renderRows = () => {
    if (!tbody) return;
    const present = items.filter((i) => !i.is_removed);
    if (summary) {
      summary.style.display = '';
      summary.innerHTML = `
        <span class="drv-chip drv-warn">${present.length} ${t('Bloat_Present', 'presentes')}</span>
        <span class="drv-chip drv-ok">${items.length - present.length} ${t('Bloat_Removed', 'removidos')}</span>
        <span class="drv-chip drv-mute">${items.length} ${t('Drivers_Total', 'no total')}</span>`;
    }
    tbody.innerHTML = items.map((item, idx) => `
      <tr>
        <td><input type="checkbox" class="bloat-check" data-idx="${idx}" ${item.is_selected && !item.is_removed ? 'checked' : ''} ${item.is_removed ? 'disabled' : ''} /></td>
        <td class="drv-name" title="${item.package_pattern}">${item.name}</td>
        <td class="text-muted" style="max-width:340px;">${item.description}</td>
        <td><span class="drv-status ${item.is_removed ? 'drv-ok' : 'drv-mute'}">${item.is_removed ? t('Bloat_Removed', 'removidos') : t('Bloat_Installed', 'Instalado')}</span></td>
        <td class="drv-action">${item.is_removed
          ? `<button class="btn btn-secondary btn-sm btn-bloat-restore" data-pattern="${item.package_pattern}">${t('Bloat_Restore', 'Restaurar')}</button>`
          : ''}</td>
      </tr>`).join('');

    tbody.querySelectorAll<HTMLInputElement>('.bloat-check').forEach((c) => {
      c.addEventListener('change', () => { items[Number(c.dataset.idx)].is_selected = c.checked; });
    });
    tbody.querySelectorAll<HTMLButtonElement>('.btn-bloat-restore').forEach((b) => {
      b.addEventListener('click', async () => {
        b.disabled = true;
        b.textContent = t('Bloat_Restoring', 'Restaurando...');
        try {
          await invoke('restore_bloatware', { pattern: b.dataset.pattern });
          logCompletion(`${t('Bloat_Restored', 'Pacote restaurado')}: ${b.dataset.pattern}`);
          await markRealState();
          renderRows();
        } catch (err) { addLog('❌ ' + String(err)); b.disabled = false; b.textContent = t('Bloat_Restore', 'Restaurar'); }
      });
    });
  };

  await markRealState();
  renderRows();

  el<HTMLButtonElement>('#btn-select-all')?.addEventListener('click', () => {
    items.forEach((i) => { if (!i.is_removed) i.is_selected = true; });
    renderRows();
  });
  el<HTMLButtonElement>('#btn-deselect-all')?.addEventListener('click', () => {
    items.forEach((i) => { i.is_selected = false; });
    renderRows();
  });
  el<HTMLButtonElement>('#btn-select-recommended')?.addEventListener('click', () => {
    items.forEach((i) => { i.is_selected = i.is_selected && !i.is_removed ? true : false; });
    renderRows();
  });

  el<HTMLButtonElement>('#btn-remove')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    const selected = items.filter((i) => i.is_selected && !i.is_removed).map((i) => i.package_pattern);
    if (selected.length === 0) { addLog('⚠️ ' + t('Cleaner_NoSelection', 'Selecione ao menos um alvo')); return; }
    if (!confirm(t('Debloat_Confirm', `Remover ${selected.length} pacotes? Isso não pode ser desfeito facilmente.`))) return;
    btn.disabled = true;
    try {
      addLog(`🧹 ${t('Debloat_Removing', 'Removendo pacotes...')}`);
      const removed = await invoke<number>('remove_bloatware', { items: selected });
      await markRealState();
      addLog(`✅ ${removed} ${t('Packages_Removed', 'pacotes removidos')}`);
      logCompletion(t('Debloat_Finished', 'Remoção concluída'));
      renderRows();
    } catch (err) {
      addLog('❌ ' + String(err));
    } finally {
      btn.disabled = false;
    }
  });

  el<HTMLButtonElement>('#btn-disable-telemetry')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    try {
      await invoke('disable_telemetry');
      logCompletion(t('Telemetry_Disabled', 'Telemetria desativada'));
    } catch (err) {
      addLog('❌ ' + String(err));
    } finally {
      btn.disabled = false;
    }
  });
}
