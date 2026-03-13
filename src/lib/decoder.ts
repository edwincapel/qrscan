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

/** Load a PNG file path as ImageData via OffscreenCanvas. */
async function loadImage(path: string): Promise<ImageData> {
  const response = await fetch(path);
  const blob = await response.blob();
  const bitmap = await createImageBitmap(blob);
  const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("Failed to get canvas context");
  ctx.drawImage(bitmap, 0, 0);
  return ctx.getImageData(0, 0, bitmap.width, bitmap.height);
}

/**
 * Decode a QR code from a PNG file at the given path.
 * Runs the 5-pass pipeline in a persistent Web Worker.
 * Times out after 5 seconds and respawns the Worker.
 */
export async function decodeQR(
  imagePath: string,
): Promise<DecodeResult | null> {
  const imageData = await loadImage(imagePath);
  const w = getWorker();

  return new Promise<DecodeResult | null>((resolve) => {
    const timeout = setTimeout(() => {
      w.terminate();
      worker = null;
      resolve(null);
    }, DECODE_TIMEOUT_MS);

    w.onmessage = (e: MessageEvent<{ text: string | null }>) => {
      clearTimeout(timeout);
      if (e.data.text) {
        resolve({ text: e.data.text });
      } else {
        resolve(null);
      }
    };

    w.postMessage({ imageData });
  });
}
