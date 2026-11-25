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
    <p class="subtitle">${t('Cleaner_Subtitle', 'Remove temporários, caches de shader e resíduos do Windows com segurança: arquivos em uso são pulados automaticamente e nada de sistema é tocado sem backup.')}</p>

    <div class="cleaner-hero">
      <div class="cleaner-hero-stats">
        <div class="cleaner-stat">
          <span>${t('Cleaner_TotalFound_Label', 'Total encontrável')}</span>
          <strong id="total-found">${fmtBytes(items.reduce((s, i) => s + i.size_bytes, 0))}</strong>
        </div>
        <div class="cleaner-stat">
          <span>${t('Cleaner_Selected_Label', 'Selecionados')}</span>
          <strong id="sel-count">${items.filter((i) => i.is_selected).length}/${items.length}</strong>
        </div>
        <div class="cleaner-stat">
          <span>${t('Cleaner_Selected_Size', 'Liberável agora')}</span>
          <strong id="total-selected" style="color:var(--accent-cyan);">${fmtBytes(items.filter((i) => i.is_selected).reduce((s, i) => s + i.size_bytes, 0))}</strong>
        </div>
      </div>
      <div class="cleaner-hero-actions">
        <button class="btn btn-secondary" id="btn-select-all">${t('Cleaner_SelectAll', 'Selecionar tudo')}</button>
        <button class="btn btn-secondary" id="btn-select-recommended">${t('Cleaner_SelectRecommended', 'Recomendados')}</button>
        <button class="btn btn-secondary" id="btn-flush-dns">${t('Cleaner_FlushDns_Button', 'Flush DNS')}</button>
        <button class="btn btn-primary" id="btn-clean">${t('Cleaner_Purge_Button', 'Limpar selecionados')}</button>
      </div>
    </div>

    <div class="card section">
      <div class="drv-toolbar">
        <h3>${t('Cleaner_Targets_Header', 'Alvos de limpeza')}</h3>
        <span class="text-muted" style="font-size:11.5px;">${t('Cleaner_ScanNote', 'Tamanhos medidos no disco nesta máquina')}</span>
      </div>
      <div class="drv-table-wrap">
        <table class="drv-table">
          <thead><tr>
            <th style="width:36px;"></th>
            <th>${t('Cleaner_Col_Target', 'Alvo')}</th>
            <th>${t('Cleaner_Col_Category', 'Categoria')}</th>
            <th>${t('Cleaner_Col_Size', 'Tamanho')}</th>
            <th style="text-align:right;">${t('Cleaner_Col_Action', 'Ação')}</th>
          </tr></thead>
          <tbody id="cleaner-tbody"></tbody>
        </table>
      </div>
      <div class="progress" id="clean-progress-wrap" style="display:none;margin-top:14px;"><div class="progress-bar" id="clean-progress"></div></div>
    </div>`;

  const RECOMMENDED = new Set(['temp', 'system_temp', 'directx_shader', 'nvidia_shader', 'amd_shader', 'recycle_bin']);
  const tbody = el<HTMLElement>('#cleaner-tbody');

  const CATEGORY: Record<string, string> = {
    temp: t('Cleaner_Cat_Temp', 'Temporários'),
    system_temp: t('Cleaner_Cat_Temp', 'Temporários'),
    prefetch: t('Cleaner_Cat_System', 'Sistema'),
    windows_update: t('Cleaner_Cat_System', 'Sistema'),
    directx_shader: t('Cleaner_Cat_Shader', 'Shaders'),
    nvidia_shader: t('Cleaner_Cat_Shader', 'Shaders'),
    amd_shader: t('Cleaner_Cat_Shader', 'Shaders'),
    recycle_bin: t('Cleaner_Cat_User', 'Usuário'),
    dns: t('Cleaner_Cat_Network', 'Rede'),
  };

  const refreshTotals = () => {
    const sel = items.filter((i) => i.is_selected);
    const set = (id: string, v: string) => { const n = el<HTMLElement>(id); if (n) n.textContent = v; };
    set('#sel-count', `${sel.length}/${items.length}`);
    set('#total-selected', fmtBytes(sel.reduce((s, i) => s + i.size_bytes, 0)));
  };

  const renderRows = () => {
    if (!tbody) return;
    tbody.innerHTML = items.map((item, idx) => `
      <tr>
        <td><input type="checkbox" class="cleaner-check" data-idx="${idx}" ${item.is_selected ? 'checked' : ''} /></td>
        <td class="drv-name" title="${item.target_path}">${item.display_name}</td>
        <td><span class="drv-status drv-mute">${CATEGORY[item.id] || '—'}</span></td>
        <td class="mono">${item.id === 'dns' ? '—' : fmtBytes(item.size_bytes)}</td>
        <td class="drv-action"><button class="btn btn-secondary btn-sm btn-clean-one" data-id="${item.id}">${t('Cleaner_Clean_One', 'Limpar')}</button></td>
      </tr>`).join('');

    tbody.querySelectorAll<HTMLInputElement>('.cleaner-check').forEach((c) => {
      c.addEventListener('change', () => {
        items[Number(c.dataset.idx)].is_selected = c.checked;
        refreshTotals();
      });
    });
    tbody.querySelectorAll<HTMLButtonElement>('.btn-clean-one').forEach((b) => {
      b.addEventListener('click', async () => {
        b.disabled = true;
        b.textContent = t('Cleaner_Cleaning', 'Limpando...');
        try {
          const r = await invoke<CleaningSummary>('clean_item', { itemId: b.dataset.id });
          logCompletion(`${t('Cleaner_One_Done', 'Alvo limpo')}: ${fmtBytes(r.total_bytes_cleaned)} / ${r.files_deleted} ${t('Cleaner_Files', 'arquivos')}`);
          const fresh = await renderCleaner;
          await fresh(host);
        } catch (err) { addLog('❌ ' + String(err)); b.disabled = false; b.textContent = t('Cleaner_Clean_One', 'Limpar'); }
      });
    });
    refreshTotals();
  };
  renderRows();

  el<HTMLButtonElement>('#btn-select-all')?.addEventListener('click', () => {
    items.forEach((i) => { i.is_selected = true; });
    renderRows();
  });

  el<HTMLButtonElement>('#btn-select-recommended')?.addEventListener('click', () => {
    items.forEach((i) => { i.is_selected = RECOMMENDED.has(i.id); });
    renderRows();
  });

  el<HTMLButtonElement>('#btn-flush-dns')?.addEventListener('click', async () => {
    try {
      await invoke('flush_dns');
      logCompletion(t('DNS_Flushed', 'Cache DNS limpo'));
    } catch (err) { addLog('❌ ' + String(err)); }
  });

  el<HTMLButtonElement>('#btn-clean')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    const selected = items.filter((i) => i.is_selected && i.id !== 'dns');
    if (selected.length === 0) { addLog('⚠️ ' + t('Cleaner_NoSelection', 'Selecione ao menos um alvo')); return; }
    btn.disabled = true;
    btn.textContent = t('Cleaner_Cleaning', 'Limpando...');
    const wrap = el<HTMLDivElement>('#clean-progress-wrap');
    const bar = el<HTMLDivElement>('#clean-progress');
    if (wrap && bar) { wrap.style.display = ''; bar.style.width = '4%'; }
    let total: CleaningSummary = { total_bytes_cleaned: 0, files_deleted: 0, errors_count: 0 };
    let step = 0;
    try {
      for (const item of selected) {
        addLog(`🧹 ${item.display_name}...`);
        const result = await invoke<CleaningSummary>('clean_item', { itemId: item.id });
        total.total_bytes_cleaned += result.total_bytes_cleaned;
        total.files_deleted += result.files_deleted;
        total.errors_count += result.errors_count;
        step += 1;
        if (bar) bar.style.width = `${Math.round(4 + (step / selected.length) * 92)}%`;
      }
      if (bar) bar.style.width = '100%';
      const skipped = total.errors_count > 0 ? ` · ${total.errors_count} ${t('Cleaner_Skipped', 'arquivos em uso pulados')}` : '';
      logCompletion(`${t('Cleaner_Finished', 'Limpeza concluída')}: ${fmtBytes(total.total_bytes_cleaned)} · ${total.files_deleted} ${t('Cleaner_Files', 'arquivos')}${skipped}`);
      addLog(`✅ ${t('Cleaner_Finished', 'Limpeza concluída')}`);
      setTimeout(async () => { await renderCleaner(host); }, 700);
    } catch (err) {
      addLog('❌ ' + String(err));
      btn.disabled = false;
      btn.textContent = t('Cleaner_Purge_Button', 'Limpar selecionados');
    }
  });
}
