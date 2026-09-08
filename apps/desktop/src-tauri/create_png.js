const fs = require('fs');
const zlib = require('zlib');
const path = require('path');

function getCrc32(buf) {
  let c;
  const crcTable = [];
  for (let n = 0; n < 256; n++) {
    c = n;
    for (let k = 0; k < 8; k++) {
      c = (c & 1) ? (0xedb88320 ^ (c >>> 1)) : (c >>> 1);
    }
    crcTable[n] = c;
  }
  let crc = 0 ^ (-1);
  for (let i = 0; i < buf.length; i++) {
    crc = (crc >>> 8) ^ crcTable[(crc ^ buf[i]) & 0xff];
  }
  return (crc ^ (-1)) >>> 0;
}

function makeChunk(type, data) {
  const lenBuf = Buffer.alloc(4);
  lenBuf.writeUInt32BE(data.length, 0);
  const typeBuf = Buffer.from(type, 'ascii');
  const crcVal = getCrc32(Buffer.concat([typeBuf, data]));
  const crcBuf = Buffer.alloc(4);
  crcBuf.writeUInt32BE(crcVal, 0);
  return Buffer.concat([lenBuf, typeBuf, data, crcBuf]);
}

function generateMinimalPng(width, height, outputPath) {
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  
  // IHDR
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8; // Bit depth: 8
  ihdr[9] = 6; // Color type: RGBA (6)
  ihdr[10] = 0; // Compression method
  ihdr[11] = 0; // Filter method
  ihdr[12] = 0; // Interlace method
  const ihdrChunk = makeChunk('IHDR', ihdr);
  
  // IDAT (raw pixels: width * 4 + 1 filter byte per scanline)
  const rawLineLength = width * 4 + 1;
  const rawData = Buffer.alloc(rawLineLength * height);
  for (let y = 0; y < height; y++) {
    const offset = y * rawLineLength;
    rawData[offset] = 0; // Filter 0 (None)
    for (let x = 0; x < width; x++) {
      const pxOffset = offset + 1 + x * 4;
      rawData[pxOffset] = 41;    // R
      rawData[pxOffset + 1] = 128; // G
      rawData[pxOffset + 2] = 185; // B
      rawData[pxOffset + 3] = 255; // A
    }
  }
  
  const compressedData = zlib.deflateSync(rawData);
  const idatChunk = makeChunk('IDAT', compressedData);
  
  // IEND
  const iendChunk = makeChunk('IEND', Buffer.alloc(0));
  
  const pngBuffer = Buffer.concat([signature, ihdrChunk, idatChunk, iendChunk]);
  fs.writeFileSync(outputPath, pngBuffer);
  console.log(`Generated PNG at ${outputPath}`);
}

const targetPath = path.join(__dirname, 'app-icon.png');
generateMinimalPng(1024, 1024, targetPath);
