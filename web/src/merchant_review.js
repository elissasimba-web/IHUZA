import { parseMerchantReviewMessage, WEB_ORIGIN } from './wallet.js';

const referencePattern = /^[A-Za-z0-9_-]{43}$/;

async function nameHash(name) {
  const bytes = new TextEncoder().encode(name);
  const digest = await globalThis.crypto.subtle.digest('SHA-256', bytes);
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}

async function validMessage(review, address, now) {
  if (!review || typeof review.proposed_name !== 'string' || review.proposed_name.length < 2 || review.proposed_name.length > 80) return null;
  const parsed = parseMerchantReviewMessage(review.message, address);
  if (!parsed || parsed.nonce_hex !== review.nonce_hex || parsed.issued_at !== review.issued_at || parsed.expires_at !== review.expires_at) return null;
  if (now < parsed.issued_at || now >= parsed.expires_at || parsed.name_hash !== await nameHash(review.proposed_name)) return null;
  return parsed;
}

export function createMerchantReviewService(wallet, fetchImpl = fetch, now = () => Math.floor(Date.now() / 1000)) {
  return {
    async status() {
      const response = await fetchImpl('/api/reviewer-status', { credentials: 'same-origin', cache: 'no-store' });
      if (!response.ok) throw new Error('reviewer status unavailable');
      const result = await response.json();
      if (typeof result?.eligible !== 'boolean') throw new Error('invalid reviewer status');
      return result.eligible;
    },
    async prepare(reference, address) {
      if (globalThis.location && globalThis.location.origin !== WEB_ORIGIN) throw new Error('invalid origin');
      if (!referencePattern.test(reference)) throw new Error('invalid reference');
      const current = await wallet.inspect();
      if (current.status !== 'ready' || current.address !== address) throw new Error('wallet changed');
      const response = await fetchImpl(`/api/merchant-review/${reference}`, { credentials: 'same-origin', cache: 'no-store' });
      if (!response.ok) throw new Error('review unavailable');
      const review = await response.json();
      if (!await validMessage(review, address, now())) throw new Error('invalid review message');
      return { ...review, reference, address };
    },
    async submit(review) {
      if (globalThis.location && globalThis.location.origin !== WEB_ORIGIN) throw new Error('invalid origin');
      if (!referencePattern.test(review?.reference) || !await validMessage(review, review.address, now())) throw new Error('expired review');
      const signed = await wallet.signMerchantReview({ address: review.address, message: review.message });
      if (signed.status !== 'signed') throw new Error('review not signed');
      if (!await validMessage(review, review.address, now())) throw new Error('expired review');
      const response = await fetchImpl(`/api/merchant-review/${review.reference}`, {
        method: 'POST', credentials: 'same-origin', cache: 'no-store',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ nonce_hex: review.nonce_hex, issued_at: review.issued_at,
          expires_at: review.expires_at, signature_base64: signed.signature_base64 }),
      });
      if (!response.ok) throw new Error('review rejected');
      const result = await response.json();
      if (!Number.isInteger(result?.votes) || result.votes < 1 || result.votes > 3 ||
          result.threshold !== 2 || result.approved !== (result.votes >= 2)) throw new Error('invalid review result');
      return result;
    },
  };
}
