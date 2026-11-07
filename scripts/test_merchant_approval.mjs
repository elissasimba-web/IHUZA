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
  assert.equal((await request(3001, '/reviewer-status')).status, 401);
  assert.deepEqual(await (await request(3001, '/reviewer-status', { cookie: reviewerOne.cookie })).json(), { eligible: true });
  assert.deepEqual(await (await request(3001, '/reviewer-status', { cookie: outsiderSession.cookie })).json(), { eligible: false });
  const application = await request(3001, '/merchant-application', {
    method: 'POST', cookie: applicant.cookie, body: { proposed_name: 'Iduka Ryawe', applicant_name: 'Aline Uwase', company_name: 'Iduka Ltd' },
  });
  assert.equal(application.status, 201);
  assert.equal((await request(3001, '/merchant-products', { method: 'POST', cookie: applicant.cookie,
    body: { name: 'Umuceri', unit_price: '1000.00' } })).status, 403);
  assert.equal((await request(3001, `/merchant-products/${applicant.qr_reference}`)).status, 403);
  const target = `/merchant-review/${applicant.qr_reference}`;
  assert.equal((await request(3001, target, { cookie: outsiderSession.cookie })).status, 403);
  assert.equal((await request(3001, target, { cookie: applicant.cookie })).status, 403);

  const firstMessage = await request(3001, target, { cookie: reviewerOne.cookie });
  assert.equal(firstMessage.status, 200);
  const first = await firstMessage.json();
  assert.equal(first.proposed_name, 'Iduka Ryawe');
  assert.equal(first.applicant_name, 'Aline Uwase');
  assert.equal(first.company_name, 'Iduka Ltd');
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
    { proposed_name: 'Iduka Ryawe', applicant_name: 'Aline Uwase', company_name: 'Iduka Ltd', status: 'approved' });
  assert.equal((await request(3001, '/merchant-products', { method: 'POST', cookie: applicant.cookie,
    body: { name: 'Umuceri', unit_price: '0.00' } })).status, 400);
  assert.equal((await request(3001, '/merchant-products', { method: 'POST', cookie: reviewerOne.cookie,
    body: { name: 'Umuceri', unit_price: '1000.00' } })).status, 403);
  const created = await request(3001, '/merchant-products', { method: 'POST', cookie: applicant.cookie,
    body: { name: 'Umuceri', unit_price: '1000.50' } });
  assert.equal(created.status, 201);
  const product = await created.json();
  assert.match(product.id, /^[0-9a-f-]{36}$/);
  assert.deepEqual({ name: product.name, unit_price: product.unit_price, currency_code: product.currency_code },
    { name: 'Umuceri', unit_price: '1000.50', currency_code: 'RWF' });
  assert.deepEqual(await (await request(3001, `/merchant-products/${applicant.qr_reference}`)).json(), [product]);

  const smallProductResponse = await request(3001, '/merchant-products', { method: 'POST', cookie: applicant.cookie,
    body: { name: 'Agapaki', unit_price: '0.01' } });
  assert.equal(smallProductResponse.status, 201);
  const smallProduct = await smallProductResponse.json();
  const quotePath = '/quote-preview';
  const quoteBody = { merchant_reference: applicant.qr_reference, items: [
    { product_id: product.id, quantity: 3 }, { product_id: smallProduct.id, quantity: 2 },
  ] };
  assert.equal((await request(3001, quotePath, { method: 'POST', body: quoteBody })).status, 401);
  assert.equal((await request(3001, quotePath, { method: 'POST', cookie: outsiderSession.cookie,
    body: { ...quoteBody, total: '0.01' } })).status, 422);
  assert.equal((await request(3001, quotePath, { method: 'POST', cookie: outsiderSession.cookie,
    body: { ...quoteBody, items: [quoteBody.items[0], quoteBody.items[0]] } })).status, 400);
  assert.equal((await request(3001, quotePath, { method: 'POST', cookie: outsiderSession.cookie,
    body: { ...quoteBody, items: [{ product_id: product.id, quantity: 0 }] } })).status, 400);
  const previewResponse = await request(3001, quotePath, { method: 'POST', cookie: outsiderSession.cookie, body: quoteBody });
  assert.equal(previewResponse.status, 200);
  assert.equal(previewResponse.headers.get('cache-control'), 'no-store');
  const preview = await previewResponse.json();
  assert.equal(preview.status, 'preview_only');
  assert.equal(preview.currency_code, 'RWF');
  assert.equal(preview.total, '3001.52');
  assert.deepEqual(preview.lines.map((line) => line.line_total), ['3001.50', '0.02']);
  const invoiceBody = { ...quoteBody, idempotency_key: 'a'.repeat(64) };
  assert.equal((await request(3001, '/invoice-drafts', { method: 'POST', body: invoiceBody })).status, 401);
  const createdDraftResponse = await request(3001, '/invoice-drafts', { method: 'POST', cookie: outsiderSession.cookie, body: invoiceBody });
  assert.equal(createdDraftResponse.status, 201);
  const draft = await createdDraftResponse.json();
  assert.equal(draft.status, 'draft');
  assert.equal(draft.total, '3001.52');
  assert.deepEqual(draft.lines.map((line) => line.line_total), ['3001.50', '0.02']);
  const repeatedDraftResponse = await request(3001, '/invoice-drafts', { method: 'POST', cookie: outsiderSession.cookie, body: invoiceBody });
  assert.equal(repeatedDraftResponse.status, 200);
  assert.deepEqual(await repeatedDraftResponse.json(), draft);
  assert.equal((await request(3001, '/invoice-drafts', { method: 'POST', cookie: outsiderSession.cookie,
    body: { ...invoiceBody, items: [{ product_id: product.id, quantity: 1 }] } })).status, 409);
  assert.deepEqual(await (await request(3001, `/invoice-drafts/${draft.id}`, { cookie: outsiderSession.cookie })).json(), draft);
  assert.equal((await request(3001, `/invoice-drafts/${draft.id}`, { cookie: applicant.cookie })).status, 404);

  rotatedApi = startApi(3002, [reviewers[0].address, reviewers[1].address, outsider.address]);
  await waitForApi(3002, rotatedApi);
  assert.deepEqual(await (await request(3002, '/reviewer-status', { cookie: outsiderSession.cookie })).json(), { eligible: true });
  assert.deepEqual(await (await request(3002, `/qr/${applicant.qr_reference}`)).json(),
    { account_kind: 'person', flow: 'person_transfer_preview' });
  assert.equal((await request(3002, `/merchant-products/${applicant.qr_reference}`)).status, 403);
  assert.equal((await request(3002, quotePath, { method: 'POST', cookie: outsiderSession.cookie, body: quoteBody })).status, 403);
  assert.deepEqual(await (await request(3002, `/invoice-drafts/${draft.id}`, { cookie: outsiderSession.cookie })).json(), draft);
  console.log('Merchant review and catalog check passed: quorum, signer gate, replay, price validation and reviewer-set rotation.');
} finally {
  rotatedApi?.kill();
  api?.kill();
}
