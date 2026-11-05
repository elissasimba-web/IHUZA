const namePattern = /^[^\p{Cc}\p{Cf}]{2,80}$/u;
const applicantPattern = /^[^\p{Cc}\p{Cf}]{2,100}$/u;
const companyPattern = /^[^\p{Cc}\p{Cf}]{2,120}$/u;

export function validMerchantName(name) {
  return typeof name === 'string' && name === name.trim() && namePattern.test(name);
}

function validProfileName(name, pattern) {
  return typeof name === 'string' && name === name.trim() && pattern.test(name);
}

export function createMerchantApplicationService(fetchImpl = fetch) {
  const url = '/api/merchant-application';
  return {
    async current() {
      const response = await fetchImpl(url, { credentials: 'same-origin', cache: 'no-store' });
      if (response.status === 404) return null;
      if (!response.ok) throw new Error('merchant request unavailable');
      const result = await response.json();
      if (!['needs_details', 'pending', 'approved'].includes(result?.status) || !validMerchantName(result.proposed_name)) throw new Error('invalid merchant request');
      if (result.status !== 'needs_details' && (!validProfileName(result.applicant_name, applicantPattern) ||
          (result.company_name !== null && !validProfileName(result.company_name, companyPattern)))) throw new Error('invalid merchant profile');
      return result;
    },
    async submit(proposedName, applicantName, companyName = null) {
      if (!validMerchantName(proposedName) || !validProfileName(applicantName, applicantPattern) ||
          (companyName !== null && !validProfileName(companyName, companyPattern))) throw new Error('invalid merchant profile');
      const response = await fetchImpl(url, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        credentials: 'same-origin',
        cache: 'no-store',
        body: JSON.stringify({ proposed_name: proposedName, applicant_name: applicantName, company_name: companyName }),
      });
      if (!response.ok) throw new Error('merchant request rejected');
      const result = await response.json();
      if (!['pending', 'approved'].includes(result?.status) || result.proposed_name !== proposedName ||
          result.applicant_name !== applicantName || result.company_name !== companyName) throw new Error('invalid merchant response');
      return result;
    },
  };
}
