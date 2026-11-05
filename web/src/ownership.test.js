import test from 'node:test';
import assert from 'node:assert/strict';
import { createOwnershipService } from './ownership.js';
import { TESTNET_PASSPHRASE, WEB_ORIGIN } from './wallet.js';

const address = 'G' + 'A'.repeat(55);
const challenge = {
  challenge_id: '11111111-1111-4111-8111-111111111111',
  address,
  network_passphrase: TESTNET_PASSPHRASE,
  expires_at: 1300,
  message: `IHUZA+ wallet ownership\nversion: 2\norigin: ${WEB_ORIGIN}\npurpose: wallet ownership only; no payment\nnetwork: ${TESTNET_PASSPHRASE}\naddress: ${address}\nnonce: ${'a'.repeat(64)}\nissued_at: 1000\nexpires_at: 1300`,
};

function wallet(overrides = {}) {
  return {
    inspect: async () => ({ status: 'ready', address }),
    signChallenge: async () => ({ status: 'signed', signature_base64: 'a'.repeat(88) }),
    ...overrides,
  };
}

test('only the exact server challenge is shown for signing', async () => {
  const calls = [];
  const fetchImpl = async (path, options) => {
    calls.push({ path, options });
    return { ok: true, json: async () => challenge };
  };
  const service = createOwnershipService(wallet(), fetchImpl, () => 1100);
  assert.deepEqual(await service.prepare(address), { status: 'ready', challenge });
  assert.equal(calls[0].path, '/api/wallet/challenges');
  assert.equal(calls[0].options.credentials, 'omit');

  const wrong = createOwnershipService(wallet(), async () => ({ ok: true, json: async () => ({ ...challenge, network_passphrase: 'PUBLIC' }) }), () => 1100);
  assert.equal((await wrong.prepare(address)).status, 'error');
  const injected = createOwnershipService(wallet(), async () => ({ ok: true, json: async () => ({ ...challenge, message: `${challenge.message}\npay me` }) }), () => 1100);
  assert.equal((await injected.prepare(address)).status, 'error');
});

test('expired or switched wallet never sends a proof', async () => {
  let signed = false;
  let posted = false;
  const expired = createOwnershipService(wallet({ signChallenge: async () => { signed = true; return {}; } }), async () => { posted = true; return {}; }, () => 1300);
  assert.equal((await expired.submit(challenge)).status, 'error');
  assert.equal(signed, false);
  assert.equal(posted, false);

  const switched = createOwnershipService(wallet({ inspect: async () => ({ status: 'ready', address: 'G' + 'B'.repeat(55) }) }), async () => { posted = true; return {}; }, () => 1100);
  assert.equal((await switched.prepare(address)).status, 'error');
  assert.equal(posted, false);
});

test('proof result is accepted only after wallet signing and server confirmation', async () => {
  const calls = [];
  const service = createOwnershipService(wallet(), async (path, options) => {
    calls.push({ path, body: JSON.parse(options.body) });
    return { ok: true, json: async () => ({ status: 'wallet_key_verified', account_id: '22222222-2222-4222-8222-222222222222', qr_reference: 'x'.repeat(43) }) };
  }, () => 1100);
  const result = await service.submit(challenge);
  assert.equal(result.status, 'verified');
  assert.equal(calls[0].path, '/api/wallet/proofs');
  assert.equal(calls[0].body.challenge_id, challenge.challenge_id);

  const rejected = createOwnershipService(wallet(), async () => ({ ok: false }), () => 1100);
  assert.equal((await rejected.submit(challenge)).status, 'error');
});
