const REPO = 'victorcatto/Capivara-Booster';
const INSTALLER = 'CapivaraBooster-Setup.exe';
const LATEST_URL = `https://github.com/${REPO}/releases/latest/download/${INSTALLER}`;
const RELEASES_URL = `https://github.com/${REPO}/releases/latest`;

const I18N = {
  'en-US': {
    nav_features: 'Features', nav_how: 'How it works', nav_notes: 'Release notes', nav_faq: 'FAQ', nav_download: 'Download free',
    hero_title: 'Maximum chill, minimum latency.',
    hero_sub: 'Free performance optimization tool for competitive gamers in CS2, LoL, and Path of Exile.',
    hero_download: 'DOWNLOAD FOR FREE', hero_source: 'View source',
    hero_note: 'Official installer for Windows 10/11 (64-bit). If SmartScreen warns you, click "More info" and "Run anyway".',
    features_title: 'What Capivara Booster does',
    features_sub: 'Low-latency engine, deep cleanup and game profiles, all reversible.',
    f1_title: 'Low-latency engine',
    f1_desc: 'Dynamic Tick, synthetic timers, MMCSS, Win32PrioritySeparation, mouse and keyboard queues, HAGS, PCIe ASPM and more: 11 reversible tweaks in one click.',
    f2_desc: 'Competitive autoexec.cfg, IFEO priority and affinity, Defender exclusions, shader cache cleanup and live process boost.',
    f3_desc: 'Riot Client low priority, Vanguard Defender exclusions and log/webcache cleanup.',
    f4_desc: 'Low Spec Mode, Close Client, VSync off, automatic game.cfg backup and Defender exclusions.',
    f5_title: 'Debloat and deep cleanup',
    f5_desc: '15 safe UWP packages, telemetry and DiagTrack disabled, Temp, Prefetch, Windows Update, DNS and DirectX/NVIDIA/AMD shader caches.',
    f7_title: 'Path of Exile (1 & 2)',
    f7_desc: 'GGG engine profile: global illumination off (30-75% FPS gain in dense maps), low sound channels, lockstep toggle, GPU preference, Nagle off and safe shader cache cleanup.',
    f6_title: 'Safety and rollback',
    f6_desc: 'Restore point before everything, every changed registry key tracked, 1-click rollback and config backups.',
    how_title: 'How it works',
    how1_title: 'Download and install', how1_desc: 'Click the download button and run the official installer. No dependencies: a single Rust executable.',
    how2_title: 'Create account and pick a profile', how2_desc: 'Login is required. Create the restore point, pick your game profile or apply the recommended latency profile.',
    how3_title: 'Play with lower latency', how3_desc: 'Follow everything on the live console. Revert any tweak in one click if needed.',
    notes_title: 'Latest updates', notes_sub: 'The two most recent releases, straight from the official repository.', notes_loading: 'Loading patch notes…',
    notes_latest: 'Latest', notes_previous: 'Previous release', notes_fallback: 'See the release page for the full changelog.',
    q1: 'Is it safe?', a1: 'Yes. Every optimization creates a restore point and can be reverted in one click. The code is open on GitHub.',
    q2: 'Is it free?', a2: 'Yes. Capivara Booster is free and open source (MIT).',
    q3: 'Does it work on my Windows?', a3: 'Windows 10 and 11, 64-bit, with WebView2 (preinstalled on updated Windows).',
    q4: 'Why does SmartScreen warn me?', a4: 'Because the open-source installer has no paid code signature. Check the SHA-256 on the release page and run it anyway.',
    q5: 'Do I need an account?', a5: 'Yes, login is required. Registration is restricted to Gmail, Outlook, Yahoo and Yahoo Brasil.',
    cta_title: 'Ready to lower your latency?', cta_download: 'Download Capivara Booster',
    notes_get: 'Get installer', changelog_title: 'Changelog',
  },
  'pt-BR': {
    nav_features: 'Recursos', nav_how: 'Como funciona', nav_notes: 'Novidades', nav_faq: 'FAQ', nav_download: 'Baixar grátis',
    hero_title: 'Frieza máxima, latência mínima.',
    hero_sub: 'Ferramenta gratuita de otimização de desempenho para jogadores competitivos de CS2, LoL e Path of Exile.',
    hero_download: 'BAIXAR GRATUITAMENTE', hero_source: 'Ver código-fonte',
    hero_note: 'Instalador oficial para Windows 10/11 (64 bits). Se o SmartScreen avisar, clique em "Mais informações" e "Executar assim mesmo".',
    features_title: 'O que o Capivara Booster faz',
    features_sub: 'Motor de baixa latência, limpeza profunda e perfis de jogo, tudo reversível.',
    f1_title: 'Motor de baixa latência',
    f1_desc: 'Dynamic Tick, timers sintéticos, MMCSS, Win32PrioritySeparation, filas de mouse e teclado, HAGS, PCIe ASPM e mais: 11 tweaks reversíveis com um clique.',
    f2_desc: 'autoexec.cfg competitivo, prioridade e afinidade por IFEO, exclusões no Defender, limpeza do shader cache e boost do processo ao vivo.',
    f3_desc: 'Riot Client em prioridade baixa, exclusões do Vanguard no Defender e limpeza de logs e webcache.',
    f4_desc: 'Low Spec Mode, Close Client, VSync off, backup automático do game.cfg e exclusões no Defender.',
    f5_title: 'Debloat e limpeza profunda',
    f5_desc: '15 pacotes UWP de remoção segura, telemetria e DiagTrack desativados, caches de Temp, Prefetch, Windows Update, DNS e shaders DirectX/NVIDIA/AMD.',
    f7_title: 'Path of Exile (1 & 2)',
    f7_desc: 'Perfil da engine GGG: iluminação global desativada (ganho de 30 a 75% de FPS em mapas densos), canais de som baixos, alternador Lockstep/Predictive, GPU de alto desempenho, Nagle off e limpeza segura de shader cache.',
    f6_title: 'Segurança e rollback',
    f6_desc: 'Ponto de restauração antes de tudo, rastreamento de cada chave alterada no Registro, rollback em 1 clique e backups de configuração.',
    how_title: 'Como funciona',
    how1_title: 'Baixe e instale', how1_desc: 'Clique no botão de download e execute o instalador oficial. Sem dependências: um único executável em Rust.',
    how2_title: 'Crie conta e escolha o perfil', how2_desc: 'O login é obrigatório. Crie o ponto de restauração, escolha o perfil de jogo ou aplique o perfil recomendado de latência.',
    how3_title: 'Jogue com menos latência', how3_desc: 'Acompanhe tudo no console ao vivo. Reverta qualquer ajuste em um clique se precisar.',
    notes_title: 'Últimas novidades', notes_sub: 'As duas versões mais recentes, direto do repositório oficial.', notes_loading: 'Carregando patch notes…',
    notes_latest: 'Mais recente', notes_previous: 'Versão anterior', notes_fallback: 'Veja a página da release para o changelog completo.',
    q1: 'É seguro?', a1: 'Sim. Cada otimização cria ponto de restauração e pode ser revertida em um clique. O código é aberto no GitHub.',
    q2: 'Preciso pagar?', a2: 'Não. O Capivara Booster é gratuito e open source (MIT).',
    q3: 'Funciona no meu Windows?', a3: 'Windows 10 e 11, 64 bits, com WebView2 (pré-instalado no Windows atualizado).',
    q4: 'Por que o SmartScreen avisa?', a4: 'Porque o instalador open source não tem assinatura de código paga. Confira o SHA-256 na página da release e execute mesmo assim.',
    q5: 'Preciso de conta?', a5: 'Sim, o login é obrigatório. Cadastro restrito a Gmail, Outlook, Yahoo e Yahoo Brasil.',
    cta_title: 'Pronto para reduzir sua latência?', cta_download: 'Baixar Capivara Booster',
    notes_get: 'Pegar instalador', changelog_title: 'Changelog',
  },
};

