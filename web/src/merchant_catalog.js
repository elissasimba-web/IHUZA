const referencePattern = /^[A-Za-z0-9_-]{43}$/;
const productNamePattern = /^[^\p{Cc}\p{Cf}]{2,120}$/u;
const pricePattern = /^(?:0|[1-9][0-9]{0,17})\.[0-9]{1,2}$/;

export function validProductInput(name, price) {
  return typeof name === 'string' && name === name.trim() && productNamePattern.test(name) &&
    typeof price === 'string' && pricePattern.test(price) && !/^0\.0{1,2}$/.test(price);
}

function validProduct(product) {
  return typeof product?.id === 'string' && /^[0-9a-f-]{36}$/.test(product.id) &&
    typeof product.name === 'string' && productNamePattern.test(product.name) &&
    typeof product.unit_price === 'string' && pricePattern.test(product.unit_price) &&
    product.currency_code === 'RWF';
}

function minorUnits(value) {
  if (typeof value !== 'string' || !/^(?:0|[1-9][0-9]*)\.[0-9]{2}$/.test(value)) return null;
  const [whole, fraction] = value.split('.');
  return BigInt(whole) * 100n + BigInt(fraction);
}

export function createMerchantCatalogService(fetchImpl = fetch) {
  return {
    async create(name, unitPrice) {
      if (!validProductInput(name, unitPrice)) throw new Error('invalid product');
      const response = await fetchImpl('/api/merchant-products', {
        method: 'POST', credentials: 'same-origin', cache: 'no-store',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ name, unit_price: unitPrice }),
      });
      if (response.status !== 201) throw new Error('product not created');
      const product = await response.json();
      if (!validProduct(product) || product.name !== name) throw new Error('invalid product response');
      return product;
    },
    async list(reference) {
      if (!referencePattern.test(reference)) throw new Error('invalid reference');
      const response = await fetchImpl(`/api/merchant-products/${reference}`, { cache: 'no-store' });
      if (!response.ok) throw new Error('catalog unavailable');
      const products = await response.json();
      if (!Array.isArray(products) || products.length > 100 || !products.every(validProduct)) throw new Error('invalid catalog');
      return products;
    },
    async preview(reference, items) {
      if (!referencePattern.test(reference) || !Array.isArray(items) || items.length < 1 || items.length > 20 ||
          new Set(items.map((item) => item.product_id)).size !== items.length ||
          !items.every((item) => typeof item.product_id === 'string' && /^[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$/.test(item.product_id) &&
            Number.isInteger(item.quantity) && item.quantity >= 1 && item.quantity <= 100)) throw new Error('invalid selection');
      const response = await fetchImpl('/api/quote-preview', {
        method: 'POST', credentials: 'same-origin', cache: 'no-store',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ merchant_reference: reference, items }),
      });
      if (!response.ok) throw new Error('quote preview unavailable');
      const preview = await response.json();
      if (preview?.status !== 'preview_only' || preview.merchant_reference !== reference || preview.currency_code !== 'RWF' ||
          !Array.isArray(preview.lines) || preview.lines.length !== items.length) throw new Error('invalid quote preview');
      let calculated = 0n;
      for (let index = 0; index < items.length; index++) {
        const line = preview.lines[index];
        const unit = minorUnits(line?.unit_price);
        const lineTotal = minorUnits(line?.line_total);
        if (line?.product_id !== items[index].product_id || line.quantity !== items[index].quantity ||
            typeof line.name !== 'string' || line.name.length < 2 || unit === null || lineTotal !== unit * BigInt(line.quantity)) throw new Error('invalid quote line');
        calculated += lineTotal;
      }
      if (minorUnits(preview.total) !== calculated) throw new Error('invalid quote total');
      return preview;
    },
    async createDraft(reference, items, idempotencyKey) {
      if (!referencePattern.test(reference) || !Array.isArray(items) || items.length < 1 || items.length > 20 ||
          !/^[0-9a-f]{64}$/.test(idempotencyKey)) throw new Error('invalid invoice request');
      const response = await fetchImpl('/api/invoice-drafts', {
        method: 'POST', credentials: 'same-origin', cache: 'no-store',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ merchant_reference: reference, items, idempotency_key: idempotencyKey }),
      });
      if (![200, 201].includes(response.status)) throw new Error('invoice draft unavailable');
      const draft = await response.json();
      if (draft?.status !== 'draft' || draft.merchant_reference !== reference || draft.currency_code !== 'RWF' ||
          typeof draft.id !== 'string' || !/^[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$/.test(draft.id) ||
          !Array.isArray(draft.lines) || draft.lines.length !== items.length || minorUnits(draft.total) === null) throw new Error('invalid invoice draft');
      let total = 0n;
      for (let i = 0; i < items.length; i++) {
        const line = draft.lines[i];
        const unit = minorUnits(line?.unit_price);
        const lineTotal = minorUnits(line?.line_total);
        if (line?.product_id !== items[i].product_id || line?.quantity !== items[i].quantity ||
            typeof line.name !== 'string' || unit === null || lineTotal !== unit * BigInt(line.quantity)) throw new Error('invalid invoice line');
        total += lineTotal;
      }
      if (minorUnits(draft.total) !== total) throw new Error('invalid invoice total');
      return draft;
    },
  };
}
