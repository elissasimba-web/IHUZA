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
  };
}
