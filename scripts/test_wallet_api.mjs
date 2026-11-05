// Local integration check. Keys are generated in memory and never written or printed.
import assert from 'node:assert/strict';
import { createHash, generateKeyPairSync, sign } from 'node:crypto';

const api = process.env.IHUZA_API_URL || 'http://127.0.0.1:3000/api';
const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';

function base32(bytes) {
  let bits = 0;
  let value = 0;
  let output = '';
  for (const byte of bytes) {
    value = (value << 8) | byte;
    bits += 8;
    while (bits >= 5) {
      output += alphabet[(value >>> (bits -= 5)) & 31];
    }
  }
  if (bits) output += alphabet[(value << (5 - bits)) & 31];
  return output;
}

function stellarAddress(publicKey) {
  const spki = publicKey.export({ type: 'spki', format: 'der' });
  const prefix = Buffer.from('302a300506032b6570032100', 'hex');
  assert.ok(spki.subarray(0, prefix.length).equals(prefix));
  const payload = Buffer.concat([Buffer.from([48]), spki.subarray(prefix.length)]);
  let crc = 0;
  for (const byte of payload) {
    crc ^= byte << 8;
    for (let i = 0; i < 8; i++) crc = ((crc << 1) ^ (crc & 0x8000 ? 0x1021 : 0)) & 0xffff;
  }
  return base32(Buffer.concat([payload, Buffer.from([crc & 255, crc >>> 8])]));
}

async function post(path, body) {
  const response = await fetch(`${api}${path}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json', origin: 'http://127.0.0.1:5173' },
    body: JSON.stringify(body),
  });
  return { status: response.status, body: await response.json(), cookie: response.headers.get('set-cookie') };
}

function signature(message, privateKey) {
  const digest = createHash('sha256')
    .update('Stellar Signed Message:\n')
    .update(message, 'utf8')
    .digest();
  return sign(null, digest, privateKey).toString('base64');
}

const { publicKey, privateKey } = generateKeyPairSync('ed25519');
const address = stellarAddress(publicKey);
assert.match(address, /^G[A-Z2-7]{55}$/);

assert.equal((await fetch(`${api}/wallet/challenges`, {
  method: 'POST',
  headers: { 'content-type': 'application/json', origin: 'https://wrong.example' },
  body: JSON.stringify({ address }),
})).status, 403);
assert.equal((await post('/wallet/challenges', { address: 'bad' })).status, 400);
const first = await post('/wallet/challenges', { address });
assert.equal(first.status, 200, JSON.stringify(first.body));
assert.equal(first.body.address, address);
assert.ok(first.body.message.includes(`address: ${address}`));
assert.ok(first.body.message.includes('version: 2\norigin: http://127.0.0.1:5173\npurpose: wallet ownership only; no payment'));

const badProof = await post('/wallet/proofs', {
  challenge_id: first.body.challenge_id,
  signature_base64: 'bad',
});
assert.equal(badProof.status, 401);
assert.equal(badProof.cookie, null);
const wrongOriginSignature = signature(first.body.message.replace('origin: http://127.0.0.1:5173', 'origin: https://wrong.example'), privateKey);
assert.equal((await post('/wallet/proofs', {
  challenge_id: first.body.challenge_id,
  signature_base64: wrongOriginSignature,
})).status, 401);

const signed = signature(first.body.message, privateKey);
const accepted = await post('/wallet/proofs', {
  challenge_id: first.body.challenge_id,
  signature_base64: signed,
});
assert.equal(accepted.status, 200, JSON.stringify(accepted.body));
assert.equal(accepted.body.status, 'wallet_key_verified');
assert.match(accepted.body.qr_reference, /^[A-Za-z0-9_-]{43}$/);
assert.match(accepted.cookie, /^ihuza_session=[A-Za-z0-9_-]{43}; HttpOnly; SameSite=Strict; Path=\/api; Max-Age=43200$/);
const cookie = accepted.cookie.split(';')[0];
const session = await fetch(`${api}/session`, { headers: { cookie } });
assert.equal(session.status, 200);
assert.equal(session.headers.get('cache-control'), 'no-store');
const sessionBody = await session.json();
assert.equal(sessionBody.account_id, accepted.body.account_id);
assert.equal(sessionBody.stellar_address, address);
const foreignLogout = await fetch(`${api}/session/logout`, { method: 'POST', headers: { cookie, origin: 'https://wrong.example' } });
assert.equal(foreignLogout.status, 403);
const logout = await fetch(`${api}/session/logout`, { method: 'POST', headers: { cookie, origin: 'http://127.0.0.1:5173' } });
assert.equal(logout.status, 200);
assert.match(logout.headers.get('set-cookie'), /Max-Age=0$/);
assert.equal((await fetch(`${api}/session`, { headers: { cookie } })).status, 401);
assert.equal((await post('/wallet/proofs', {
  challenge_id: first.body.challenge_id,
  signature_base64: signed,
})).status, 409);

const second = await post('/wallet/challenges', { address });
assert.equal(second.status, 200, JSON.stringify(second.body));
const again = await post('/wallet/proofs', {
  challenge_id: second.body.challenge_id,
  signature_base64: signature(second.body.message, privateKey),
});
assert.equal(again.status, 200, JSON.stringify(again.body));
assert.equal(again.body.account_id, accepted.body.account_id);
assert.equal(again.body.qr_reference, accepted.body.qr_reference);

const concurrentChallenge = await post('/wallet/challenges', { address });
assert.equal(concurrentChallenge.status, 200, JSON.stringify(concurrentChallenge.body));
const concurrentProof = {
  challenge_id: concurrentChallenge.body.challenge_id,
  signature_base64: signature(concurrentChallenge.body.message, privateKey),
};
const concurrent = await Promise.all([
  post('/wallet/proofs', concurrentProof),
  post('/wallet/proofs', concurrentProof),
]);
assert.deepEqual(concurrent.map((result) => result.status).sort(), [200, 409]);

console.log('Wallet API integration check passed: issue, reject, accept, session, logout, replay, concurrent replay, stable account QR.');