let lang = 'pt-BR';

function applyLang(next) {
  lang = next;
  document.documentElement.lang = next;
  const dict = I18N[next];
  document.querySelectorAll('[data-i18n]').forEach((el) => {
    const key = el.dataset.i18n;
    if (dict[key]) el.textContent = dict[key];
  });
  document.getElementById('lang-pt').classList.toggle('active', next === 'pt-BR');
  document.getElementById('lang-en').classList.toggle('active', next === 'en-US');
  localStorage.setItem('capivara-lang', next);
  renderNotes(lastReleases);
}

document.getElementById('lang-pt').addEventListener('click', () => applyLang('pt-BR'));
document.getElementById('lang-en').addEventListener('click', () => applyLang('en-US'));

['cta-nav', 'cta-hero', 'cta-bottom'].forEach((id) => {
  const el = document.getElementById(id);
  if (el) el.href = LATEST_URL;
});

let lastReleases = [];

async function loadReleases() {
  try {
    const local = await fetch('releases-data.json', { cache: 'no-store' });
    if (local.ok) {
      const data = await local.json();
      const arr = Array.isArray(data) ? data : [data];
      if (arr.length > 0) {
        lastReleases = arr.map((rel) => ({
          tag_name: rel.tag,
          name: rel.name,
          published_at: rel.date,
          html_url: rel.url,
          body: (rel.notes || []).join('\n'),
          assets: [{ name: INSTALLER, browser_download_url: rel.installer }],
        }));
      }
    }
  } catch (err) {
    // JSON local ausente: cai para a API do GitHub
  }
  if (lastReleases.length === 0) {
    try {
      const res = await fetch(`https://api.github.com/repos/${REPO}/releases?per_page=2`);
      if (res.ok) lastReleases = await res.json();
    } catch (err) {
      lastReleases = [];
    }
  }
  const heroVersion = document.getElementById('hero-version');
  const heroDate = document.getElementById('hero-date');
  if (lastReleases.length > 0) {
    const latest = lastReleases[0];
    heroVersion.textContent = latest.tag_name;
    heroDate.textContent = new Date(latest.published_at).toLocaleDateString(lang === 'pt-BR' ? 'pt-BR' : 'en-US');
  } else {
    heroVersion.textContent = 'v2.0.0';
    heroDate.textContent = '—';
  }
  renderNotes(lastReleases);
}

