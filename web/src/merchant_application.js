const namePattern = /^[^\p{Cc}\p{Cf}]{2,80}$/u;

export function validMerchantName(name) {
  return typeof name === 'string' && name === name.trim() && namePattern.test(name);
}

export function createMerchantApplicationService(fetchImpl = fetch) {
  const url = '/api/merchant-application';
  return {
    async current() {
      const response = await fetchImpl(url, { credentials: 'same-origin', cache: 'no-store' });
      if (response.status === 404) return null;
      if (!response.ok) throw new Error('merchant request unavailable');
      const result = await response.json();
      if (result?.status !== 'pending' || !validMerchantName(result.proposed_name)) throw new Error('invalid merchant request');
      return result;
    },
    async submit(proposedName) {
      if (!validMerchantName(proposedName)) throw new Error('invalid merchant name');
      const response = await fetchImpl(url, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        credentials: 'same-origin',
        cache: 'no-store',
        body: JSON.stringify({ proposed_name: proposedName }),
      });
      if (!response.ok) throw new Error('merchant request rejected');
      const result = await response.json();
      if (result?.status !== 'pending' || result.proposed_name !== proposedName) throw new Error('invalid merchant response');
      return result;
    },
  };
}
