import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

export const MAP_TOOL_REVISION = "8b70fce25558f2662ebee0855c12d35fe21e7beb";
export const MAP_TOOL_ARCHIVE_SHA256 = "85d0699f57bf1dd61a52dbfb5b507b7ac16393aa0e27993cfd0d527e15f57797";

export function parsePpm(path: string) {
  const tokens = readFileSync(path, "utf8")
    .replace(/#[^\n]*/g, " ").trim().split(/\s+/);
  if (tokens.shift() !== "P3") throw new Error(`${path}: expected P3 PPM`);
  const width = Number(tokens.shift()), height = Number(tokens.shift());
  if (!Number.isInteger(width) || !Number.isInteger(height) || width < 16 || height < 16 ||
      (width & (width - 1)) || (height & (height - 1))) {
    throw new Error(`${path}: dimensions must be power-of-two and at least 16`);
  }
  if (Number(tokens.shift()) !== 255) throw new Error(`${path}: expected max value 255`);
  const rgb = tokens.map(Number);
  if (rgb.length !== width * height * 3 || rgb.some((v) => !Number.isInteger(v) || v < 0 || v > 255))
    throw new Error(`${path}: invalid pixel data`);
  return { width, height, rgb };
}

export function buildWad3(ppmPath: string, output: string, textureName = "WALL") {
  const { width, height, rgb } = parsePpm(ppmPath);
  const palette: number[][] = [];
  const indices: number[] = [];
  for (let i = 0; i < rgb.length; i += 3) {
    const color = rgb.slice(i, i + 3);
    let index = palette.findIndex((p) => p[0] === color[0] && p[1] === color[1] && p[2] === color[2]);
    if (index < 0) { index = palette.length; palette.push(color); }
    if (index >= 256) throw new Error(`${ppmPath}: texture exceeds the 256-color WAD3 limit`);
    indices.push(index);
  }
  const levels: Uint8Array[] = [];
  for (let level = 0; level < 4; level++) {
    const w = width >> level, h = height >> level;
    const data = new Uint8Array(w * h);
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++)
      data[y * w + x] = indices[(y << level) * width + (x << level)];
    levels.push(data);
  }
  const mipSize = 40 + levels.reduce((n, b) => n + b.length, 0) + 2 + 768;
  const lumpOffset = 12, directoryOffset = lumpOffset + mipSize;
  const out = Buffer.alloc(directoryOffset + 32);
  out.write("WAD3", 0); out.writeInt32LE(1, 4); out.writeInt32LE(directoryOffset, 8);
  out.write(textureName.slice(0, 15), lumpOffset, "ascii");
  out.writeUInt32LE(width, lumpOffset + 16); out.writeUInt32LE(height, lumpOffset + 20);
  let cursor = 40;
  for (let i = 0; i < 4; i++) { out.writeUInt32LE(cursor, lumpOffset + 24 + i * 4); out.set(levels[i], lumpOffset + cursor); cursor += levels[i].length; }
  out.writeUInt16LE(256, lumpOffset + cursor); cursor += 2;
  for (let i = 0; i < 256; i++) { const p = palette[i] ?? [0, 0, 0]; out.set(p, lumpOffset + cursor + i * 3); }
  out.writeInt32LE(lumpOffset, directoryOffset); out.writeInt32LE(mipSize, directoryOffset + 4);
  out.writeInt32LE(mipSize, directoryOffset + 8); out[directoryOffset + 12] = 0x43;
  out.write(textureName.slice(0, 15), directoryOffset + 16, "ascii");
  mkdirSync(dirname(output), { recursive: true }); writeFileSync(output, out);
  return { width, height, colors: palette.length, bytes: out.length };
}
