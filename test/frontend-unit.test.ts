import { spawnSync } from 'node:child_process';

const files = [
  'test/stores.test.ts',
  'test/output-buffer.test.ts',
  'test/types.test.ts',
  'test/ipc-contract.test.ts'
];

function runTestFile(file: string): boolean {
  if (process.platform === 'win32') {
    const result = spawnSync('cmd.exe', ['/c', 'npx', 'tsx', file], { stdio: 'inherit' });
    return result.status === 0 || result.status === null;
  }
  const result = spawnSync('npx', ['tsx', file], { stdio: 'inherit' });
  return result.status === 0 || result.status === null;
}

let allPassed = true;
for (const file of files) {
  const passed = runTestFile(file);
  if (!passed) {
    console.error(`${file} FAILED`);
    allPassed = false;
  }
}
if (!allPassed) {
  process.exit(1);
}
