import { invoke } from '@tauri-apps/api/core';

/** Log a simple message to the app's log.txt file. */
export async function logMessage(message: string): Promise<void> {
  try {
    await invoke('log_message', { message });
  } catch (e) {
    console.error('[Logger] log_message failed:', e);
  }
}

/** Log a tagged debug entry with JSON payload. */
export async function logDebug(tag: string, payload: unknown): Promise<void> {
  try {
    const jsonStr =
      typeof payload === 'string' ? payload : JSON.stringify(payload);
    await invoke('log_debug', { tag, payload: jsonStr });
  } catch (e) {
    console.error('[Logger] log_debug failed:', e);
  }
}

/** Log a tagged entry with a JSON-serializable object. */
export async function logJson(tag: string, payload: unknown): Promise<void> {
  try {
    await invoke('log_json', { tag, payload });
  } catch (e) {
    console.error('[Logger] log_json failed:', e);
  }
}