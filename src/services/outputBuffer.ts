const buffers = new Map<string, string[]>();
const writers = new Map<string, (data: string) => void>();

let writeBatchTimeout: ReturnType<typeof setTimeout> | null = null;
const PENDING_WRITES = new Map<string, string>();

function flushPendingWrites() {
  PENDING_WRITES.forEach((data, sessionId) => {
    const writer = writers.get(sessionId);
    if (writer) writer(data);
  });
  PENDING_WRITES.clear();
}

export function initBuffer(sessionId: string): void {
  buffers.set(sessionId, []);
}

export function pushOutput(sessionId: string, data: string): void {
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
  }
}

export function flushBuffer(sessionId: string, writer: (data: string) => void): void {
  const buf = buffers.get(sessionId);
  if (buf && buf.length > 0) {
    writer(buf.join(''));
    buf.length = 0;
  }
}

export function setWriter(sessionId: string, writer: (data: string) => void): void {
  writers.set(sessionId, writer);
}

export function clearWriter(sessionId: string): void {
  writers.delete(sessionId);
  PENDING_WRITES.delete(sessionId);
}

export function cleanupBuffer(sessionId: string): void {
  buffers.delete(sessionId);
  writers.delete(sessionId);
  PENDING_WRITES.delete(sessionId);
}