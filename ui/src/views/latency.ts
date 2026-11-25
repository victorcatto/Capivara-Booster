import { invoke } from '../tauri';
import { t } from '../i18n';
import { addLog, logCompletion, el } from '../shell';

interface LatencyTweak {
  id: string;
  title: string;
  description: string;
  tooltip: string;
  requires_reboot: boolean;
  is_applied: boolean | null;
}

export async function renderLatency(host: HTMLElement): Promise<void> {
  const tweaks: LatencyTweak[] = await invoke('get_latency_tweaks');

  host.innerHTML = `
    <h1>${t('Latency_Title', 'Motor de Baixa Latência')}</h1>
    <p class="subtitle">${t('Latency_Subtitle', 'Timers de kernel, MMCSS, USB, rede, GPU e RAM. Cada tweak é reversível e documentado no tooltip.')}</p>

    <div class="card section">
      <h3>⚡ ${t('Latency_Profile_Title', 'Perfil Recomendado de Baixa Latência')}</h3>
      <p class="subtitle">${t('Latency_Profile_Desc', 'Aplica todos os tweaks seguros de uma vez, com ponto de restauração automático.')}</p>
      <div class="actions">
        <button class="btn btn-primary" id="btn-latency-apply-all">${t('Latency_Apply_All', 'Aplicar Perfil Recomendado')}</button>
        <button class="btn btn-secondary" id="btn-latency-revert-all">${t('Latency_Revert_All', 'Reverter Tudo')}</button>
        <button class="btn btn-secondary" id="btn-standby">${t('Latency_Standby', 'Limpar RAM em Standby')}</button>
      </div>
    </div>

    <div class="section">
      <h3>🎛️ ${t('Latency_Tweaks_Header', 'Tweaks Individuais')}</h3>
      <div id="latency-list"></div>
    </div>`;

  const list = el<HTMLDivElement>('#latency-list');
  const renderRows = () => {
    if (!list) return;
    list.innerHTML = tweaks.map((tweak, idx) => `
      <div class="list-row">
        <span title="${tweak.tooltip.replace(/"/g, '&quot;')}" class="tweak-info">ℹ️</span>
        <div>
          <div class="name">${tweak.title} ${tweak.requires_reboot ? `<span class="badge badge-reboot">${t('Badge_RequiresReboot', 'Requer reinício')}</span>` : ''}</div>
          <div class="desc">${tweak.description}</div>
          <div class="desc text-muted">${tweak.tooltip}</div>
        </div>
        <div class="actions" style="margin:0;">
          <button class="btn btn-primary btn-sm" data-apply="${idx}">${t('Latency_Apply', 'Aplicar')}</button>
          <button class="btn btn-secondary btn-sm" data-revert="${idx}">${t('Latency_Revert', 'Reverter')}</button>
        </div>
      </div>`).join('');

    list.querySelectorAll<HTMLButtonElement>('[data-apply]').forEach((button) => {
      button.addEventListener('click', async () => {
        const tweak = tweaks[Number(button.dataset.apply)];
        button.disabled = true;
        try {
          await invoke('apply_latency_tweak', { tweakId: tweak.id });
          addLog(`⚡ ${tweak.title}: aplicado`);
          logCompletion(`${tweak.title} ${t('Latency_Applied_Suffix', 'aplicado com sucesso')}`);
        } catch (err) {
          addLog(`❌ ${String(err)}`);
        } finally {
          button.disabled = false;
        }
      });
    });

    list.querySelectorAll<HTMLButtonElement>('[data-revert]').forEach((button) => {
      button.addEventListener('click', async () => {
        const tweak = tweaks[Number(button.dataset.revert)];
        button.disabled = true;
        try {
          await invoke('revert_latency_tweak', { tweakId: tweak.id });
          addLog(`↩️ ${tweak.title}: revertido`);
          logCompletion(`${tweak.title} ${t('Latency_Reverted_Suffix', 'revertido ao padrão')}`);
        } catch (err) {
          addLog(`❌ ${String(err)}`);
        } finally {
          button.disabled = false;
        }
      });
    });
  };
  renderRows();

  el<HTMLButtonElement>('#btn-latency-apply-all')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    try {
      for (const tweak of tweaks) {
        await invoke('apply_latency_tweak', { tweakId: tweak.id });
        addLog(`⚡ ${tweak.title}: aplicado`);
      }
      logCompletion(t('Latency_Profile_Applied', 'Perfil de baixa latência aplicado'));
    } catch (err) {
      addLog(`❌ ${String(err)}`);
    } finally {
      btn.disabled = false;
    }
  });

  el<HTMLButtonElement>('#btn-latency-revert-all')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    btn.disabled = true;
    try {
      for (const tweak of tweaks) {
        await invoke('revert_latency_tweak', { tweakId: tweak.id });
        addLog(`↩️ ${tweak.title}: revertido`);
      }
      logCompletion(t('Latency_Profile_Reverted', 'Todos os tweaks revertidos ao padrão'));
    } catch (err) {
      addLog(`❌ ${String(err)}`);
    } finally {
      btn.disabled = false;
    }
  });

  el<HTMLButtonElement>('#btn-standby')?.addEventListener('click', async () => {
    try {
      await invoke('clear_standby_list');
      logCompletion(t('Latency_Standby_Cleared', 'RAM em standby liberada'));
    } catch (err) {
      addLog(`❌ ${String(err)}`);
    }
  });
}
