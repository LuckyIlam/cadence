const sharp = require('sharp');
const path = require('path');
const fs = require('fs');
const os = require('os');
const pngToIco = require('png-to-ico').default;

const SVG_PATH = path.resolve(__dirname, '..', 'src-tauri', 'icons', 'conductor.svg');
const ICONS_DIR = path.resolve(__dirname, '..', 'src-tauri', 'icons');

const sizes = [
  { name: '32x32.png', size: 32 },
  { name: '64x64.png', size: 64 },
  { name: '128x128.png', size: 128 },
  { name: '128x128@2x.png', size: 256 },
  { name: 'icon.png', size: 512 },
  { name: 'Square30x30Logo.png', size: 30 },
  { name: 'Square44x44Logo.png', size: 44 },
  { name: 'Square71x71Logo.png', size: 71 },
  { name: 'Square89x89Logo.png', size: 89 },
  { name: 'Square107x107Logo.png', size: 107 },
  { name: 'Square142x142Logo.png', size: 142 },
  { name: 'Square150x150Logo.png', size: 150 },
  { name: 'Square284x284Logo.png', size: 284 },
  { name: 'Square310x310Logo.png', size: 310 },
  { name: 'StoreLogo.png', size: 50 },
];

async function generate() {
  const svg = fs.readFileSync(SVG_PATH, 'utf-8');

  for (const { name, size } of sizes) {
    await sharp(Buffer.from(svg))
      .resize(size, size)
      .png()
      .toFile(path.join(ICONS_DIR, name));
    console.log(`Generated ${name} (${size}x${size})`);
  }

  // Generate ICO from temp PNG files
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'cadence-ico-'));
  const icoSizes = [16, 32, 48, 64, 128, 256];
  const icoFiles = [];
  for (const size of icoSizes) {
    const filePath = path.join(tmpDir, `icon-${size}.png`);
    await sharp(Buffer.from(svg))
      .resize(size, size)
      .png()
      .toFile(filePath);
    icoFiles.push(filePath);
  }
  const icoBuf = await pngToIco(icoFiles);
  fs.writeFileSync(path.join(ICONS_DIR, 'icon.ico'), icoBuf);
  console.log('Generated icon.ico');

  // Clean up temp files
  for (const f of icoFiles) {
    fs.unlinkSync(f);
  }
  fs.rmdirSync(tmpDir);

  // Note: icon.icns not regenerated (needs macOS tools)
  console.log('Note: icon.icns not regenerated (needs macOS tools)');
}

generate().catch(console.error);
