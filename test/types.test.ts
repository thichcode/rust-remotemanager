import assert from 'node:assert/strict';
import { AuthType, ConnectionType, ProxyType, TunnelType } from '../src/services/types';
import { failOnError, runTests, test } from './support/testRunner';

test('types: enum values match backend contract', () => {
  assert.equal(ConnectionType.SSH, 'ssh');
  assert.equal(ConnectionType.RDP, 'rdp');
  assert.equal(ConnectionType.Serial, 'serial');
  assert.equal(AuthType.Password, 'password');
  assert.equal(AuthType.Key, 'key');
  assert.equal(AuthType.Agent, 'agent');
  assert.equal(ProxyType.None, 'none');
  assert.equal(ProxyType.Socks5, 'socks5');
  assert.equal(ProxyType.Http, 'http');
  assert.equal(TunnelType.Local, 'local');
  assert.equal(TunnelType.Remote, 'remote');
  assert.equal(TunnelType.Dynamic, 'dynamic');
});

runTests('Type contract tests').catch(failOnError);
