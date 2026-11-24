import { invoke } from './tauri';

type Strings = Record<string, string>;

let current: Strings = {};
let lang = 'pt-BR';

export async function loadI18n(): Promise<void> {
  const payload = await invoke<{ language: string; strings: Strings }>('get_i18n');
  current = payload.strings;
  lang = payload.language;
  document.documentElement.lang = lang;
}

export async function setLanguage(next: string): Promise<void> {
  const payload = await invoke<{ language: string; strings: Strings }>('set_language', { lang: next });
  current = payload.strings;
  lang = payload.language;
  document.documentElement.lang = lang;
  document.querySelectorAll('[data-i18n]').forEach((el) => {
    const key = (el as HTMLElement).dataset.i18n as string;
    el.textContent = t(key);
  });
}

export function t(key: string, fallback?: string): string {
  return current[key] ?? fallback ?? `[${key}]`;
}

export function currentLanguage(): string {
  return lang;
}
