import type { TestCase } from './types';

const tests: TestCase[] = [];

export function test(name: string, run: TestCase['run']): void {
  tests.push({ name, run });
}

export function resetTests(): void {
  tests.length = 0;
}

export async function runTests(suiteName = 'Tests', cases: TestCase[] = tests): Promise<void> {
  let passed = 0;

  for (const t of cases) {
    await t.run();
    passed += 1;
    console.log(`✓ ${t.name}`);
  }

  console.log(`\n${suiteName}: ${passed}/${cases.length} passed`);
}

export function failOnError(error: unknown): never {
  console.error(error);
  process.exit(1);
}
