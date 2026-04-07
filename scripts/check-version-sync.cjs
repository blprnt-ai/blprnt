const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const crateManifestPath = path.join(repoRoot, 'backend', 'crates', 'blprnt', 'Cargo.toml');
const crateManifest = fs.readFileSync(crateManifestPath, 'utf8');
const versionMatch = crateManifest.match(/^version\s*=\s*"([^"]+)"\s*$/m);

if (!versionMatch) {
  console.error('Missing version in backend/crates/blprnt/Cargo.toml');
  process.exit(1);
}

const crateVersion = versionMatch[1];
const packageFiles = [
  path.join(repoRoot, 'npm', 'blprnt', 'package.json'),
  ...fs
    .readdirSync(path.join(repoRoot, 'npm'), { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && entry.name !== 'blprnt')
    .map((entry) => path.join(repoRoot, 'npm', entry.name, 'package.json')),
];

const mismatches = [];
const rootPackage = JSON.parse(fs.readFileSync(packageFiles[0], 'utf8'));

for (const packageFile of packageFiles) {
  const manifest = JSON.parse(fs.readFileSync(packageFile, 'utf8'));

  if (manifest.version !== crateVersion) {
    mismatches.push(`${path.relative(repoRoot, packageFile)} version=${manifest.version} expected=${crateVersion}`);
  }
}

for (const [name, version] of Object.entries(rootPackage.optionalDependencies || {})) {
  if (version !== crateVersion) {
    mismatches.push(
      `npm/blprnt/package.json optionalDependencies.${name}=${version} expected=${crateVersion}`,
    );
  }
}

if (mismatches.length > 0) {
  console.error('Version sync check failed:');
  for (const mismatch of mismatches) {
    console.error(`- ${mismatch}`);
  }
  process.exit(1);
}

console.log(`Version sync OK: ${crateVersion}`);
