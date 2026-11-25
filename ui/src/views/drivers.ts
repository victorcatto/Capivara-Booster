import { invoke } from '../tauri';
import { t } from '../i18n';
import { addLog, logCompletion, el } from '../shell';

interface DriverInfo {
  device_name: string;
  driver_version: string;
  driver_date: string;
  provider: string;
  manufacturer: string;
  status: string;
  download_url: string | null;
}

const STATUS: Record<string, { label: string; cls: string }> = {
  updated: { label: 'Atualizado', cls: 'drv-ok' },
  update_available: { label: 'Atualização disponível', cls: 'drv-warn' },
  vendor_managed: { label: 'Gerenciado pelo fabricante', cls: 'drv-mute' },
};

export async function renderDrivers(host: HTMLElement): Promise<void> {
  host.innerHTML = `
    <h1>${t('Drivers_Title', 'Driver Booster')}</h1>
    <p class="subtitle">${t('Drivers_Subtitle', 'Compara a data do driver local com o último do fabricante e atualiza com 1 clique via Windows Update. Backup completo antes de qualquer instalação.')}</p>

    <div class="card section" id="hardware-card" style="display:none;">
      <h3>🖥️ ${t('Drivers_Hardware_Header', 'Configuração do Hardware')}</h3>
      <div id="hardware-details" class="hw-grid"></div>
    </div>

    <div class="card section">
      <div class="drv-toolbar">
        <h3>🧭 ${t('Drivers_Scan_Header', 'Varredura de drivers')}</h3>
        <div class="actions" style="margin:0;">
          <button class="btn btn-primary" id="btn-scan">${t('Drivers_Scan', 'Rastrear drivers')}</button>
          <button class="btn btn-secondary" id="btn-backup">${t('Drivers_Backup', 'Exportar backup')}</button>
        </div>
      </div>
      <p class="hint">${t('Drivers_Backup_Note', 'O backup usa pnputil e guarda todos os drivers atuais antes de qualquer atualização.')}</p>
      <div id="drivers-summary" class="drv-summary" style="display:none;"></div>
      <div id="drivers-table" class="drv-table-wrap" style="display:none;">
        <table class="drv-table">
          <thead><tr>
            <th>${t('Drivers_Col_Device', 'Dispositivo')}</th>
            <th>${t('Drivers_Col_Version', 'Versão instalada')}</th>
            <th>${t('Drivers_Col_Date', 'Data')}</th>
            <th>${t('Drivers_Col_Provider', 'Provedor')}</th>
            <th>${t('Drivers_Col_Status', 'Status')}</th>
            <th></th>
          </tr></thead>
          <tbody id="drivers-tbody"></tbody>
        </table>
      </div>
    </div>

    <div class="card section">
      <h3>🔗 ${t('Drivers_Vendors_Header', 'Suporte oficial do fabricante')}</h3>
      <div id="vendors-list"></div>
    </div>`;

  const hwCard = el<HTMLDivElement>('#hardware-card');
  const hwDetails = el<HTMLDivElement>('#hardware-details');
  if (hwCard && hwDetails) {
    try {
      const hw = await invoke<{ motherboard_vendor: string; motherboard_product: string; bios_version: string; cpu_name: string; gpu_names: string[]; ram_gb: number; os_name: string }>('get_hardware_profile');
      hwCard.style.display = '';
      hwDetails.innerHTML = `
        <div class="hw-item"><span>${t('Drivers_Hw_Board', 'Placa-mãe')}</span><strong>${hw.motherboard_vendor} ${hw.motherboard_product}</strong></div>
        <div class="hw-item"><span>${t('Drivers_Hw_Bios', 'BIOS')}</span><strong>${hw.bios_version || 'N/A'}</strong></div>
        <div class="hw-item"><span>${t('Drivers_Hw_Cpu', 'CPU')}</span><strong>${hw.cpu_name}</strong></div>
        <div class="hw-item"><span>${t('Drivers_Hw_Gpu', 'GPU')}</span><strong>${hw.gpu_names.join(', ') || 'N/A'}</strong></div>
        <div class="hw-item"><span>${t('Drivers_Hw_Ram', 'RAM')}</span><strong>${hw.ram_gb} GB</strong></div>
        <div class="hw-item"><span>${t('Drivers_Hw_Os', 'Sistema')}</span><strong>${hw.os_name}</strong></div>`;
    } catch { /* perfil de hardware indisponível: mantém o card oculto */ }
  }

  const vendors: Array<[string, string]> = await invoke('get_driver_vendors');
  const vendorsList = el<HTMLDivElement>('#vendors-list');
  if (vendorsList) {
    vendorsList.innerHTML = vendors.length === 0
      ? `<p class="text-muted">${t('Drivers_NoVendor', 'Fabricante não identificado: use o site do suporte da sua placa-mãe.')}</p>`
      : vendors.map(([name, url]) => `
        <div class="list-row">
          <div><div class="name">${name}</div><div class="desc text-muted">${t('Drivers_Vendor_Note', 'Driver oficial mais recente direto do fabricante.')}</div></div>
          <a class="btn btn-secondary btn-sm" href="${url}" target="_blank" rel="noopener">${t('Drivers_Open', 'Abrir site')}</a>
        </div>`).join('');
  }

  el<HTMLButtonElement>('#btn-scan')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    addLog('🧭 ' + t('Drivers_Scanning', 'Rastreando drivers e comparando com o fabricante...'));
    try {
      const drivers: DriverInfo[] = await invoke('scan_drivers');
      const updated = drivers.filter((d) => d.status === 'updated').length;
      const pending = drivers.filter((d) => d.status === 'update_available');
      const summary = el<HTMLDivElement>('#drivers-summary');
      if (summary) {
        summary.style.display = '';
        summary.innerHTML = `
          <span class="drv-chip drv-ok">${updated} ${t('Drivers_Updated', 'atualizados')}</span>
          <span class="drv-chip drv-warn">${pending.length} ${t('Drivers_Pending', 'pendentes')}</span>
          <span class="drv-chip drv-mute">${drivers.length} ${t('Drivers_Total', 'no total')}</span>`;
      }
      const tbody = el<HTMLElement>('#drivers-tbody');
      const wrap = el<HTMLDivElement>('#drivers-table');
      if (tbody && wrap) {
        wrap.style.display = '';
        tbody.innerHTML = drivers.slice(0, 120).map((d) => {
          const st = STATUS[d.status] || { label: d.status, cls: 'drv-mute' };
          const action = d.status === 'update_available'
            ? `<button class="btn btn-primary btn-sm btn-drv-update" data-device="${d.device_name.replace(/"/g, '&quot;')}">${t('Drivers_Update_One', 'Atualizar')}</button>`
            : '';
          return `
            <tr>
              <td class="drv-name" title="${d.device_name}">${d.device_name}</td>
              <td class="mono">${d.driver_version}</td>
              <td>${d.driver_date}</td>
              <td>${d.provider || d.manufacturer}</td>
              <td><span class="drv-status ${st.cls}">${st.label}</span></td>
              <td class="drv-action">${action}</td>
            </tr>`;
        }).join('');
        tbody.querySelectorAll<HTMLButtonElement>('.btn-drv-update').forEach((b) => {
          b.addEventListener('click', async () => {
            const device = b.dataset.device || '';
            b.disabled = true;
            b.textContent = t('Drivers_Updating', 'Atualizando...');
            try {
              const msg = await invoke<string>('update_driver_device', { deviceName: device });
              logCompletion(msg);
              b.textContent = t('Drivers_Updated_Done', 'Atualizado');
              b.classList.remove('btn-primary');
            } catch (err) {
              addLog('❌ ' + String(err));
              b.disabled = false;
              b.textContent = t('Drivers_Update_One', 'Atualizar');
            }
          });
        });
      }
      logCompletion(`${t('Drivers_Found', 'Drivers')}: ${drivers.length} (${updated} ${t('Drivers_Updated', 'atualizados')}, ${pending.length} ${t('Drivers_Pending', 'pendentes')})`);
    } catch (err) {
      addLog('❌ ' + String(err));
    } finally {
      btn.disabled = false;
    }
  });

  el<HTMLButtonElement>('#btn-backup')?.addEventListener('click', async () => {
    const dest = `C:\\DriverBackups\\capivara-${new Date().toISOString().slice(0, 10)}`;
    try {
      await invoke('export_driver_backup', { dest });
      logCompletion(`${t('Drivers_Backup_Done', 'Backup exportado para')} ${dest}`);
    } catch (err) {
      addLog('❌ ' + String(err));
    }
  });
}
