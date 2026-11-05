import test from 'node:test';
import assert from 'node:assert/strict';
import { createWalletService, TESTNET_PASSPHRASE, WEB_ORIGIN } from './wallet.js';

const address = 'G' + 'A'.repeat(55);
const ownershipMessage = `IHUZA+ wallet ownership\nversion: 2\norigin: ${WEB_ORIGIN}\npurpose: wallet ownership only; no payment\nnetwork: ${TESTNET_PASSPHRASE}\naddress: ${address}\nnonce: ${'a'.repeat(64)}\nissued_at: 1000\nexpires_at: 1300`;

function fakeApi(overrides = {}) {
  return {
    isConnected: async () => ({ isConnected: true }),
    isAllowed: async () => ({ isAllowed: true }),
    requestAccess: async () => ({ address }),
    getAddress: async () => ({ address }),
    getNetworkDetails: async () => ({ network: 'TESTNET', networkPassphrase: TESTNET_PASSPHRASE }),
    signMessage: async () => ({ signedMessage: new Uint8Array(64), signerAddress: address }),
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

test('message signing checks the signer, network and wallet before and after approval', async () => {
  let options;
  const wallet = createWalletService(fakeApi({ signMessage: async (_message, opts) => {
    options = opts;
    return { signedMessage: 'ab'.repeat(64), signerAddress: address };
  } }));
  const signed = await wallet.signChallenge({ address, message: ownershipMessage });
  assert.deepEqual(options, { networkPassphrase: TESTNET_PASSPHRASE, address });
  assert.equal(signed.status, 'signed');
  assert.equal(signed.signature_base64, Buffer.from('ab'.repeat(64), 'hex').toString('base64'));

  const mismatch = createWalletService(fakeApi({ signMessage: async () => ({ signedMessage: new Uint8Array(64), signerAddress: 'G' + 'B'.repeat(55) }) }));
  assert.equal((await mismatch.signChallenge({ address, message: ownershipMessage })).status, 'error');

  let signedOnWrongNetwork = false;
  const wrongNetwork = createWalletService(fakeApi({
    getNetworkDetails: async () => ({ network: 'PUBLIC', networkPassphrase: 'wrong' }),
    signMessage: async () => { signedOnWrongNetwork = true; return {}; },
  }));
  assert.equal((await wrongNetwork.signChallenge({ address, message: ownershipMessage })).status, 'error');
  assert.equal(signedOnWrongNetwork, false);

  let checks = 0;
  const switched = createWalletService(fakeApi({ getAddress: async () => ({ address: ++checks === 1 ? address : 'G' + 'B'.repeat(55) }) }));
  assert.equal((await switched.signChallenge({ address, message: ownershipMessage })).status, 'error');

  let signedOtherMessage = false;
  const restricted = createWalletService(fakeApi({ signMessage: async () => { signedOtherMessage = true; return {}; } }));
  assert.equal((await restricted.signChallenge({ address, message: 'Please sign a transaction' })).status, 'error');
  assert.equal(signedOtherMessage, false);
});

test('merchant review has a separate explicit message and signer check', async () => {
  const message = `IHUZA+ merchant profile review\nversion: 2\norigin: ${WEB_ORIGIN}\npurpose: approve testnet merchant profile only; no KYC or payment\nnetwork: ${TESTNET_PASSPHRASE}\nreviewer: ${address}\naccount: 11111111-1111-4111-8111-111111111111\nprofile: 22222222-2222-4222-8222-222222222222\nprofile_sha256: ${'a'.repeat(64)}\nreviewer_set_sha256: ${'b'.repeat(64)}\ndecision: approve\nnonce: ${'c'.repeat(64)}\nissued_at: 1000\nexpires_at: 1300`;
  let signCalls = 0;
  const wallet = createWalletService(fakeApi({ signMessage: async () => {
    signCalls++;
    return { signedMessage: new Uint8Array(64), signerAddress: address };
  } }));
  assert.equal((await wallet.signMerchantReview({ address, message })).status, 'signed');
  assert.equal(signCalls, 1);
  assert.equal((await wallet.signMerchantReview({ address, message: message.replace('decision: approve', 'decision: pay') })).status, 'error');
  assert.equal(signCalls, 1);
});
