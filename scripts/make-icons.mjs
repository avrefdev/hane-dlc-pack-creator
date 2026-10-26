import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";

const crcTable = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});
const crc = (buf) => {
  let c = 0xffffffff;
  for (const b of buf) c = crcTable[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
};
const chunk = (type, data) => {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type), data]);
  const c = Buffer.alloc(4);
  c.writeUInt32BE(crc(body));
  return Buffer.concat([len, body, c]);
};

function png(size, pixel) {
  const raw = Buffer.alloc(size * (size * 4 + 1));
  const ss = 4;
  for (let y = 0; y < size; y++) {
    raw[y * (size * 4 + 1)] = 0;
    for (let x = 0; x < size; x++) {
      let r = 0, g = 0, b = 0, a = 0;
      for (let sy = 0; sy < ss; sy++)
        for (let sx = 0; sx < ss; sx++) {
          const p = pixel((x + (sx + 0.5) / ss) / size, (y + (sy + 0.5) / ss) / size);
          r += p[0] * p[3]; g += p[1] * p[3]; b += p[2] * p[3]; a += p[3];
        }
      const o = y * (size * 4 + 1) + 1 + x * 4;
      raw[o] = a ? r / a : 0; raw[o + 1] = a ? g / a : 0; raw[o + 2] = a ? b / a : 0;
      raw[o + 3] = (a / (ss * ss)) * 255;
    }
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0); ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8; ihdr[9] = 6;
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", ihdr), chunk("IDAT", deflateSync(raw)), chunk("IEND", Buffer.alloc(0)),
  ]);
}

const inRect = (x, y, x0, y0, x1, y1) => x >= x0 && x <= x1 && y >= y0 && y <= y1;

function glyph(u, v) {
  if (inRect(u, v, 0.28, 0.24, 0.39, 0.76)) return 1;
  if (inRect(u, v, 0.61, 0.24, 0.72, 0.76)) return 1;
  if (inRect(u, v, 0.39, 0.45, 0.61, 0.55)) return 2;
  return 0;
}

function roundedSquare(u, v, radius) {
  const dx = Math.max(Math.abs(u - 0.5) - (0.5 - radius), 0);
  const dy = Math.max(Math.abs(v - 0.5) - (0.5 - radius), 0);
  return Math.hypot(dx, dy) <= radius;
}

const BG = [17, 17, 17], FG = [237, 237, 237], BAR = [150, 150, 150];

mkdirSync("src-tauri/icons", { recursive: true });
writeFileSync(
  "src-tauri/icons/app.png",
  png(1024, (u, v) => {
    if (!roundedSquare(u, v, 0.2)) return [0, 0, 0, 0];
    const g = glyph(u, v);
    return g === 1 ? [...FG, 1] : g === 2 ? [...BAR, 1] : [...BG, 1];
  }),
);

mkdirSync("ui/img", { recursive: true });
writeFileSync(
  "ui/img/logo.svg",
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><rect x="28" y="24" width="11" height="52" fill="#ededed"/><rect x="61" y="24" width="11" height="52" fill="#ededed"/><rect x="39" y="45" width="22" height="10" fill="#969696"/></svg>`,
);
console.log("icons ready");
