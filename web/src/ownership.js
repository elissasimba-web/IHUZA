import { parseOwnershipMessage, TESTNET_PASSPHRASE, WEB_ORIGIN } from './wallet.js';

const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const addressPattern = /^G[A-Z2-7]{55}$/;
const qrPattern = /^[A-Za-z0-9_-]{43}$/;

function validChallenge(data, address, now) {
  if (!data || !uuid.test(data.challenge_id) || data.address !== address || !addressPattern.test(address)) return false;
  if (data.network_passphrase !== TESTNET_PASSPHRASE || typeof data.message !== 'string') return false;
  const parsed = parseOwnershipMessage(data.message, address);
  return parsed !== null && parsed.expires_at === data.expires_at &&
    now >= parsed.issued_at && now < parsed.expires_at;
}

export function createOwnershipService(wallet, fetchImpl = fetch, now = () => Math.floor(Date.now() / 1000)) {
  async function post(path, body) {
    const response = await fetchImpl(`/api${path}`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      credentials: 'same-origin',
      cache: 'no-store',
      body: JSON.stringify(body),
    });
    if (!response.ok) return null;
    return response.json();
  }

  return {
    async session(address) {
      try {
        const response = await fetchImpl('/api/session', { credentials: 'same-origin', cache: 'no-store' });
        if (!response.ok) return null;
        const result = await response.json();
        if (result.stellar_address !== address || !uuid.test(result.account_id) || !qrPattern.test(result.qr_reference) ||
            !Number.isSafeInteger(result.expires_at) || result.expires_at <= now()) return null;
        return result;
      } catch {
        return null;
      }
    },
    async logout() {
      try {
        const response = await fetchImpl('/api/session/logout', {
          method: 'POST', credentials: 'same-origin', cache: 'no-store',
        });
        return response.ok;
      } catch {
        return false;
      }
    },
    async prepare(address) {
      if (globalThis.location && globalThis.location.origin !== WEB_ORIGIN) {
        return { status: 'error', message: 'Fungura IHUZA+ kuri origin ya testnet yemewe.' };
      }
      const current = await wallet.inspect();
      if (current.status !== 'ready' || current.address !== address) {
        return { status: 'error', message: 'Wallet cyangwa network byahindutse. Ongera ugenzure.' };
      }
      try {
        const challenge = await post('/wallet/challenges', { address });
        if (!validChallenge(challenge, address, now())) throw new Error('invalid challenge');
        return { status: 'ready', challenge };
      } catch {
        return { status: 'error', message: 'Ntitwabashije gutegura challenge. Reba ko API ikora, wongere ugerageze.' };
      }
    },
    async submit(challenge) {
      if (globalThis.location && globalThis.location.origin !== WEB_ORIGIN) {
        return { status: 'error', message: 'Origin ya page ntiyemewe.' };
      }
      if (!validChallenge(challenge, challenge?.address, now())) {
        return { status: 'error', message: 'Challenge yarangiye cyangwa ntiyemewe. Tegura indi.' };
      }
      const signed = await wallet.signChallenge({ address: challenge.address, message: challenge.message });
      if (signed.status !== 'signed') return signed;
      if (!validChallenge(challenge, challenge.address, now())) {
        return { status: 'error', message: 'Challenge yarangiye. Tegura indi.' };
      }
      try {
        const result = await post('/wallet/proofs', {
          challenge_id: challenge.challenge_id,
          signature_base64: signed.signature_base64,
        });
        if (result?.status !== 'wallet_key_verified' || !uuid.test(result.account_id) || !qrPattern.test(result.qr_reference)) {
          throw new Error('invalid proof result');
        }
        return { status: 'verified', account_id: result.account_id, qr_reference: result.qr_reference };
      } catch {
        return { status: 'error', message: 'API ntiyemeje umukono. Ongera utegure challenge.' };
      }
    },
  };
}
