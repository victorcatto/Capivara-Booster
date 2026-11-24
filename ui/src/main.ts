import './styles/theme.css';
import './styles/controls.css';

const app = document.getElementById('app');

if (app) {
  app.innerHTML = `
    <aside class="sidebar">
      <div style="padding: 0 0 18px 0; display:flex; align-items:center; gap:12px;">
        <div style="width:42px;height:42px;border-radius:10px;background:var(--gradient);"></div>
        <div>
          <div style="font-weight:700;font-size:14.5px;color:var(--accent-cyan);">CAPIVARA BOOSTER</div>
          <div style="font-size:9.5px;color:var(--accent-purple);">v2.0.0 Rust</div>
        </div>
      </div>
      <div class="nav-item active">⚡ Dashboard</div>
      <div class="nav-item">🧹 Debloat</div>
      <div class="nav-item">🧼 Cleaner</div>
      <div class="nav-item">🚀 System Boost</div>
      <div class="nav-item">🎮 Game Optimizer</div>
      <div class="nav-item">🛡️ Backup & Restore</div>
    </aside>
    <main style="padding:30px;">
      <h1 style="font-size:24px;font-weight:700;">Visão Geral do Sistema</h1>
      <p style="color:var(--text-secondary);font-size:13px;margin-top:4px;">
        Backend Rust + Tauri v2 ativo. UI em migração.
      </p>
    </main>
  `;
}