import assert from 'node:assert/strict';
import {
  cleanupBuffer,
  clearWriter,
  flushBuffer,
  initBuffer,
  pushOutput,
  setWriter,
} from '../src/services/outputBuffer';
import { failOnError, runTests, test } from './support/testRunner';

test('outputBuffer: buffer, flush, direct writer, cleanup', async () => {
  cleanupBuffer('s1');
  const writes: string[] = [];

  initBuffer('s1');
  pushOutput('s1', 'a');
  pushOutput('s1', 'b');
  flushBuffer('s1', (data) => writes.push(data));
  assert.deepEqual(writes, ['ab']);

  setWriter('s1', (data) => writes.push(data));
  pushOutput('s1', 'c');
  await new Promise((r) => setTimeout(r, 32));
  assert.deepEqual(writes, ['ab', 'c']);

  clearWriter('s1');
  pushOutput('s1', 'd');
  flushBuffer('s1', (data) => writes.push(data));
  assert.deepEqual(writes, ['ab', 'c', 'd']);

  cleanupBuffer('s1');
  pushOutput('s1', 'ignored');
  flushBuffer('s1', (data) => writes.push(data));
  assert.deepEqual(writes, ['ab', 'c', 'd']);
});

test('outputBuffer: isolates multiple session buffers', () => {
  cleanupBuffer('a');
  cleanupBuffer('b');
  const aWrites: string[] = [];
  const bWrites: string[] = [];

  initBuffer('a');
  initBuffer('b');
  pushOutput('a', 'A1');
  pushOutput('b', 'B1');
  flushBuffer('a', (data) => aWrites.push(data));
  flushBuffer('b', (data) => bWrites.push(data));

  assert.deepEqual(aWrites, ['A1']);
  assert.deepEqual(bWrites, ['B1']);
  cleanupBuffer('a');
  cleanupBuffer('b');
});

runTests('Output buffer tests').catch(failOnError);
