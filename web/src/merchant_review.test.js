import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createMerchantReviewService } from './merchant_review.js';
import { TESTNET_PASSPHRASE, WEB_ORIGIN } from './wallet.js';

test('reviewer status is a server decision and malformed responses fail closed', async () => {
  const wallet = { inspect: async () => ({ status: 'ready', address: 'G'.repeat(56) }) };
  const eligible = createMerchantReviewService(wallet, async (url, options) => {
    assert.equal(url, '/api/reviewer-status');
    assert.equal(options.credentials, 'same-origin');
    return { ok: true, json: async () => ({ eligible: true }) };
  });
  assert.equal(await eligible.status(), true);
  const malformed = createMerchantReviewService(wallet, async () => ({ ok: true, json: async () => ({ eligible: 'true' }) }));
  await assert.rejects(() => malformed.status());
});
const reference = 'a'.repeat(43);
const address = 'G' + 'A'.repeat(55);
const name = 'Iduka Ryawe';
const accountId = '11111111-1111-4111-8111-111111111111';
const profileId = '22222222-2222-4222-8222-222222222222';
const hash = createHash('sha256').update([profileId, accountId, name, 'Aline Uwase', 'Iduka Ltd'].join('\n')).digest('hex');
const review = {
  proposed_name: name,
  applicant_name: 'Aline Uwase', company_name: 'Iduka Ltd', profile_id: profileId,
  nonce_hex: 'b'.repeat(64), issued_at: 1000, expires_at: 1300,
  message: `IHUZA+ merchant profile review\nversion: 2\norigin: ${WEB_ORIGIN}\npurpose: approve testnet merchant profile only; no KYC or payment\nnetwork: ${TESTNET_PASSPHRASE}\nreviewer: ${address}\naccount: ${accountId}\nprofile: ${profileId}\nprofile_sha256: ${hash}\nreviewer_set_sha256: ${'c'.repeat(64)}\ndecision: approve\nnonce: ${'b'.repeat(64)}\nissued_at: 1000\nexpires_at: 1300`,
};
const wallet = { inspect: async () => ({ status: 'ready', address }), signMerchantReview: async () => ({ status: 'signed', signature_base64: 'x'.repeat(88) }) };

test('reviewer sees exact application and message before explicit signing', async () => {
  const calls = [];
  const service = createMerchantReviewService(wallet, async (path, options) => {
    calls.push({ path, options });
    if (options.method === 'POST') return { ok: true, json: async () => ({ votes: 2, threshold: 2, approved: true }) };
    return { ok: true, json: async () => review };
  }, () => 1100);
  const prepared = await service.prepare(reference, address);
  assert.equal(calls.length, 1);
  assert.equal(prepared.message, review.message);
  assert.deepEqual(await service.submit(prepared), { votes: 2, threshold: 2, approved: true });
  assert.equal(calls[1].options.credentials, 'same-origin');
  assert.equal(JSON.parse(calls[1].options.body).nonce_hex, review.nonce_hex);
});

test('changed name hash or expired review cannot be signed', async () => {
  let signed = false;
  const protectedWallet = { ...wallet, signMerchantReview: async () => { signed = true; return {}; } };
  const changed = createMerchantReviewService(protectedWallet, async () => ({ ok: true, json: async () => ({ ...review, proposed_name: 'Irindi Duka' }) }), () => 1100);
  await assert.rejects(() => changed.prepare(reference, address));
  const changedApplicant = createMerchantReviewService(protectedWallet, async () => ({ ok: true, json: async () => ({ ...review, applicant_name: 'Irindi Zina' }) }), () => 1100);
  await assert.rejects(() => changedApplicant.prepare(reference, address));
  const changedCompany = createMerchantReviewService(protectedWallet, async () => ({ ok: true, json: async () => ({ ...review, company_name: null }) }), () => 1100);
  await assert.rejects(() => changedCompany.prepare(reference, address));
  const expired = createMerchantReviewService(protectedWallet, async () => ({ ok: true, json: async () => review }), () => 1300);
  await assert.rejects(() => expired.prepare(reference, address));
  assert.equal(signed, false);
});
