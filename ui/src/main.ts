import './styles/theme.css';
import './styles/controls.css';
import { loadI18n, t } from './i18n';
import { invoke } from './tauri';
import { refreshAuth, renderAuth, setupConsoleToggle, setupLanguageSwitch, addLog, refreshLogs, el } from './shell';
import { renderDashboard } from './views/dashboard';
import { renderDebloat } from './views/debloat';
import { renderCleaner } from './views/cleaner';
import { renderBoost } from './views/boost';
import { renderLatency } from './views/latency';
import { renderGames } from './views/games';
import { renderPoe } from './views/poe';
import { renderDrivers } from './views/drivers';
import { renderBackup } from './views/backup';

type ViewRenderer = (host: HTMLElement) => Promise<void> | void;

const views: Record<string, ViewRenderer> = {
  dashboard: renderDashboard,
  debloat: renderDebloat,
  cleaner: renderCleaner,
  boost: renderBoost,
  latency: renderLatency,
  games: renderGames,
  poe: renderPoe,
  drivers: renderDrivers,
  backup: renderBackup,
};

let currentView = 'dashboard';
let pendingUpdate: { available: boolean; version: string | null; installer_url: string | null; sha256: string | null; changelog: string | null } | null = null;

async function navigate(view: string): Promise<void> {
  currentView = view;
  document.querySelectorAll('.nav-item').forEach((n) => n.classList.toggle('active', (n as HTMLElement).dataset.view === view));
  const host = el<HTMLElement>('#view-host');
  if (host) await views[view]?.(host);
  renderUpdateBanner();
}

export async function applyPendingUpdate(): Promise<void> {
  if (!pendingUpdate?.version || !pendingUpdate.installer_url || !pendingUpdate.sha256) return;
  const btn = el<HTMLButtonElement>('#btn-apply-update');
  if (btn) { btn.disabled = true; btn.textContent = t('Update_Downloading', 'Baixando atualização...'); }
  try {
    await invoke('apply_update', { version: pendingUpdate.version, installerUrl: pendingUpdate.installer_url, sha256: pendingUpdate.sha256 });
    addLog('🚀 ' + t('Update_Running', 'Instalador iniciado: conclua a instalação e o app reabrirá atualizado'));
    if (btn) btn.textContent = t('Update_Running', 'Instalador iniciado');
  } catch (err) {
    addLog('❌ ' + String(err));
    if (btn) { btn.disabled = false; btn.textContent = t('Update_Install', 'Atualizar agora'); }
  }
}

function renderUpdateBanner(): void {
  // Popup discreto no canto: só aparece quando há versão nova; fecha com ×.
  const existing = el<HTMLDivElement>('#update-popup');
  if (existing) existing.remove();
  if (!pendingUpdate?.available || !pendingUpdate.version) return;
  if (sessionStorage.getItem('update-dismissed') === pendingUpdate.version) return;
  const popup = document.createElement('div');
  popup.id = 'update-popup';
  popup.className = 'update-popup';
  popup.innerHTML = `
    <div class="update-popup-head">
      <strong>🚀 ${t('Update_Available', 'Nova versão disponível')}: v${pendingUpdate.version}</strong>
      <button class="update-popup-close" id="btn-update-close" title="Fechar">×</button>
    </div>
    <p class="update-popup-notes">${(pendingUpdate.changelog || '').slice(0, 320)}</p>
    <div class="update-popup-actions">
      <button class="btn btn-secondary btn-sm" id="btn-update-later">${t('Update_Later', 'Depois')}</button>
      <button class="btn btn-primary btn-sm" id="btn-apply-update">${t('Update_Install', 'Atualizar agora')}</button>
    </div>`;
  document.body.appendChild(popup);
  el<HTMLButtonElement>('#btn-update-close')?.addEventListener('click', () => { sessionStorage.setItem('update-dismissed', pendingUpdate!.version || ''); popup.remove(); });
  el<HTMLButtonElement>('#btn-update-later')?.addEventListener('click', () => { sessionStorage.setItem('update-dismissed', pendingUpdate!.version || ''); popup.remove(); });
  el<HTMLButtonElement>('#btn-apply-update')?.addEventListener('click', () => applyPendingUpdate());
}

async function checkUpdates(silent: boolean): Promise<void> {
  try {
    const result = await invoke<{ available: boolean; version: string | null; installer_url: string | null; sha256: string | null; changelog: string | null }>('check_updates');
    if (result.available && result.version) {
      pendingUpdate = result;
      addLog(`🚀 ${t('Update_Available', 'Nova versão disponível')}: v${result.version}`);
      renderUpdateBanner();
    } else if (!silent) {
      pendingUpdate = null;
      renderUpdateBanner();
      addLog('✅ ' + t('Up_To_Date', 'Aplicativo atualizado'));
    }
  } catch (e) {
    if (!silent) addLog('⚠️ ' + String(e));
  }
}

async function boot(): Promise<void> {
  await loadI18n();
  document.querySelectorAll('[data-i18n]').forEach((elx) => {
    const key = (elx as HTMLElement).dataset.i18n as string;
    elx.textContent = t(key);
  });

  setupConsoleToggle();
  setupLanguageSwitch();
  await refreshAuth();
  renderAuth();

  document.querySelectorAll('.nav-item').forEach((n) => {
    n.addEventListener('click', () => navigate((n as HTMLElement).dataset.view as string));
  });

  el<HTMLButtonElement>('#btn-check-updates')?.addEventListener('click', () => checkUpdates(false));

  // Versão do app no sidebar: comando Rust dedicado (getVersion do plugin pode
  // não estar exposto em com build sem plugin-app; o comando é à prova de falha).
  try {
    const version = await invoke<string>('get_app_version');
    const badge = document.querySelector('[data-app-version]');
    if (badge && version) badge.textContent = `v${version} Rust Edition`;
  } catch { /* fallback: mantém o texto do HTML */ }

  window.addEventListener('language-changed', () => {
    document.querySelectorAll('[data-i18n]').forEach((elx) => {
      const key = (elx as HTMLElement).dataset.i18n as string;
      elx.textContent = t(key);
    });
    renderUpdateBanner();
    navigate(currentView);
  });

  await navigate('dashboard');
  refreshLogs();
  checkUpdates(true);
}

boot();
