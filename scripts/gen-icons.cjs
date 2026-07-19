const fs = require('fs');
const path = require('path');

const dir = path.join(__dirname, '..', 'src-tauri', 'icons');
fs.mkdirSync(dir, { recursive: true });

// Pre-computed CRC32 for IHDR, IDAT, IEND chunks
// 1x1 white pixel PNG
const pngBuf = Buffer.from([
  0x89,0x50,0x4E,0x47,0x0D,0x0A,0x1A,0x0A,
  0x00,0x00,0x00,0x0D,0x49,0x48,0x44,0x52,
  0x00,0x00,0x00,0x01,0x00,0x00,0x00,0x01,
  0x08,0x02,0x00,0x00,0x00,0x90,0x77,0x53,
  0xDE,
  0x00,0x00,0x00,0x0C,0x49,0x44,0x41,0x54,
  0x08,0xD7,0x63,0x60,0x60,0x60,0x00,
  0x00,0x00,0x04,0x00,0x01,0x27,0x34,0x27,
  0x00,0x00,0x00,0x00,0x49,0x45,0x4E,0x44,
  0xAE,0x42,0x60,0x82
]);

fs.writeFileSync(path.join(dir, '32x32.png'), pngBuf);
fs.writeFileSync(path.join(dir, '128x128.png'), pngBuf);
fs.writeFileSync(path.join(dir, '128x128@2x.png'), pngBuf);
fs.writeFileSync(path.join(dir, 'icon.png'), pngBuf);

// Create ICO file
const header = Buffer.alloc(6);
header.writeUInt16LE(0, 0);
header.writeUInt16LE(1, 2);
header.writeUInt16LE(1, 4);

const entry = Buffer.alloc(16);
entry.writeUInt8(1, 0);
entry.writeUInt8(1, 1);
entry.writeUInt8(0, 2);
entry.writeUInt8(0, 3);
entry.writeUInt16LE(1, 4);
entry.writeUInt16LE(32, 6);
entry.writeUInt32LE(pngBuf.length, 8);
entry.writeUInt32LE(22, 12);

const ico = Buffer.concat([header, entry, pngBuf]);
fs.writeFileSync(path.join(dir, 'icon.ico'), ico);

console.log('All icons created in ' + dir);
