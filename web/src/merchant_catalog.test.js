import test from 'node:test';
import assert from 'node:assert/strict';
import { createMerchantCatalogService, validProductInput } from './merchant_catalog.js';

const reference = 'A'.repeat(43);

test('merchant sets a positive display price and catalog uses one account reference', async () => {
  const calls = [];
  const product = { id: '11111111-1111-4111-8111-111111111111', name: 'Umuceri', unit_price: '1000.50', currency_code: 'RWF' };
  const service = createMerchantCatalogService(async (url, options) => {
    calls.push([url, options]);
    return { status: options.method === 'POST' ? 201 : 200, ok: true, json: async () => options.method === 'POST' ? product : [product] };
  });
  assert.equal(validProductInput('Umuceri', '1000.5'), true);
  assert.equal(validProductInput('Umuceri', '0.00'), false);
  assert.equal(validProductInput('Umuceri', '1e3'), false);
  assert.deepEqual(await service.create('Umuceri', '1000.50'), product);
  assert.deepEqual(await service.list(reference), [product]);
  assert.equal(calls[0][0], '/api/merchant-products');
  assert.equal(calls[1][0], `/api/merchant-products/${reference}`);
});

test('quote preview accepts only server-calculated lines and total', async () => {
  const productId = '11111111-1111-4111-8111-111111111111';
  const items = [{ product_id: productId, quantity: 3 }];
  const preview = { status: 'preview_only', merchant_reference: reference, currency_code: 'RWF',
    lines: [{ product_id: productId, name: 'Umuceri', quantity: 3, unit_price: '1000.50', line_total: '3001.50' }], total: '3001.50' };
  const calls = [];
  const service = createMerchantCatalogService(async (url, options) => {
    calls.push({ url, options });
    return { ok: true, json: async () => preview };
  });
  assert.deepEqual(await service.preview(reference, items), preview);
  assert.equal(calls[0].url, '/api/quote-preview');
  assert.equal(calls[0].options.credentials, 'same-origin');
  assert.deepEqual(JSON.parse(calls[0].options.body), { merchant_reference: reference, items });
  await assert.rejects(() => service.preview(reference, [...items, ...items]));
  assert.equal(calls.length, 1);
  preview.total = '1.00';
  await assert.rejects(() => service.preview(reference, items));
});
