const buffers = new Map<string, string[]>();
const writers = new Map<string, (data: string) => void>();

let writeBatchTimeout: ReturnType<typeof setTimeout> | null = null;
const PENDING_WRITES = new Map<string, string>();

function flushPendingWrites() {
  console.log('[outputBuffer] flushPendingWrites, sessions:', PENDING_WRITES.size);
  PENDING_WRITES.forEach((data, sessionId) => {
    const writer = writers.get(sessionId);
    if (writer) {
      console.log('[outputBuffer] writing', data.length, 'chars to sessionId:', sessionId);
      writer(data);
    } else {
      console.warn('[outputBuffer] no writer for sessionId:', sessionId);
    }
  });
  PENDING_WRITES.clear();
}

export function initBuffer(sessionId: string): void {
  console.log('[outputBuffer] initBuffer', sessionId);
  buffers.set(sessionId, []);
}

export function pushOutput(sessionId: string, data: string): void {
  console.log('[outputBuffer] pushOutput', sessionId, 'len:', data.length);
  const writer = writers.get(sessionId);
  if (writer) {
    PENDING_WRITES.set(sessionId, (PENDING_WRITES.get(sessionId) ?? '') + data);
    if (!writeBatchTimeout) {
      writeBatchTimeout = setTimeout(() => {
        flushPendingWrites();
        writeBatchTimeout = null;
      }, 16);
    }
    return;
  }
  const buf = buffers.get(sessionId);
  if (buf) {
    buf.push(data);
  } else {
    console.warn('[outputBuffer] no buffer for sessionId:', sessionId);
  }
}

export function flushBuffer(sessionId: string, writer: (data: string) => void): void {
  console.log('[outputBuffer] flushBuffer', sessionId);
  const buf = buffers.get(sessionId);
  if (buf && buf.length > 0) {
    const joined = buf.join('');
    console.log('[outputBuffer] flushing', buf.length, 'chunks,', joined.length, 'total chars');
    writer(joined);
    buf.length = 0;
  } else {
    console.log('[outputBuffer] flushBuffer: no buffered data');
  }
}

export function setWriter(sessionId: string, writer: (data: string) => void): void {
  console.log('[outputBuffer] setWriter', sessionId);
  writers.set(sessionId, writer);
}

export function clearWriter(sessionId: string): void {
  console.log('[outputBuffer] clearWriter', sessionId);
  writers.delete(sessionId);
  PENDING_WRITES.delete(sessionId);
}

export function cleanupBuffer(sessionId: string): void {
  console.log('[outputBuffer] cleanupBuffer', sessionId);
  buffers.delete(sessionId);
  writers.delete(sessionId);
  PENDING_WRITES.delete(sessionId);
}