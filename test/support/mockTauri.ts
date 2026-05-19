import type { UnlistenFn } from '@tauri-apps/api/event';
import { __resetIpcTestAdapters, __setIpcTestAdapters } from '../../src/services/ipc';
import type { MockInvokeCall, MockInvokeRule } from './types';

type Listener = (event: { payload: unknown }) => void;

const calls: MockInvokeCall[] = [];
const rules: MockInvokeRule[] = [];
const listeners = new Map<string, Set<Listener>>();

async function invokeMock<T>(method: string, args?: Record<string, unknown>): Promise<T> {
  calls.push({ method, args });
  const rule = rules.find((r) => r.method === method && (!r.match || r.match(args)));
  if (!rule) {
    throw new Error(`No mock invoke rule registered for ${method}`);
  }
  if (rule.error) {
    throw rule.error instanceof Error ? rule.error : new Error(rule.error);
  }
  return rule.result as T;
}

async function listenMock<T>(eventName: string, handler: (event: { payload: T }) => void): Promise<UnlistenFn> {
  const set = listeners.get(eventName) ?? new Set<Listener>();
  set.add(handler as Listener);
  listeners.set(eventName, set);
  return () => {
    set.delete(handler as Listener);
  };
}

export function installMockTauri(): void {
  __setIpcTestAdapters({ invoke: invokeMock, listen: listenMock });
}

export function restoreMockTauri(): void {
  __resetIpcTestAdapters();
  resetMockTauri();
}

export function mockInvoke<T>(method: string, result: T, match?: MockInvokeRule<T>['match']): void {
  rules.push({ method, result, match });
}

export function mockInvokeError(method: string, error: string | Error, match?: MockInvokeRule['match']): void {
  rules.push({ method, error, match });
}

export function emitTauriEvent<T>(eventName: string, payload: T): void {
  for (const listener of listeners.get(eventName) ?? []) {
    listener({ payload });
  }
}

export function getInvokeCalls(): MockInvokeCall[] {
  return [...calls];
}

export function resetMockTauri(): void {
  calls.length = 0;
  rules.length = 0;
  listeners.clear();
}
