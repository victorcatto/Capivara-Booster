const { execSync } = require('child_process');
const runId = execSync('gh run list --repo victorcatto/Capivara-Booster --workflow release.yml --limit 1 --json databaseId --jq ".[0].databaseId"', { encoding: 'utf8', shell: true }).trim();
const raw = execSync(`gh api repos/victorcatto/Capivara-Booster/actions/runs/${runId}/jobs`, { encoding: 'utf8', maxBuffer: 20 * 1024 * 1024, shell: true });
const jobs = JSON.parse(raw).jobs;
for (const job of jobs) {
  console.log(`JOB: ${job.name} => ${job.conclusion}`);
  for (const s of job.steps) {
    if (s.conclusion !== 'success' && s.conclusion !== 'skipped') console.log(`  STEP: ${s.name} => ${s.conclusion}`);
  }
}
