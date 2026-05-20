import { spawnSync } from 'node:child_process';

const files = [
  'test/stores.test.ts',
  'test/output-buffer.test.ts',
  'test/types.test.ts',
  'test/ipc-contract.test.ts'
];

for (const file of files) {
  const result = spawnSync(process.platform === 'win32' ? 'npx.cmd' : 'npx', ['tsx', file], { stdio: 'inherit' });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
