export const TESTNET_PASSPHRASE = 'Test SDF Network ; September 2015';
export const WEB_ORIGIN = 'http://127.0.0.1:5173';

function apiError(result) {
  return result?.error?.message || 'Wallet ntiyashoboye gusubiza neza.';
}

function addressIsPlausible(address) {
  return typeof address === 'string' && /^G[A-Z2-7]{55}$/.test(address);
}

export function parseOwnershipMessage(message, address) {
  if (typeof message !== 'string' || !addressIsPlausible(address)) return null;
  const lines = message.split('\n');
  if (lines.length !== 9 || lines[0] !== 'IHUZA+ wallet ownership' || lines[1] !== 'version: 2') return null;
  if (lines[2] !== `origin: ${WEB_ORIGIN}` || lines[3] !== 'purpose: wallet ownership only; no payment') return null;
  if (lines[4] !== `network: ${TESTNET_PASSPHRASE}` || lines[5] !== `address: ${address}`) return null;
  if (!/^nonce: [0-9a-f]{64}$/.test(lines[6])) return null;
  if (!/^issued_at: [0-9]+$/.test(lines[7]) || !/^expires_at: [0-9]+$/.test(lines[8])) return null;
  const issued_at = Number(lines[7].slice('issued_at: '.length));
  const expires_at = Number(lines[8].slice('expires_at: '.length));
  if (!Number.isSafeInteger(issued_at) || !Number.isSafeInteger(expires_at) ||
      issued_at <= 0 || expires_at <= issued_at || expires_at - issued_at > 300) return null;
  return { issued_at, expires_at };
}

export function parseMerchantReviewMessage(message, address) {
  if (typeof message !== 'string' || !addressIsPlausible(address)) return null;
  const lines = message.split('\n');
  if (lines.length !== 13 || lines[0] !== 'IHUZA+ merchant review' || lines[1] !== 'version: 1') return null;
  if (lines[2] !== `origin: ${WEB_ORIGIN}` || lines[3] !== 'purpose: approve merchant application only; no payment') return null;
  if (lines[4] !== `network: ${TESTNET_PASSPHRASE}` || lines[5] !== `reviewer: ${address}`) return null;
  if (!/^account: [0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(lines[6])) return null;
  if (!/^name_sha256: [0-9a-f]{64}$/.test(lines[7]) || !/^reviewer_set_sha256: [0-9a-f]{64}$/.test(lines[8])) return null;
  if (lines[9] !== 'decision: approve' || !/^nonce: [0-9a-f]{64}$/.test(lines[10])) return null;
  if (!/^issued_at: [0-9]+$/.test(lines[11]) || !/^expires_at: [0-9]+$/.test(lines[12])) return null;
  const issued_at = Number(lines[11].slice('issued_at: '.length));
  const expires_at = Number(lines[12].slice('expires_at: '.length));
  if (!Number.isSafeInteger(issued_at) || !Number.isSafeInteger(expires_at) ||
      issued_at <= 0 || expires_at <= issued_at || expires_at - issued_at > 300) return null;
  return {
    account_id: lines[6].slice('account: '.length),
    name_hash: lines[7].slice('name_sha256: '.length),
    reviewer_set_hash: lines[8].slice('reviewer_set_sha256: '.length),
    nonce_hex: lines[10].slice('nonce: '.length),
    issued_at,
    expires_at,
  };
}

function bytesToBase64(bytes) {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function signatureBase64(value) {
  if (value instanceof Uint8Array) {
    return value.length === 64 ? bytesToBase64(value) : null;
  }
  if (typeof value !== 'string') return null;
  const hex = value.startsWith('0x') ? value.slice(2) : value;
  if (/^[0-9a-fA-F]{128}$/.test(hex)) {
    return bytesToBase64(Uint8Array.from(hex.match(/../g), (part) => Number.parseInt(part, 16)));
  }
  if (!/^[A-Za-z0-9+/]{86}==$/.test(value)) return null;
  try {
    return btoa(atob(value)) === value && atob(value).length === 64 ? value : null;
  } catch {
    return null;
  }
}

export function createWalletService(api) {
  async function read({ requestPermission }) {
    try {
      const connected = await api.isConnected();
      if (connected.error) return { status: 'error', message: apiError(connected) };
      if (!connected.isConnected) return { status: 'missing' };

      let addressResult;
      if (requestPermission) {
        addressResult = await api.requestAccess();
      } else {
        const allowed = await api.isAllowed();
        if (allowed.error) return { status: 'error', message: apiError(allowed) };
        if (!allowed.isAllowed) return { status: 'not_allowed' };
        addressResult = await api.getAddress();
      }
      if (addressResult.error) return { status: 'error', message: apiError(addressResult) };
      if (!addressIsPlausible(addressResult.address)) return { status: 'error', message: 'Aderesi ya wallet ntiyemewe.' };

      const network = await api.getNetworkDetails();
      if (network.error) return { status: 'error', message: apiError(network) };
      if (network.networkPassphrase !== TESTNET_PASSPHRASE) {
        return { status: 'wrong_network', address: addressResult.address, network: network.network || 'Unknown' };
      }
      return { status: 'ready', address: addressResult.address, network: 'Stellar testnet' };
    } catch {
      return { status: 'error', message: 'Ntitwabashije kugenzura Freighter. Ongera ugerageze.' };
    }
  }

  async function signValidatedMessage({ address, message, validate }) {
    if (!validate(message, address)) return { status: 'error', message: 'Ubutumwa bwo gusinya ntibwemewe.' };
    const before = await read({ requestPermission: false });
    if (before.status !== 'ready' || before.address !== address) {
      return { status: 'error', message: 'Wallet cyangwa network byahindutse. Ongera ugenzure.' };
    }
    try {
      const result = await api.signMessage(message, { networkPassphrase: TESTNET_PASSPHRASE, address });
      if (result.error) return { status: 'error', message: 'Umukono ntiwemejwe muri Freighter.' };
      if (result.signerAddress !== address) return { status: 'error', message: 'Aderesi yasinyiye si wallet wahisemo.' };
      const signature = signatureBase64(result.signedMessage);
      if (!signature) return { status: 'error', message: 'Umukono wa Freighter ufite format itemewe.' };
      const after = await read({ requestPermission: false });
      if (after.status !== 'ready' || after.address !== address) {
        return { status: 'error', message: 'Wallet cyangwa network byahindutse. Ongera ugenzure.' };
      }
      return { status: 'signed', signature_base64: signature };
    } catch {
      return { status: 'error', message: 'Ntitwabashije gusinyisha Freighter. Ongera ugerageze.' };
    }
  }

  return {
    inspect: () => read({ requestPermission: false }),
    connect: () => read({ requestPermission: true }),
    signChallenge: ({ address, message }) => signValidatedMessage({ address, message, validate: parseOwnershipMessage }),
    signMerchantReview: ({ address, message }) => signValidatedMessage({ address, message, validate: parseMerchantReviewMessage }),
  };
}
