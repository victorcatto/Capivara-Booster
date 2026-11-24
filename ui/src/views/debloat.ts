import { invoke } from '../tauri';
import { t } from '../i18n';
import { addLog, el } from '../shell';

interface BloatwareItem { id: string; name: string; package_pattern: string; description: string; category: string; is_selected: boolean; is_removed: boolean; }

export async function renderDebloat(host: HTMLElement): Promise<void> {
  const items: BloatwareItem[] = await invoke('get_bloatware_items');

  host.innerHTML = `
    <h1>${t('Debloat_Title', 'Windows Debloater & Privacidade')}</h1>
    <p class="subtitle">${t('Debloat_Subtitle', 'Remoção segura de bloatware UWP e desativação de telemetria.')}</p>

    <div class="card section">
      <h3>🔒 ${t('Telemetry_Title', 'Telemetria e Rastreamento')}</h3>
      <p class="subtitle">${t('Telemetry_Desc', 'Desativa DiagTrack, dmwappushservice e tarefas agendadas de telemetria.')}</p>
      <div class="actions">
        <button class="btn btn-primary" id="btn-disable-telemetry">${t('Telemetry_Disable', 'Desativar Telemetria')}</button>
      </div>
    </div>

    <div class="section">
      <h3>🧹 ${t('Bloatware_Header', 'Bloatware UWP')}</h3>
      <div class="actions" style="margin-bottom:12px;">
        <button class="btn btn-secondary btn-sm" id="btn-select-all">${t('Cleaner_SelectAll', 'Selecionar tudo')}</button>
        <button class="btn btn-secondary btn-sm" id="btn-deselect-all">${t('Cleaner_DeselectAll', 'Desmarcar tudo')}</button>
        <button class="btn btn-primary" id="btn-remove">${t('Debloat_Remove_Selected', 'Remover Selecionados')}</button>
      </div>
      <div id="bloat-list"></div>
    </div>`;

  const list = el<HTMLDivElement>('#bloat-list');
  const renderRows = () => {
    if (!list) return;
    list.innerHTML = '';
    items.forEach((item, idx) => {
      const row = document.createElement('div');
      row.className = 'list-row';
      row.innerHTML = `
        <input type="checkbox" ${item.is_selected ? 'checked' : ''} ${item.is_removed ? 'disabled' : ''} data-idx="${idx}" />
        <div><div class="name">${item.name} <span class="text-muted" style="font-size:11px;">${item.category}</span></div>
        <div class="desc">${item.description}</div>
        <div class="desc text-muted mono">${item.package_pattern}</div></div>
        ${item.is_removed ? '<span class="text-success">✓</span>' : '<span></span>'}`;
      row.querySelector('input')?.addEventListener('change', (e) => {
        items[idx].is_selected = (e.target as HTMLInputElement).checked;
      });
      list.appendChild(row);
    });
  };
  renderRows();

  el<HTMLButtonElement>('#btn-select-all')?.addEventListener('click', () => {
    items.forEach((i) => { if (!i.is_removed) i.is_selected = true; });
    renderRows();
  });
  el<HTMLButtonElement>('#btn-deselect-all')?.addEventListener('click', () => {
    items.forEach((i) => { i.is_selected = false; });
    renderRows();
  });

  el<HTMLButtonElement>('#btn-remove')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    const selected = items.filter((i) => i.is_selected && !i.is_removed).map((i) => i.package_pattern);
    if (selected.length === 0) return;
    if (!confirm(t('Debloat_Confirm', `Remover ${selected.length} pacotes? Isso não pode ser desfeito facilmente.`))) return;
    btn.disabled = true;
    try {
      addLog(`🧹 ${t('Debloat_Removing', 'Removendo pacotes...')}`);
      const removed = await invoke<number>('remove_bloatware', { items: selected });
      items.forEach((i) => { if (selected.includes(i.package_pattern)) i.is_removed = true; });
      addLog(`✅ ${removed} ${t('Packages_Removed', 'pacotes removidos')}`);
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
      addLog('✅ ' + t('Telemetry_Disabled', 'Telemetria desativada'));
    } catch (err) {
      addLog('❌ ' + String(err));
    } finally {
      btn.disabled = false;
    }
  });
}
