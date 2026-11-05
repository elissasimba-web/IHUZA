import test from 'node:test';
import assert from 'node:assert/strict';
import QRCode from 'qrcode';
import { qrContent, renderAccountQr, resolveAccountQr } from './account_qr.js';

const reference = 'a'.repeat(43);

test('the one account QR contains only its opaque reference', async () => {
  assert.equal(qrContent(reference), reference);
  assert.throws(() => qrContent('https://example.com/qr/' + reference));
  assert.throws(() => qrContent('a'.repeat(42)));
  const canvas = {};
  let drawn;
  await renderAccountQr(canvas, reference, async (...args) => { drawn = args; });
  assert.equal(drawn[0], canvas);
  assert.equal(drawn[1], reference);
  assert.equal(drawn[2].margin, 4);
  assert.ok(QRCode.create(reference).modules.size > 0);
});

test('resolver keeps person and merchant flows separate', async () => {
  const calls = [];
  const person = await resolveAccountQr(reference, async (path, options) => {
    calls.push({ path, options });
    return { ok: true, json: async () => ({ account_kind: 'person', flow: 'person_transfer_preview' }) };
  });
  assert.deepEqual(person, { account_kind: 'person', flow: 'person_transfer_preview' });
  assert.equal(calls[0].path, `/api/qr/${reference}`);
  assert.equal(calls[0].options.cache, 'no-store');
  const merchant = await resolveAccountQr(reference, async () => ({ ok: true, json: async () => ({ account_kind: 'merchant', flow: 'unavailable' }) }));
  assert.equal(merchant.flow, 'unavailable');
  const forged = await resolveAccountQr(reference, async () => ({ ok: true, json: async () => ({ account_kind: 'merchant', flow: 'merchant_checkout' }) }));
  assert.equal(forged, null);
});
