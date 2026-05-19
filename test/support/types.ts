export type TestSuiteName = 'frontend-unit' | 'ipc-contract' | 'tauri-e2e' | 'remote-e2e';

export type TestCase = {
  name: string;
  run: () => void | Promise<void>;
};

export type MockInvokeCall = {
  method: string;
  args?: Record<string, unknown>;
};

export type MockInvokeRule<T = unknown> = {
  method: string;
  match?: (args?: Record<string, unknown>) => boolean;
  result?: T;
  error?: string | Error;
};

export type MockTauriEvent<T = unknown> = {
  eventName: string;
  payload: T;
};
