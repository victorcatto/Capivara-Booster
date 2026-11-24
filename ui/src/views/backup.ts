import { invoke } from '../tauri';
import { t } from '../i18n';
import { addLog, el } from '../shell';

interface BackupRecord { id: string; name: string; category: string; original_path: string; backup_path: string; description: string; created_at: string; }

export async function renderBackup(host: HTMLElement): Promise<void> {
  host.innerHTML = `
    <h1>${t('Backup_Title', 'Backup & Restauração')}</h1>
    <p class="subtitle">${t('Backup_Subtitle', 'Pontos de restauração, rollback de registro e backups de configuração.')}</p>

    <div class="card section">
      <h3>🛡️ ${t('Safety_Title', 'Segurança do Sistema')}</h3>
      <div class="actions">
        <button class="btn btn-primary" id="btn-restore-point">${t('Restore_Create_Point', 'Criar Ponto de Restauração')}</button>
        <button class="btn btn-danger" id="btn-revert">${t('Backup_RevertRegistry', 'Reverter Tweaks de Registro')}</button>
      </div>
    </div>

    <div class="section">
      <h3>💾 ${t('Backup_History', 'Backups de Configuração')}</h3>
      <div id="backup-list"></div>
    </div>`;

  // A lista de backups vem do manifest local; comando get_backups exposto quando necessário
  const list = el<HTMLDivElement>('#backup-list');
  if (list) list.innerHTML = `<p class="text-muted">${t('Backup_Empty', 'Nenhum backup criado ainda nesta máquina.')}</p>`;

  el<HTMLButtonElement>('#btn-restore-point')?.addEventListener('click', async () => {
    try {
      await invoke('create_restore_point', { description: 'CapivaraBooster_Manual' });
      addLog('🛡️ ' + t('Restore_Point_Created', 'Ponto de restauração criado'));
    } catch (err) { addLog('❌ ' + String(err)); }
  });

  el<HTMLButtonElement>('#btn-revert')?.addEventListener('click', async () => {
    try {
      const n = await invoke<number>('revert_registry');
      addLog(`↩️ ${n} ${t('Registry_Reverted', 'valores de registro restaurados')}`);
    } catch (err) { addLog('❌ ' + String(err)); }
  });
}
