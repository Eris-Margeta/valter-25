import fs from 'fs-extra';
import path from 'path';
import chokidar from 'chokidar';
import { fileURLToPath } from 'url';

// Setup paths
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const MONOREPO_ROOT = path.resolve(__dirname, '../../');
const WEBSITE_CONTENT_DIR = path.resolve(__dirname, '../src/content/docs');
const WEBSITE_DATA_DIR = path.resolve(__dirname, '../src/data');

// Files to sync
const SYNC_MAP = {
  'README.md': 'index.md', // Main readme becomes home page content or intro
  'ROADMAP.md': 'roadmap.md',
  'CHANGELOG.md': 'changelog.md',
  'LICENSE': 'license.md'
};

// Ensure dirs exist
fs.ensureDirSync(WEBSITE_CONTENT_DIR);
fs.ensureDirSync(WEBSITE_DATA_DIR);

async function syncFile(sourceName, targetName) {
  const sourcePath = path.join(MONOREPO_ROOT, sourceName);
  const targetPath = path.join(WEBSITE_CONTENT_DIR, targetName);

  if (fs.existsSync(sourcePath)) {
    let content = await fs.readFile(sourcePath, 'utf-8');
    
    // Add Frontmatter for Astro if missing
    if (!content.startsWith('---')) {
      const title = targetName.replace('.md', '').toUpperCase();
      content = `---\ntitle: ${title}\n---\n\n${content}`;
    }

    await fs.writeFile(targetPath, content);
    console.log(`[SYNC] Synced ${sourceName} -> ${targetName}`);
  } else {
    console.warn(`[SYNC] Warning: Source file ${sourceName} not found.`);
  }
}

async function syncVersion() {
  // Extract version from core/Cargo.toml
  const cargoPath = path.join(MONOREPO_ROOT, 'core/Cargo.toml');
  if (fs.existsSync(cargoPath)) {
    const content = await fs.readFile(cargoPath, 'utf-8');
    const match = content.match(/name\s*=\s*"valter"\s*version\s*=\s*"([\d\.]+)"/);
    if (match && match[1]) {
      const versionData = { version: match[1], syncedAt: new Date().toISOString() };
      await fs.writeJson(path.join(WEBSITE_DATA_DIR, 'version.json'), versionData);
      console.log(`[SYNC] Detected VALTER Version: ${match[1]}`);
    }
  }
}

async function runSync() {
  console.log('--- STARTING DOCS SYNC ---');
  await syncVersion();
  for (const [src, dest] of Object.entries(SYNC_MAP)) {
    await syncFile(src, dest);
  }
  console.log('--- SYNC COMPLETE ---');
}

// Watch mode (optional, run with --watch)
if (process.argv.includes('--watch')) {
  console.log('[WATCH] Watching for changes in root markdown files...');
  chokidar.watch(Object.keys(SYNC_MAP).map(f => path.join(MONOREPO_ROOT, f))).on('change', () => {
    runSync();
  });
} else {
  runSync();
}

