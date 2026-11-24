import { invoke } from './tauri';
import { t, setLanguage, currentLanguage } from './i18n';

interface AuthState {
  authenticated: boolean;
  is_admin: boolean;
  email: string | null;
  display_name: string | null;
  role: string | null;
}

const state = {
  auth: null as AuthState | null,
  consoleVisible: false,
};

export function el<T extends HTMLElement>(sel: string): T {
  return document.querySelector(sel) as T;
}

export async function refreshAuth(): Promise<void> {
  state.auth = await invoke<AuthState>('get_auth_state');
  renderAuth();
}

export function renderAuth(): void {
  const authArea = el<HTMLDivElement>('#auth-area');
  if (!authArea) return;
  const a = state.auth;
  if (a?.authenticated) {
    authArea.innerHTML = `
      <span class="user-badge">${a.display_name}</span>
      <span class="role-badge ${a.is_admin ? 'admin' : 'user'}">${a.is_admin ? 'ADMIN' : 'USER'}</span>
      <button class="btn btn-secondary btn-sm" id="btn-logout">${t('Auth_Logout', 'Sair')}</button>`;
    el<HTMLButtonElement>('#btn-logout')?.addEventListener('click', async () => {
      await invoke('logout');
      await refreshAuth();
    });
  } else {
    authArea.innerHTML = `<button class="btn btn-secondary" id="btn-login">👤 ${t('Auth_LoginRegister', 'Login / Cadastro')}</button>`;
    el<HTMLButtonElement>('#btn-login')?.addEventListener('click', openAuthModal);
  }
}

export function openAuthModal(): void {
  const modal = el<HTMLDivElement>('#auth-modal');
  if (!modal) return;
  modal.classList.add('open');
  modal.innerHTML = `
    <div class="modal-card">
      <div class="modal-head">
        <h3>${t('Auth_Modal_Subtitle', 'Acesse sua conta')}</h3>
        <button class="btn-close" id="auth-close">✕</button>
      </div>
      <div class="mode-switch">
        <button id="mode-login" class="active">${t('Auth_Modal_TabLogin', 'Entrar')}</button>
        <button id="mode-register">${t('Auth_Modal_TabSignUp', 'Cadastrar')}</button>
      </div>
      <label>${t('Auth_Modal_EmailLabel', 'E-mail')}</label>
      <input type="email" id="auth-email" placeholder="voce@gmail.com" />
      <small class="hint">${t('Auth_Modal_AllowedDomains', 'Domínios: @gmail.com, @outlook.com, @yahoo.com.br, @yahoo.com')}</small>
      <label>${t('Auth_Modal_PasswordLabel', 'Senha')}</label>
      <input type="password" id="auth-pass" />
      <p class="error" id="auth-error"></p>
      <button class="btn btn-primary btn-block" id="auth-submit">${t('Auth_Modal_LoginBtn', 'Entrar')}</button>
    </div>`;

  let registerMode = false;
  const setMode = (reg: boolean) => {
    registerMode = reg;
    el('#mode-login')?.classList.toggle('active', !reg);
    el('#mode-register')?.classList.toggle('active', reg);
    el<HTMLButtonElement>('#auth-submit')!.textContent = reg ? t('Auth_Modal_RegisterBtn', 'Cadastrar') : t('Auth_Modal_LoginBtn', 'Entrar');
  };
  el('#mode-login')?.addEventListener('click', () => setMode(false));
  el('#mode-register')?.addEventListener('click', () => setMode(true));
  el('#auth-close')?.addEventListener('click', () => modal.classList.remove('open'));
  modal.addEventListener('click', (e) => { if (e.target === modal) modal.classList.remove('open'); });

  el('#auth-submit')?.addEventListener('click', async () => {
    const email = el<HTMLInputElement>('#auth-email')?.value ?? '';
    const pass = el<HTMLInputElement>('#auth-pass')?.value ?? '';
    const err = el<HTMLParagraphElement>('#auth-error');
    try {
      if (registerMode) {
        await invoke('register', { email, password: pass });
      } else {
        await invoke('login', { email, password: pass });
      }
      modal.classList.remove('open');
      await refreshAuth();
      addLog('✅ ' + (currentLanguage() === 'pt-BR' ? 'Autenticado com sucesso' : 'Authenticated successfully'));
    } catch (e: any) {
      err!.textContent = String(e);
    }
  });
}

export function addLog(msg: string): void {
  const list = el<HTMLDivElement>('#console-list');
  if (!list) return;
  const line = document.createElement('div');
  line.className = 'log-line';
  line.textContent = msg;
  list.appendChild(line);
  list.scrollTop = list.scrollHeight;
}

export async function refreshLogs(): Promise<void> {
  const payload = await invoke<{ items: string[] }>('get_logs');
  const list = el<HTMLDivElement>('#console-list');
  if (!list) return;
  list.innerHTML = '';
  payload.items.forEach(addLog);
}

export function setupConsoleToggle(): void {
  const panel = el<HTMLDivElement>('#console-panel');
  el<HTMLButtonElement>('#btn-console-toggle')?.addEventListener('click', () => {
    state.consoleVisible = !state.consoleVisible;
    panel?.classList.toggle('open', state.consoleVisible);
    if (state.consoleVisible) refreshLogs();
  });
  el<HTMLButtonElement>('#btn-console-clear')?.addEventListener('click', async () => {
    await invoke('clear_logs');
    await refreshLogs();
  });
}

export function setupLanguageSwitch(): void {
  el<HTMLButtonElement>('#lang-pt')?.addEventListener('click', () => applyLanguage('pt-BR'));
  el<HTMLButtonElement>('#lang-en')?.addEventListener('click', () => applyLanguage('en-US'));
}

export async function applyLanguage(lang: string): Promise<void> {
  await setLanguage(lang);
  const ptBtn = el<HTMLButtonElement>('#lang-pt');
  const enBtn = el<HTMLButtonElement>('#lang-en');
  ptBtn?.classList.toggle('active', lang === 'pt-BR');
  enBtn?.classList.toggle('active', lang === 'en-US');
  window.dispatchEvent(new CustomEvent('language-changed'));
}

export { state };
