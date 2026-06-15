#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');

const rootDir = path.resolve(__dirname, '..');
const rootDist = path.join(rootDir, 'dist');
const webSrc = path.join(rootDir, 'apps', 'ui', 'dist');
const tauriBundleSrc = path.join(rootDir, 'apps', 'ui', 'src-tauri', 'target', 'release', 'bundle');

const ensureCleanDir = (dir) => {
  fs.rmSync(dir, { recursive: true, force: true });
  fs.mkdirSync(dir, { recursive: true });
};

const copyIfExists = (src, dest) => {
  if (fs.existsSync(src)) {
    fs.cpSync(src, dest, { recursive: true });
    return true;
  }
  return false;
};

ensureCleanDir(rootDist);

const copiedWeb = copyIfExists(webSrc, path.join(rootDist, 'web'));
const copiedTauri = copyIfExists(tauriBundleSrc, path.join(rootDist, 'tauri'));

if (!copiedWeb) {
  console.warn('[collect-builds] Warning: frontend build output not found at', webSrc);
}

if (!copiedTauri) {
  console.warn('[collect-builds] Warning: Tauri bundle output not found at', tauriBundleSrc);
}

console.log('[collect-builds] Build artifacts copied to dist/');

