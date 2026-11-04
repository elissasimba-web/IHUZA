import test from 'node:test';
import assert from 'node:assert/strict';
import { createWalletService, TESTNET_PASSPHRASE } from './wallet.js';

const address = 'G' + 'A'.repeat(55);

function fakeApi(overrides = {}) {
  return {
    isConnected: async () => ({ isConnected: true }),
    isAllowed: async () => ({ isAllowed: true }),
    requestAccess: async () => ({ address }),
    getAddress: async () => ({ address }),
    getNetworkDetails: async () => ({ network: 'TESTNET', networkPassphrase: TESTNET_PASSPHRASE }),
    ...overrides,
  };
}

test('inspect never requests permission when the site is not allowed', async () => {
  let prompted = false;
  const wallet = createWalletService(fakeApi({ isAllowed: async () => ({ isAllowed: false }), requestAccess: async () => { prompted = true; return { address }; } }));
  assert.deepEqual(await wallet.inspect(), { status: 'not_allowed' });
  assert.equal(prompted, false);
});

test('connection accepts only the exact Stellar testnet passphrase', async () => {
  const wallet = createWalletService(fakeApi());
  assert.deepEqual(await wallet.connect(), { status: 'ready', address, network: 'Stellar testnet' });

  const wrong = createWalletService(fakeApi({ getNetworkDetails: async () => ({ network: 'PUBLIC', networkPassphrase: 'Public Global Stellar Network ; September 2015' }) }));
  assert.deepEqual(await wrong.connect(), { status: 'wrong_network', address, network: 'PUBLIC' });
});

test('missing extension and malformed address never become ready', async () => {
  const missing = createWalletService(fakeApi({ isConnected: async () => ({ isConnected: false }) }));
  assert.deepEqual(await missing.connect(), { status: 'missing' });

  const invalid = createWalletService(fakeApi({ requestAccess: async () => ({ address: 'not-an-address' }) }));
  assert.deepEqual(await invalid.connect(), { status: 'error', message: 'Aderesi ya wallet ntiyemewe.' });
});

test('wallet API errors do not reveal a connected state', async () => {
  const wallet = createWalletService(fakeApi({ getNetworkDetails: async () => { throw new Error('private detail'); } }));
  assert.deepEqual(await wallet.connect(), { status: 'error', message: 'Ntitwabashije kugenzura Freighter. Ongera ugerageze.' });
});
