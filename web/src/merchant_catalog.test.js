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