function renderNotes(releases) {
  const host = document.getElementById('notes-list');
  if (!host) return;
  if (releases.length === 0) {
    host.innerHTML = `<div class="note"><div class="note-head"><span class="note-version">v2.0.0</span><span class="note-date">2026-11-25</span><a class="note-download" href="${RELEASES_URL}" rel="noopener">${I18N[lang].notes_get} →</a></div><ul><li>${I18N[lang].a1}</li></ul></div>`;
    return;
  }
  host.innerHTML = releases.map((rel, index) => {
    const date = new Date(rel.published_at).toLocaleDateString(lang === 'pt-BR' ? 'pt-BR' : 'en-US');
    const sections = parseNotes(rel.body || '');
    if (sections.length > 0) {
    const asset = (rel.assets || []).find((a) => a.name === INSTALLER);
    const download = asset ? asset.browser_download_url : RELEASES_URL;
    const label = index === 0 ? I18N[lang].notes_latest : I18N[lang].notes_previous;
    const bodyHtml = sections.length > 0
      ? sections.map((section) => `
          <div class="note-section">
            <h4 class="note-section-title ${sectionClass(section.title)}">${escapeHtml(section.title)}</h4>
            <ul>${section.items.map((item) => `<li>${escapeHtml(item)}</li>`).join('')}</ul>
          </div>`).join('')
      : `<ul><li>${escapeHtml((rel.body || I18N[lang].notes_fallback).slice(0, 400))}</li></ul>`;
    return `
      <article class="note reveal">
        <div class="note-head">
          <span class="note-version">${escapeHtml(rel.tag_name)}</span>
          ${index === 0 ? `<span class="pill pill-cyan">${label}</span>` : ''}
          <span class="note-date">${date}</span>
          <a class="note-download" href="${download}" rel="noopener">${I18N[lang].notes_get} →</a>
        </div>
        ${rel.name ? `<p class="note-title">${escapeHtml(rel.name)}</p>` : ''}
        ${bodyHtml}
      </article>`;
  }).join('');
  observeReveals();
}

/// Divide o corpo da release em seções por título (##, **Título**, [EN-US] etc.).
/// Quando o corpo tem blocos EN-US e PT-BR, mantém apenas o idioma da página.
function parseNotes(body) {
  const lines = body.split('\n');
  const blocks = [];
  let current = null;
  let langBlock = null;
  const headingRe = /^(?:#{1,3}\s*(.+)|\*\*(.+)\*\*|\[([^\]]+)\]\s*)$/;

  for (const raw of lines) {
    const line = raw.trim();
    if (line.length === 0) { if (current) { blocks.push({ langBlock, section: current }); current = null; } continue; }

    const langHeader = line.match(/^\[?(EN-US|PT-BR|EN|PT)\]?[:.]?\s*$/i);
    if (langHeader) {
      if (current) { blocks.push({ langBlock, section: current }); current = null; }
      langBlock = langHeader[1].toUpperCase().startsWith('EN') ? 'EN' : 'PT';
      continue;
    }

    // Also accept markdown headings like "## EN-US" or "## PT-BR"
    const langHeading = line.match(/^#{1,3}\s*\[?(EN-US|PT-BR|EN|PT)\]?\s*$/i);
    if (langHeading) {
      if (current) { blocks.push({ langBlock, section: current }); current = null; }
      langBlock = langHeading[1].toUpperCase().startsWith('EN') ? 'EN' : 'PT';
      continue;
    }

    const heading = line.match(headingRe);
    if (heading) {
      const title = (heading[1] || heading[2] || heading[3] || '').trim();
      if (title.length > 0 && title.length < 60) {
        if (current) blocks.push({ langBlock, section: current });
        current = { title, items: [] };
        continue;
      }
    }

    const bullet = line.match(/^[-*•]\s*(.+)$/);
    if (bullet) {
      if (!current) current = { title: I18N[lang].changelog_title, items: [] };
      current.items.push(bullet[1].trim());
    } else if (current && current.items.length === 0 && line.length > 20) {
      current.items.push(line);
    }
  }
  if (current) blocks.push({ langBlock, section: current });

  // Se o corpo tem blocos dos dois idiomas, filtra pelo idioma da página.
  const hasEn = blocks.some((b) => b.langBlock === 'EN');
  const hasPt = blocks.some((b) => b.langBlock === 'PT');
  const pageLang = lang === 'pt-BR' ? 'PT' : 'EN';
  const filtered = hasEn && hasPt ? blocks.filter((b) => b.langBlock === pageLang || b.langBlock === null) : blocks;

  const sections = [];
  for (const { section } of filtered) {
    const existing = sections.find((s) => s.title === section.title);
    if (existing) existing.items.push(...section.items);
    else sections.push(section);
  }
  return sections.filter((section) => section.items.length > 0);
}

function sectionClass(title) {
  const normalized = title.toLowerCase();
  if (normalized.includes('added') || normalized.includes('adicion')) return 'st-added';
  if (normalized.includes('fix') || normalized.includes('corrig')) return 'st-fixed';
  if (normalized.includes('perf') || normalized.includes('desempen')) return 'st-perf';
  if (normalized.includes('secur') || normalized.includes('segur')) return 'st-sec';
  if (normalized.includes('chang') || normalized.includes('mudan')) return 'st-changed';
  return 'st-info';
}

function escapeHtml(value) {
  return value.replace(/[<>&"]/g, (ch) => ({ '<': '&lt;', '>': '&gt;', '&': '&amp;', '"': '&quot;' }[ch]));
}

function observeReveals() {
  const observer = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) { entry.target.classList.add('visible'); observer.unobserve(entry.target); }
    });
  }, { threshold: 0.12 });
  document.querySelectorAll('.reveal').forEach((el) => observer.observe(el));
}

const saved = localStorage.getItem('capivara-lang');
const browser = navigator.language || 'pt-BR';
applyLang(saved || (browser.toLowerCase().startsWith('pt') ? 'pt-BR' : 'en-US'));
loadReleases();
