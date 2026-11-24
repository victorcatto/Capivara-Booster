// Ponte única para o backend Tauri, com fallback para navegador (dev fora do Tauri).
const hasTauri = typeof (window as any).__TAURI__ !== 'undefined';

export async function invoke<T = unknown>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  if (hasTauri) {
    const { invoke } = (window as any).__TAURI__.core;
    return invoke(cmd, args) as Promise<T>;
  }
  // Mock de desenvolvimento fora do Tauri
  return Promise.resolve(mockInvoke(cmd, args) as T);
}

function mockInvoke(cmd: string, args: Record<string, unknown>): unknown {
  switch (cmd) {
    case 'get_i18n':
    case 'set_language':
      return { language: 'pt-BR', strings: {} };
    case 'get_auth_state':
      return { authenticated: false, is_admin: false, email: null, display_name: null, role: null };
    case 'get_cleaner_items':
      return [];
    case 'get_bloatware_items':
      return [];
    case 'get_logs':
      return { items: ['(modo navegador: sem backend)'] };
    case 'get_startup_items':
      return [];
    case 'check_updates':
      return { available: false };
    case 'detect_game':
      return { game_id: args.game_id, found: false, executable_path: '', install_directory: '' };
    default:
      return null;
  }
}
