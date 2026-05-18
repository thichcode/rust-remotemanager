const buffers = new Map<string, string[]>();
const writers = new Map<string, (data: string) => void>();

export function initBuffer(sessionId: string): void {
  buffers.set(sessionId, []);
}

export function pushOutput(sessionId: string, data: string): void {
  const writer = writers.get(sessionId);
  if (writer) {
    writer(data);
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
    for (const chunk of buf) {
      writer(chunk);
    }
    buf.length = 0;
  }
}

/** Register a direct writer. After this, pushOutput calls writer immediately. */
export function setWriter(sessionId: string, writer: (data: string) => void): void {
  writers.set(sessionId, writer);
}

export function clearWriter(sessionId: string): void {
  writers.delete(sessionId);
}

export function cleanupBuffer(sessionId: string): void {
  buffers.delete(sessionId);
  writers.delete(sessionId);
}
