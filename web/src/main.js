import { getAddress, getNetworkDetails, isAllowed, isConnected, requestAccess, signMessage } from '@stellar/freighter-api';
import { createWalletService } from './wallet.js';
import { createOwnershipService } from './ownership.js';
import './styles.css';

const service = createWalletService({ getAddress, getNetworkDetails, isAllowed, isConnected, requestAccess, signMessage });
const ownership = createOwnershipService(service);
const badge = document.querySelector('#status-badge');
const message = document.querySelector('#wallet-message');
const details = document.querySelector('#wallet-details');
const address = document.querySelector('#wallet-address');
const network = document.querySelector('#wallet-network');
const connectButton = document.querySelector('#connect-button');
const refreshButton = document.querySelector('#refresh-button');
const ownershipPanel = document.querySelector('#ownership-panel');
const prepareButton = document.querySelector('#prepare-button');
const challengePreview = document.querySelector('#challenge-preview');
const challengeMessage = document.querySelector('#challenge-message');
const signButton = document.querySelector('#sign-button');
const ownershipMessage = document.querySelector('#ownership-message');
const ownershipResult = document.querySelector('#ownership-result');
const accountReference = document.querySelector('#account-reference');
const logoutButton = document.querySelector('#logout-button');

const messages = {
  missing: ['Freighter ntiboneka', 'Shyira Freighter extension muri browser, hanyuma wongere ugenzure.'],
  not_allowed: ['Tegereje uburenganzira', 'Kanda “Huza Freighter” kugira ngo wallet ikwereke aderesi yayo rusange.'],
  wrong_network: ['Network itari yo', 'Hindura Freighter ijye kuri Stellar testnet, hanyuma wongere ugenzure. Nta kindi gikorwa gikomeza kuri iyi network.'],
  ready: ['Testnet yemejwe', 'Aderesi ya wallet yabonetse. Ushobora kwemeza urufunguzo rwawe ukoresheje ubutumwa bwa testnet.'],
  error: ['Habaye ikibazo', ''],
};

let activeAddress = null;
let activeChallenge = null;
let busy = false;

function clearOwnership() {
  activeChallenge = null;
  challengePreview.hidden = true;
  challengeMessage.textContent = '';
  signButton.disabled = true;
  ownershipResult.hidden = true;
  accountReference.textContent = '';
  ownershipMessage.textContent = '';
}

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
  if (state.status !== 'ready' || state.address !== activeAddress) {
    clearOwnership();
    activeAddress = state.status === 'ready' ? state.address : null;
  }
  ownershipPanel.hidden = state.status !== 'ready';
  prepareButton.disabled = state.status !== 'ready';
}

async function restoreSession() {
  if (!activeAddress) return;
  const currentAddress = activeAddress;
  const session = await ownership.session(currentAddress);
  if (currentAddress !== activeAddress) return;
  if (!session) {
    if (!ownershipResult.hidden) ownershipMessage.textContent = 'Session yarangiye. Ongera wemeze wallet yawe.';
    ownershipResult.hidden = true;
    accountReference.textContent = '';
    return;
  }
  accountReference.textContent = session.qr_reference;
  ownershipResult.hidden = false;
  ownershipMessage.textContent = 'Session ya testnet irakora.';
}

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
    await restoreSession();
  } finally {
    busy = false;
    refreshButton.disabled = false;
  }
}

async function prepareOwnership() {
  if (busy || !activeAddress) return;
  busy = true;
  prepareButton.disabled = true;
  signButton.disabled = true;
  clearOwnership();
  ownershipMessage.textContent = 'Turategura ubutumwa bwa testnet...';
  try {
    const result = await ownership.prepare(activeAddress);
    if (result.status === 'ready') {
      activeChallenge = result.challenge;
      challengeMessage.textContent = result.challenge.message;
      challengePreview.hidden = false;
      signButton.disabled = false;
      ownershipMessage.textContent = 'Soma ubutumwa, hanyuma ukande gusinya niba ubyemeye.';
    } else {
      ownershipMessage.textContent = result.message;
    }
  } finally {
    busy = false;
    prepareButton.disabled = false;
  }
}

async function signOwnership() {
  if (busy || !activeChallenge) return;
  busy = true;
  signButton.disabled = true;
  prepareButton.disabled = true;
  ownershipMessage.textContent = 'Soma ubutumwa muri Freighter, wemeze gusa niba buhuye n’ubwo wabonye hano.';
  try {
    const result = await ownership.submit(activeChallenge);
    if (result.status === 'verified') {
      activeChallenge = null;
      challengePreview.hidden = true;
      accountReference.textContent = result.qr_reference;
      ownershipResult.hidden = false;
      ownershipMessage.textContent = 'Umukono wagenzuwe na IHUZA+ API.';
    } else {
      clearOwnership();
      ownershipMessage.textContent = result.message;
    }
  } finally {
    busy = false;
    prepareButton.disabled = false;
  }
}

connectButton.addEventListener('click', () => check('connect'));
refreshButton.addEventListener('click', () => check('inspect'));
prepareButton.addEventListener('click', prepareOwnership);
signButton.addEventListener('click', signOwnership);
logoutButton.addEventListener('click', async () => {
  if (busy) return;
  busy = true;
  logoutButton.disabled = true;
  const done = await ownership.logout();
  if (done) {
    clearOwnership();
    ownershipMessage.textContent = 'Wasohotse muri IHUZA+.';
  } else {
    ownershipMessage.textContent = 'Ntitwabashije gufunga session. Ongera ugerageze.';
  }
  logoutButton.disabled = false;
  busy = false;
});
window.addEventListener('focus', () => { if (!busy) check('inspect'); });
check('inspect');
