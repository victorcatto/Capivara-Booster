import { invoke } from '../tauri';
import { t } from '../i18n';
import { addLog, el } from '../shell';

interface GameDetection { game_id: string; found: boolean; executable_path: string; install_directory: string; }

const GAMES: Array<{ id: string; name: string; icon: string }> = [
  { id: 'cs2', name: 'Counter-Strike 2', icon: '🎯' },
  { id: 'valorant', name: 'VALORANT', icon: '🏹' },
  { id: 'lol', name: 'League of Legends', icon: '⚔️' },
];

export async function renderGames(host: HTMLElement): Promise<void> {
  host.innerHTML = `
    <h1>${t('Games_Title', 'Otimização para Jogos Competitivos')}</h1>
    <p class="subtitle">${t('Games_Subtitle', 'Perfis competitivos para CS2, Valorant e LoL.')}</p>
    <div id="games-list"></div>`;

  const list = el<HTMLDivElement>('#games-list');
  if (!list) return;

  for (const game of GAMES) {
    const detection = await invoke<GameDetection>('detect_game', { gameId: game.id });
    const card = document.createElement('div');
    card.className = 'card section game-card';
    card.innerHTML = `
      <div>
        <h3>${game.icon} ${game.name} ${detection.found ? '<span class="text-success" style="font-size:12px;">✔ Detectado</span>' : '<span class="text-danger" style="font-size:12px;">✘ Não encontrado</span>'}</h3>
        <div class="desc text-muted mono">${detection.executable_path || t('Game_NotFound', 'Instale o jogo ou verifique a instalação.')}</div>
      </div>
      <div class="actions">
        <button class="btn btn-primary" data-game="${game.id}" ${detection.found ? '' : 'disabled'}>${t('Game_Optimize', 'Otimizar')}</button>
      </div>`;
    card.querySelector('button')?.addEventListener('click', async (e) => {
      const btn = e.currentTarget as HTMLButtonElement;
      btn.disabled = true;
      try {
        addLog(`🎮 ${game.name}: ${t('Game_Optimizing', 'aplicando perfil competitivo...')}`);
        await invoke('optimize_game', { gameId: game.id, exePath: detection.executable_path });
        addLog(`✅ ${game.name} ${t('Game_Optimized', 'otimizado')}`);
      } catch (err) {
        addLog('❌ ' + String(err));
      } finally {
        btn.disabled = false;
      }
    });
    list.appendChild(card);
  }
}
