import { invoke } from '../tauri';
import { t } from '../i18n';
import { addLog, logCompletion, el } from '../shell';

interface CleanerItem { id: string; display_name: string; description: string; target_path: string; size_bytes: number; is_selected: boolean; }
interface CleaningSummary { total_bytes_cleaned: number; files_deleted: number; errors_count: number; }

function fmtBytes(b: number): string {
  if (b < 1024) return `${b} B`;
  if (b < 1024 ** 2) return `${(b / 1024).toFixed(1)} KB`;
  if (b < 1024 ** 3) return `${(b / 1024 ** 2).toFixed(1)} MB`;
  return `${(b / 1024 ** 3).toFixed(2)} GB`;
}

export async function renderCleaner(host: HTMLElement): Promise<void> {
  const items: CleanerItem[] = await invoke('get_cleaner_items');

  host.innerHTML = `
    <h1>${t('Cleaner_Title', 'Limpeza Profunda de Cache')}</h1>
    <p class="subtitle">${t('Cleaner_Subtitle', 'Temp, Prefetch, caches de shader e DNS.')}</p>

    <div class="card section">
      <div class="game-card">
        <div>
          <span class="text-muted" style="font-size:12px;">${t('Cleaner_TotalFound_Label', 'Total encontrado')}</span>
          <div style="font-size:28px;font-weight:700;color:var(--accent-cyan);" id="total-found">0 B</div>
        </div>
        <div class="actions">
          <button class="btn btn-secondary" id="btn-flush-dns">${t('Cleaner_FlushDns_Button', 'Flush DNS')}</button>
          <button class="btn btn-primary" id="btn-clean">${t('Cleaner_Purge_Button', 'Limpar Selecionados')}</button>
        </div>
      </div>
    </div>

    <div class="section">
      <h3>${t('Cleaner_Targets_Header', 'Alvos de Limpeza')}</h3>
      <div id="cleaner-list"></div>
    </div>`;

  const list = el<HTMLDivElement>('#cleaner-list');
  const refreshTotal = () => {
    const total = items.filter((i) => i.is_selected).reduce((sum, i) => sum + i.size_bytes, 0);
    const totalEl = el<HTMLDivElement>('#total-found');
    if (totalEl) totalEl.textContent = fmtBytes(total);
  };

  const renderRows = () => {
    if (!list) return;
    list.innerHTML = '';
    items.forEach((item, idx) => {
      const row = document.createElement('div');
      row.className = 'list-row';
      row.innerHTML = `
        <input type="checkbox" ${item.is_selected ? 'checked' : ''} data-idx="${idx}" />
        <div><div class="name">${item.display_name}</div><div class="desc">${item.description}</div><div class="desc text-muted mono">${item.target_path}</div></div>
        <span class="size-badge">${fmtBytes(item.size_bytes)}</span>`;
      row.querySelector('input')?.addEventListener('change', (e) => {
        items[idx].is_selected = (e.target as HTMLInputElement).checked;
        refreshTotal();
      });
      list.appendChild(row);
    });
    refreshTotal();
  };
  renderRows();

  el<HTMLButtonElement>('#btn-flush-dns')?.addEventListener('click', async () => {
    await invoke('flush_dns');
    addLog('🌐 ' + t('DNS_Flushed', 'Cache DNS limpo'));
  });

  el<HTMLButtonElement>('#btn-clean')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    let total: CleaningSummary = { total_bytes_cleaned: 0, files_deleted: 0, errors_count: 0 };
    try {
      for (const item of items.filter((i) => i.is_selected && i.id !== 'dns')) {
        addLog(`🧹 ${item.display_name}...`);
        const result = await invoke<CleaningSummary>('clean_item', { itemId: item.id });
        total.total_bytes_cleaned += result.total_bytes_cleaned;
        total.files_deleted += result.files_deleted;
        total.errors_count += result.errors_count;
      }
      addLog(`✅ ${total.files_deleted} ${t('Files_Cleaned', 'arquivos removidos')} (${fmtBytes(total.total_bytes_cleaned)})`);
      logCompletion(t('Cleaner_Finished', 'Limpeza concluída'));
      await renderCleaner(host);
    } catch (err) {
      addLog('❌ ' + String(err));
    } finally {
      btn.disabled = false;
    }
  });
}
