import './styles/theme.css';
import './styles/controls.css';
import { loadI18n, t } from './i18n';
import { invoke } from './tauri';
import { refreshAuth, renderAuth, setupConsoleToggle, setupLanguageSwitch, applyLanguage, addLog, refreshLogs, el } from './shell';
import { renderDashboard } from './views/dashboard';
import { renderDebloat } from './views/debloat';
import { renderCleaner } from './views/cleaner';
import { renderBoost } from './views/boost';
import { renderLatency } from './views/latency';
import { renderGames } from './views/games';
import { renderPoe } from './views/poe';
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
  backup: renderBackup,
};

let currentView = 'dashboard';

async function navigate(view: string): Promise<void> {
  currentView = view;
  document.querySelectorAll('.nav-item').forEach((n) => n.classList.toggle('active', (n as HTMLElement).dataset.view === view));
  const host = el<HTMLElement>('#view-host');
  if (host) await views[view]?.(host);
}

async function checkUpdates(silent: boolean): Promise<void> {
  try {
    const result = await invoke<{ available: boolean; version: string | null }>('check_updates');
    if (result.available && result.version) {
      addLog(`🚀 ${t('Update_Available', 'Nova versão disponível')}: v${result.version}`);
    } else if (!silent) {
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

  window.addEventListener('language-changed', () => {
    navigate(currentView);
  });

  await navigate('dashboard');
  refreshLogs();
  checkUpdates(true);
}

boot();
