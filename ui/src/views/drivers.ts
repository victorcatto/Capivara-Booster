import { invoke } from '../tauri';
import { t } from '../i18n';
import { addLog, logCompletion, el } from '../shell';

interface DriverInfo {
  device_name: string;
  driver_version: string;
  driver_date: string;
  provider: string;
  manufacturer: string;
}
interface DriverUpdate { device_name: string; download_url: string; vendor: string; note: string; }

export async function renderDrivers(host: HTMLElement): Promise<void> {
  host.innerHTML = `
    <h1>${t('Drivers_Title', 'Driver Booster')}</h1>
    <p class="subtitle">${t('Drivers_Subtitle', 'Rastreia os drivers do PC e aponta o download oficial do fabricante da placa-mãe e da GPU. Backup completo antes de qualquer instalação.')}</p>

    <div class="card section">
      <h3>🧭 ${t('Drivers_Scan_Header', 'Varredura de drivers')}</h3>
      <div class="actions">
        <button class="btn btn-primary" id="btn-scan">${t('Drivers_Scan', 'Rastrear drivers')}</button>
        <button class="btn btn-secondary" id="btn-backup">${t('Drivers_Backup', 'Exportar backup dos drivers')}</button>
      </div>
      <p class="hint">${t('Drivers_Backup_Note', 'O backup usa pnputil e guarda todos os drivers atuais na pasta escolhida.')}</p>
      <div id="drivers-list" class="section" style="padding:0;"></div>
    </div>

    <div class="card section">
      <h3>🔗 ${t('Drivers_Vendors_Header', 'Downloads oficiais do fabricante')}</h3>
      <div id="vendors-list"></div>
    </div>`;

  const vendors: DriverUpdate[] = await invoke('get_driver_vendors');
  const vendorsList = el<HTMLDivElement>('#vendors-list');
  if (vendorsList) {
    vendorsList.innerHTML = vendors.length === 0
      ? `<p class="text-muted">${t('Drivers_NoVendor', 'Fabricante não identificado: use o site do suporte da sua placa-mãe.')}</p>`
      : vendors.map((v) => `
        <div class="list-row">
          <div><div class="name">${v.vendor}</div><div class="desc text-muted">${v.note}</div></div>
          <a class="btn btn-primary btn-sm" href="${v.download_url}" target="_blank" rel="noopener">${t('Drivers_Open', 'Abrir site')}</a>
        </div>`).join('');
  }

  el<HTMLButtonElement>('#btn-scan')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    addLog('🧭 ' + t('Drivers_Scanning', 'Rastreando drivers...'));
    try {
      const drivers: DriverInfo[] = await invoke('scan_drivers');
      const list = el<HTMLDivElement>('#drivers-list');
      if (list) {
        list.innerHTML = drivers.length === 0
          ? `<p class="text-muted">${t('Drivers_Empty', 'Nenhum driver listado.')}</p>`
          : `<div class="list-row list-row-head"><div class="name">${t('Drivers_Device', 'Dispositivo')}</div><div class="name">${t('Drivers_Version', 'Versão')}</div><div class="name">${t('Drivers_Provider', 'Provedor')}</div></div>` +
            drivers.slice(0, 80).map((d) => `
              <div class="list-row">
                <div class="name">${d.device_name}</div>
                <div class="desc mono">${d.driver_version}</div>
                <div class="desc">${d.provider}</div>
              </div>`).join('');
      }
      logCompletion(`${t('Drivers_Found', 'Drivers encontrados')}: ${drivers.length}`);
    } catch (err) {
      addLog('❌ ' + String(err));
    } finally {
      btn.disabled = false;
    }
  });

  el<HTMLButtonElement>('#btn-backup')?.addEventListener('click', async () => {
    const dest = `${(navigator.userAgent.includes('Windows') ? 'C:' : '~')}\\DriverBackups\\capivara-${new Date().toISOString().slice(0, 10)}`;
    try {
      await invoke('export_driver_backup', { dest });
      logCompletion(`${t('Drivers_Backup_Done', 'Backup exportado para')} ${dest}`);
    } catch (err) {
      addLog('❌ ' + String(err));
    }
  });
}
