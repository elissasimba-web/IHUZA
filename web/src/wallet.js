export const TESTNET_PASSPHRASE = 'Test SDF Network ; September 2015';

function apiError(result) {
  return result?.error?.message || 'Wallet ntiyashoboye gusubiza neza.';
}

function addressIsPlausible(address) {
  return typeof address === 'string' && /^G[A-Z2-7]{55}$/.test(address);
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

  return {
    inspect: () => read({ requestPermission: false }),
    connect: () => read({ requestPermission: true }),
  };
}
