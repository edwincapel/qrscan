import { readBarcodesFromImageData } from "zxing-wasm/reader";

interface DecodeRequest {
  imageData: ImageData;
}

interface DecodeResponse {
  text: string | null;
  error?: string;
}

/** Convert RGBA ImageData to grayscale using luminance weighting. */
function toGrayscale(img: ImageData): Uint8Array {
  const gray = new Uint8Array(img.width * img.height);
  const d = img.data;
  for (let i = 0; i < gray.length; i++) {
    const off = i * 4;
    gray[i] = Math.round(0.299 * d[off] + 0.587 * d[off + 1] + 0.114 * d[off + 2]);
  }
  return gray;
}

/** Reconstruct ImageData from grayscale buffer. */
function fromGrayscale(gray: Uint8Array, w: number, h: number): ImageData {
  const data = new Uint8ClampedArray(w * h * 4);
  for (let i = 0; i < gray.length; i++) {
    const off = i * 4;
    data[off] = data[off + 1] = data[off + 2] = gray[i];
    data[off + 3] = 255;
  }
  return new ImageData(data, w, h);
}

/** Downscale grayscale buffer to fit within maxDim. */
function downscale(gray: Uint8Array, w: number, h: number, maxDim: number): { buf: Uint8Array; w: number; h: number } {
  const longEdge = Math.max(w, h);
  if (longEdge <= maxDim) return { buf: gray, w, h };
  const scale = maxDim / longEdge;
  const nw = Math.round(w * scale);
  const nh = Math.round(h * scale);
  const out = new Uint8Array(nw * nh);
  for (let y = 0; y < nh; y++) {
    const sy = Math.min(Math.floor(y / scale), h - 1);
    for (let x = 0; x < nw; x++) {
      const sx = Math.min(Math.floor(x / scale), w - 1);
      out[y * nw + x] = gray[sy * w + sx];
    }
  }
  return { buf: out, w: nw, h: nh };
}

/** Apply Otsu's threshold to grayscale buffer. */
function otsuThreshold(gray: Uint8Array): Uint8Array {
  const hist = new Int32Array(256);
  for (const v of gray) hist[v]++;
  const total = gray.length;
  let sum = 0;
  for (let i = 0; i < 256; i++) sum += i * hist[i];
  let sumB = 0, wB = 0, maxVar = 0, threshold = 0;
  for (let t = 0; t < 256; t++) {
    wB += hist[t];
    if (wB === 0) continue;
    const wF = total - wB;
    if (wF === 0) break;
    sumB += t * hist[t];
    const mB = sumB / wB;
    const mF = (sum - sumB) / wF;
    const variance = wB * wF * (mB - mF) * (mB - mF);
    if (variance > maxVar) { maxVar = variance; threshold = t; }
  }
  const out = new Uint8Array(gray.length);
  for (let i = 0; i < gray.length; i++) {
    out[i] = gray[i] > threshold ? 255 : 0;
  }
  return out;
}

/** Invert grayscale buffer (XOR 0xFF). */
function invert(gray: Uint8Array): Uint8Array {
  const out = new Uint8Array(gray.length);
  for (let i = 0; i < gray.length; i++) out[i] = gray[i] ^ 0xff;
  return out;
}

async function tryDecode(img: ImageData): Promise<string | null> {
  const results = await readBarcodesFromImageData(img, {
    formats: ["QRCode"],
    tryHarder: true,
    maxNumberOfSymbols: 1,
  });
  return results.length > 0 ? results[0].text : null;
}

async function decodePipeline(imageData: ImageData): Promise<string | null> {
  const { width: w, height: h } = imageData;
  const originalGray = toGrayscale(imageData);

  // Pass 1: native resolution
  let result = await tryDecode(fromGrayscale(originalGray, w, h));
  if (result) return result;

  // Pass 2: downscale (only if >2500px on long edge)
  if (Math.max(w, h) > 2500) {
    const ds = downscale(originalGray, w, h, 1920);
    result = await tryDecode(fromGrayscale(ds.buf, ds.w, ds.h));
    if (result) return result;
  }

  // Pass 3: Otsu threshold on original
  const thresholded = otsuThreshold(originalGray);
  result = await tryDecode(fromGrayscale(thresholded, w, h));
  if (result) return result;

  // Pass 4: invert original
  const inverted = invert(originalGray);
  result = await tryDecode(fromGrayscale(inverted, w, h));
  if (result) return result;

  // Pass 5: invert + threshold
  const invertedThresh = otsuThreshold(inverted);
  result = await tryDecode(fromGrayscale(invertedThresh, w, h));
  if (result) return result;

  return null;
}

self.onmessage = async (e: MessageEvent<DecodeRequest>) => {
  try {
    const text = await decodePipeline(e.data.imageData);
    self.postMessage({ text } as DecodeResponse);
  } catch (err) {
    self.postMessage({ text: null, error: String(err) } as DecodeResponse);
  }
};
