import type { DecodeResult } from "./types";

const DECODE_TIMEOUT_MS = 5000;

let worker: Worker | null = null;

function getWorker(): Worker {
  if (!worker) {
    worker = new Worker(new URL("./decoder.worker.ts", import.meta.url), {
      type: "module",
    });
    worker.onerror = () => {
      worker?.terminate();
      worker = null;
    };
  }
  return worker;
}

/**
 * Load a base64-encoded PNG into ImageData via OffscreenCanvas.
 * Uses atob() — no fetch(), no filesystem access, no CSP issues.
 */
async function loadImageFromBase64(base64: string): Promise<ImageData> {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  const blob = new Blob([bytes], { type: "image/png" });
  const bitmap = await createImageBitmap(blob);
  const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("Failed to get canvas context");
  ctx.drawImage(bitmap, 0, 0);
  return ctx.getImageData(0, 0, bitmap.width, bitmap.height);
}

/**
 * Decode a QR code from a base64-encoded PNG.
 * Runs the 5-pass pipeline in a persistent Web Worker.
 * Times out after 5 seconds and respawns the Worker.
 */
export async function decodeQR(
  imageData: string,
): Promise<DecodeResult | null> {
  const imgData = await loadImageFromBase64(imageData);
  const w = getWorker();

  return new Promise<DecodeResult | null>((resolve) => {
    const timeout = setTimeout(() => {
      w.terminate();
      worker = null;
      resolve(null);
    }, DECODE_TIMEOUT_MS);

    w.onmessage = (e: MessageEvent<{ text: string | null }>) => {
      clearTimeout(timeout);
      resolve(e.data.text ? { text: e.data.text } : null);
    };

    w.postMessage({ imageData: imgData });
  });
}
