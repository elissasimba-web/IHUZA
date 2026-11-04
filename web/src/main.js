import { getAddress, getNetworkDetails, isAllowed, isConnected, requestAccess } from '@stellar/freighter-api';
import { createWalletService } from './wallet.js';
import './styles.css';

const service = createWalletService({ getAddress, getNetworkDetails, isAllowed, isConnected, requestAccess });
const badge = document.querySelector('#status-badge');
const message = document.querySelector('#wallet-message');
const details = document.querySelector('#wallet-details');
const address = document.querySelector('#wallet-address');
const network = document.querySelector('#wallet-network');
const connectButton = document.querySelector('#connect-button');
const refreshButton = document.querySelector('#refresh-button');

const messages = {
  missing: ['Freighter ntiboneka', 'Shyira Freighter extension muri browser, hanyuma wongere ugenzure.'],
  not_allowed: ['Tegereje uburenganzira', 'Kanda “Huza Freighter” kugira ngo wallet ikwereke aderesi yayo rusange.'],
  wrong_network: ['Network itari yo', 'Hindura Freighter ijye kuri Stellar testnet, hanyuma wongere ugenzure. Nta kindi gikorwa gikomeza kuri iyi network.'],
  ready: ['Testnet yemejwe', 'Aderesi ya wallet yabonetse. Guhuza na konti ya IHUZA+ no kwishyura ntibirubakwa.'],
  error: ['Habaye ikibazo', ''],
};

function render(state) {
  const [label, defaultMessage] = messages[state.status] || messages.error;
  badge.textContent = label;
  badge.dataset.status = state.status;
  message.textContent = state.message || defaultMessage;
  const showDetails = state.status === 'ready' || state.status === 'wrong_network';
  details.hidden = !showDetails;
  address.textContent = showDetails ? state.address : '';
  network.textContent = showDetails ? state.network : '';
  connectButton.disabled = state.status === 'missing' || state.status === 'ready' || state.status === 'wrong_network';
}

let busy = false;
async function check(method) {
  if (busy) return;
  busy = true;
  connectButton.disabled = true;
  refreshButton.disabled = true;
  badge.textContent = 'Biragenzurwa';
  badge.dataset.status = 'checking';
  message.textContent = method === 'connect' ? 'Tegereza uburenganzira muri Freighter...' : 'Turagenzura wallet na network...';
  details.hidden = true;
  address.textContent = '';
  network.textContent = '';
  try {
    render(await service[method]());
  } finally {
    busy = false;
    refreshButton.disabled = false;
  }
}

connectButton.addEventListener('click', () => check('connect'));
refreshButton.addEventListener('click', () => check('inspect'));
window.addEventListener('focus', () => { if (!busy) check('inspect'); });
check('inspect');
