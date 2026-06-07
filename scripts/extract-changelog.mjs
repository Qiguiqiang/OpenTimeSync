import fs from 'node:fs';

const [, , versionArg] = process.argv;

if (!versionArg) {
  console.error('Usage: node scripts/extract-changelog.mjs <version>');
  process.exit(1);
}

const changelog = fs.readFileSync(new URL('../CHANGELOG.md', import.meta.url), 'utf8');
const escaped = versionArg.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const pattern = new RegExp(`## \\[${escaped}\\][^\\n]*\\n([\\s\\S]*?)(?=\\n## \\[|$)`);
const match = changelog.match(pattern);

if (!match) {
  console.error(`Version ${versionArg} not found in CHANGELOG.md`);
  process.exit(2);
}

const notes = match[1].trim();
process.stdout.write(notes);
