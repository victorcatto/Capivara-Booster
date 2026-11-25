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

const STATUS_LABEL: Record<string, { label: string; cls: string }> = {
  updated: { label: 'Updated', cls: 'text-success' },
  update_available: { label: 'Update available', cls: 'text-warning' },
  vendor_managed: { label: 'Vendor managed', cls: 'text-muted' },
};

export async function renderDrivers(host: HTMLElement): Promise<void> {
  host.innerHTML = `
    <h1>${t('Drivers_Title', 'Driver Booster')}</h1>
    <p class="subtitle">${t('Drivers_Subtitle', 'Compara a data do driver local com o último do fabricante e aponta o download oficial. Backup completo antes de qualquer instalação.')}</p>

    <div class="card section">
      <h3>🧭 ${t('Drivers_Scan_Header', 'Varredura de drivers')}</h3>
      <div class="actions">
        <button class="btn btn-primary" id="btn-scan">${t('Drivers_Scan', 'Rastrear drivers')}</button>
        <button class="btn btn-secondary" id="btn-backup">${t('Drivers_Backup', 'Exportar backup dos drivers')}</button>
      </div>
      <p class="hint">${t('Drivers_Backup_Note', 'O backup usa pnputil e guarda todos os drivers atuais na pasta escolhida antes de qualquer atualização.')}</p>
      <div id="drivers-summary" class="section" style="padding:8px 0;"></div>
      <div id="drivers-list" class="section" style="padding:0;"></div>
    </div>

    <div class="card section">
      <h3>🔗 ${t('Drivers_Vendors_Header', 'Downloads oficiais do fabricante')}</h3>
      <div id="vendors-list"></div>
    </div>`;

  const vendors: Array<[string, string]> = await invoke('get_driver_vendors');
  const vendorsList = el<HTMLDivElement>('#vendors-list');
  if (vendorsList) {
    vendorsList.innerHTML = vendors.length === 0
      ? `<p class="text-muted">${t('Drivers_NoVendor', 'Fabricante não identificado: use o site do suporte da sua placa-mãe.')}</p>`
      : vendors.map(([name, url]) => `
        <div class="list-row">
          <div><div class="name">${name}</div><div class="desc text-muted">${t('Drivers_Vendor_Note', 'Driver oficial mais recente direto do fabricante.')}</div></div>
          <a class="btn btn-primary btn-sm" href="${url}" target="_blank" rel="noopener">${t('Drivers_Open', 'Abrir site')}</a>
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
        summary.innerHTML = `<strong class="text-success">${updated} ${t('Drivers_Updated', 'atualizados')}</strong> · <strong class="text-warning">${pending.length} ${t('Drivers_Pending', 'com atualização no fabricante')}</strong> · ${drivers.length} ${t('Drivers_Total', 'no total')}`;
      }
      const list = el<HTMLDivElement>('#drivers-list');
      if (list) {
        list.innerHTML = drivers.length === 0
          ? `<p class="text-muted">${t('Drivers_Empty', 'Nenhum driver listado.')}</p>`
          : drivers.slice(0, 100).map((d) => {
              const st = STATUS_LABEL[d.status] || { label: d.status, cls: 'text-muted' };
              const action = d.download_url
                ? `<a class="btn btn-primary btn-sm" href="${d.download_url}" target="_blank" rel="noopener">${t('Drivers_Download', 'Baixar')}</a>`
                : '';
              return `
                <div class="list-row">
                  <div>
                    <div class="name">${d.device_name}</div>
                    <div class="desc text-muted">${t('Drivers_Version', 'Versão')} ${d.driver_version} · ${d.driver_date} · ${d.provider}</div>
                  </div>
                  <strong class="${st.cls}" style="font-size:12px;">${st.label}</strong>
                  ${action}
                </div>`;
            }).join('');
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
