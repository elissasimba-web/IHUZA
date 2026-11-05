// Local testnet integration check. Reviewer keys exist only in this process.
import assert from 'node:assert/strict';
import { createHash, generateKeyPairSync, sign } from 'node:crypto';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const executable = join(root, 'target', 'debug', process.platform === 'win32' ? 'ihuza-api.exe' : 'ihuza-api');
const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';

function base32(bytes) {
  let bits = 0, value = 0, output = '';
  for (const byte of bytes) {
    value = (value << 8) | byte;
    bits += 8;
    while (bits >= 5) output += alphabet[(value >>> (bits -= 5)) & 31];
  }
  if (bits) output += alphabet[(value << (5 - bits)) & 31];
  return output;
}

function address(publicKey) {
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

function signature(message, privateKey) {
  const digest = createHash('sha256').update('Stellar Signed Message:\n').update(message).digest();
  return sign(null, digest, privateKey).toString('base64');
}

const reviewers = Array.from({ length: 3 }, () => {
  const keys = generateKeyPairSync('ed25519');
  return { ...keys, address: address(keys.publicKey) };
});
const outsiderKeys = generateKeyPairSync('ed25519');
const outsider = { ...outsiderKeys, address: address(outsiderKeys.publicKey) };

function startApi(port, addresses) {
  return spawn(executable, [], {
    cwd: root,
    env: { ...process.env, IHUZA_API_BIND: `127.0.0.1:${port}`, IHUZA_REVIEWER_ADDRESSES: addresses.join(',') },
    stdio: 'ignore',
  });
}

async function waitForApi(port, child) {
  for (let attempt = 0; attempt < 50; attempt++) {
    if (child.exitCode !== null) throw new Error('review API stopped before readiness');
    try {
      const response = await fetch(`http://127.0.0.1:${port}/api/session`, { signal: AbortSignal.timeout(500) });
      if (response.status === 401) return;
    } catch { /* wait for local API */ }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error('review API did not start');
}

async function request(port, path, { method = 'GET', cookie, body } = {}) {
  const headers = { origin: 'http://127.0.0.1:5173' };
  if (cookie) headers.cookie = cookie;
  if (body) headers['content-type'] = 'application/json';
  return fetch(`http://127.0.0.1:${port}/api${path}`, {
    method, headers, body: body ? JSON.stringify(body) : undefined,
  });
}

async function login(port, reviewer) {
  const issued = await request(port, '/wallet/challenges', { method: 'POST', body: { address: reviewer.address } });
  assert.equal(issued.status, 200);
  const challenge = await issued.json();
  const proof = await request(port, '/wallet/proofs', {
    method: 'POST', body: { challenge_id: challenge.challenge_id, signature_base64: signature(challenge.message, reviewer.privateKey) },
  });
  assert.equal(proof.status, 200);
  return { ...(await proof.json()), cookie: proof.headers.get('set-cookie').split(';')[0] };
}

let api;
let rotatedApi;
try {
  api = startApi(3001, reviewers.map((reviewer) => reviewer.address));
  await waitForApi(3001, api);
  const reviewerOne = await login(3001, reviewers[0]);
  const reviewerTwo = await login(3001, reviewers[1]);
  const applicant = await login(3001, reviewers[2]);
  const outsiderSession = await login(3001, outsider);
  const application = await request(3001, '/merchant-application', {
    method: 'POST', cookie: applicant.cookie, body: { proposed_name: 'Iduka Ryawe' },
  });
  assert.equal(application.status, 201);
  const target = `/merchant-review/${applicant.qr_reference}`;
  assert.equal((await request(3001, target, { cookie: outsiderSession.cookie })).status, 403);
  assert.equal((await request(3001, target, { cookie: applicant.cookie })).status, 403);

  const firstMessage = await request(3001, target, { cookie: reviewerOne.cookie });
  assert.equal(firstMessage.status, 200);
  const first = await firstMessage.json();
  assert.equal(first.proposed_name, 'Iduka Ryawe');
  assert.ok(first.message.includes('decision: approve\n'));
  const firstBody = { nonce_hex: first.nonce_hex, issued_at: first.issued_at, expires_at: first.expires_at,
    signature_base64: signature(first.message, reviewers[0].privateKey) };
  assert.equal((await request(3001, target, { method: 'POST', cookie: reviewerOne.cookie,
    body: { ...firstBody, signature_base64: signature(first.message, reviewers[1].privateKey) } })).status, 401);
  const firstVote = await request(3001, target, { method: 'POST', cookie: reviewerOne.cookie, body: firstBody });
  assert.equal(firstVote.status, 200);
  assert.deepEqual(await firstVote.json(), { votes: 1, threshold: 2, approved: false });
  assert.equal((await request(3001, target, { method: 'POST', cookie: reviewerOne.cookie, body: firstBody })).status, 409);
  assert.deepEqual(await (await request(3001, `/qr/${applicant.qr_reference}`)).json(),
    { account_kind: 'person', flow: 'person_transfer_preview' });

  const secondMessage = await request(3001, target, { cookie: reviewerTwo.cookie });
  assert.equal(secondMessage.status, 200);
  const second = await secondMessage.json();
  const secondVote = await request(3001, target, { method: 'POST', cookie: reviewerTwo.cookie, body: {
    nonce_hex: second.nonce_hex, issued_at: second.issued_at, expires_at: second.expires_at,
    signature_base64: signature(second.message, reviewers[1].privateKey),
  } });
  assert.equal(secondVote.status, 200);
  assert.deepEqual(await secondVote.json(), { votes: 2, threshold: 2, approved: true });
  assert.deepEqual(await (await request(3001, `/qr/${applicant.qr_reference}`)).json(),
    { account_kind: 'merchant', flow: 'unavailable' });
  assert.deepEqual(await (await request(3001, '/merchant-application', { cookie: applicant.cookie })).json(),
    { proposed_name: 'Iduka Ryawe', status: 'approved' });

  rotatedApi = startApi(3002, [reviewers[0].address, reviewers[1].address, outsider.address]);
  await waitForApi(3002, rotatedApi);
  assert.deepEqual(await (await request(3002, `/qr/${applicant.qr_reference}`)).json(),
    { account_kind: 'person', flow: 'person_transfer_preview' });
  console.log('Merchant 2-of-3 review check passed: signer gate, self-approval, replay, quorum and reviewer-set rotation.');
} finally {
  rotatedApi?.kill();
  api?.kill();
}
