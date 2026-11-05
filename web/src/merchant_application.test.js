import test from 'node:test';
import assert from 'node:assert/strict';
import { createMerchantApplicationService, validMerchantName } from './merchant_application.js';

test('merchant application validates profile and sends it with the session cookie', async () => {
  assert.equal(validMerchantName('Iduka Ryawe'), true);
  assert.equal(validMerchantName(' A '), false);
  assert.equal(validMerchantName('A\nB'), false);
  const calls = [];
  const service = createMerchantApplicationService(async (path, options) => {
    calls.push({ path, options });
    return { ok: true, json: async () => ({ proposed_name: 'Iduka Ryawe', applicant_name: 'Aline Uwase', company_name: null, status: 'pending' }) };
  });
  assert.deepEqual(await service.submit('Iduka Ryawe', 'Aline Uwase'), { proposed_name: 'Iduka Ryawe', applicant_name: 'Aline Uwase', company_name: null, status: 'pending' });
  assert.equal(calls[0].path, '/api/merchant-application');
  assert.equal(calls[0].options.credentials, 'same-origin');
  assert.deepEqual(JSON.parse(calls[0].options.body), { proposed_name: 'Iduka Ryawe', applicant_name: 'Aline Uwase', company_name: null });
  assert.deepEqual(await service.current(), { proposed_name: 'Iduka Ryawe', applicant_name: 'Aline Uwase', company_name: null, status: 'pending' });
  assert.equal(calls[1].options.credentials, 'same-origin');
});

test('unsubmitted application and forged approval do not appear as approved', async () => {
  const missing = createMerchantApplicationService(async () => ({ status: 404 }));
  assert.equal(await missing.current(), null);
  const approved = createMerchantApplicationService(async () => ({ ok: true, json: async () => ({ proposed_name: 'Iduka', applicant_name: 'Aline Uwase', company_name: null, status: 'approved' }) }));
  assert.equal((await approved.current()).status, 'approved');
  const forged = createMerchantApplicationService(async () => ({ ok: true, json: async () => ({ proposed_name: 'Iduka', status: 'checkout_ready' }) }));
  await assert.rejects(() => forged.current());
});
