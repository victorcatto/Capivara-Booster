const { execSync } = require('child_process');
const fs = require('fs');

const raw = execSync('gh api "repos/victorcatto/Capivara-Booster/releases?per_page=2"', { encoding: 'utf8', maxBuffer: 10 * 1024 * 1024, shell: true });
const rels = JSON.parse(raw);
const data = rels.map((r) => ({
  tag: r.tag_name,
  name: r.name,
  date: (r.published_at || '').slice(0, 10),
  url: r.html_url,
  installer: (r.assets || []).find((a) => a.name === 'CapivaraBooster-Setup.exe')?.browser_download_url || r.html_url,
  notes: (r.body || '').split('\n').filter((l) => l.trim() !== '').slice(0, 60),
}));
fs.writeFileSync('C:/Users/User/Documents/PROJETO2/docs/site/releases-data.json', JSON.stringify(data, null, 2));
console.log('tags:', data.map((d) => d.tag).join(', '));
