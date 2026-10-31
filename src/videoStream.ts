const RETRY_DELAY_MS = 1000;
const NAL_IDR = 5;

const isKeyframe = (frame: Uint8Array): boolean => {
  for (let i = 0; i + 3 < frame.length; i++) {
    if (frame[i] === 0 && frame[i + 1] === 0 && frame[i + 2] === 1 && (frame[i + 3] & 0x1f) === NAL_IDR) {
      return true;
    }
  }
  return false;
};

/** Decodes the local length-prefixed H.264 stream onto a canvas; returns a stop function. */
export const playVideoStream = (url: string, canvas: HTMLCanvasElement): (() => void) => {
  const abort = new AbortController();
  const context = canvas.getContext('2d');
  let decoder: VideoDecoder | null = null;
  let needsKeyframe = true;
  let timestamp = 0;

  const createDecoder = () => {
    decoder = new VideoDecoder({
      output: (frame) => {
        if (canvas.width !== frame.displayWidth || canvas.height !== frame.displayHeight) {
          canvas.width = frame.displayWidth;
          canvas.height = frame.displayHeight;
        }
        context?.drawImage(frame, 0, 0);
        frame.close();
      },
      error: () => {
        decoder = null;
        needsKeyframe = true;
      },
    });
    decoder.configure({ codec: 'avc1.640033', optimizeForLatency: true });
  };

  const decodeFrame = (frame: Uint8Array) => {
    const key = isKeyframe(frame);
    if (needsKeyframe && !key) return;
    if (!decoder) createDecoder();
    needsKeyframe = false;
    timestamp += 1;
    decoder?.decode(
      new EncodedVideoChunk({ type: key ? 'key' : 'delta', timestamp, data: frame }),
    );
  };

  const readStream = async () => {
    const response = await fetch(url, { signal: abort.signal });
    const reader = response.body?.getReader();
    if (!reader) return;

    let pending = new Uint8Array(0);
    while (true) {
      const { done, value } = await reader.read();
      if (done) return;

      const merged = new Uint8Array(pending.length + value.length);
      merged.set(pending);
      merged.set(value, pending.length);

      let offset = 0;
      while (merged.length - offset >= 4) {
        const size = new DataView(merged.buffer, offset).getUint32(0);
        if (merged.length - offset - 4 < size) break;
        decodeFrame(merged.subarray(offset + 4, offset + 4 + size));
        offset += 4 + size;
      }
      pending = merged.slice(offset);
    }
  };

  const run = async () => {
    while (!abort.signal.aborted) {
      needsKeyframe = true;
      await readStream().catch(() => undefined);
      await new Promise((resolve) => setTimeout(resolve, RETRY_DELAY_MS));
    }
  };
  void run();

  return () => {
    abort.abort();
    if (decoder?.state === 'configured') decoder.close();
  };
};
