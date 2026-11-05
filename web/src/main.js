import { getAddress, getNetworkDetails, isAllowed, isConnected, requestAccess, signMessage } from '@stellar/freighter-api';
import { createWalletService } from './wallet.js';
import { createOwnershipService } from './ownership.js';
import { renderAccountQr, resolveAccountQr } from './account_qr.js';
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
const accountQr = document.querySelector('#account-qr');
const scanForm = document.querySelector('#scan-form');
const scanReference = document.querySelector('#scan-reference');
const scanResult = document.querySelector('#scan-result');

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
  accountQr.hidden = true;
  ownershipMessage.textContent = '';
}

async function showAccountQr(reference) {
  accountReference.textContent = reference;
  accountQr.hidden = true;
  try {
    await renderAccountQr(accountQr, reference);
    accountQr.hidden = false;
  } catch {
    ownershipMessage.textContent = 'QR ntiyashushanyijwe. Reference ya konti iraboneka.';
  }
  ownershipResult.hidden = false;
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
    accountQr.hidden = true;
    return;
  }
  ownershipMessage.textContent = 'Session ya testnet irakora.';
  await showAccountQr(session.qr_reference);
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
      ownershipMessage.textContent = 'Umukono wagenzuwe na IHUZA+ API.';
      await showAccountQr(result.qr_reference);
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
scanForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  scanResult.textContent = 'Turagenzura reference ya konti...';
  const button = scanForm.querySelector('button');
  button.disabled = true;
  try {
    const result = await resolveAccountQr(scanReference.value.trim());
    scanResult.textContent = result?.account_kind === 'person'
      ? 'Iyi ni konti isanzwe. Uburyo bwo kohereza amafaranga ntiburubakwa.'
      : result?.account_kind === 'merchant'
        ? 'Iyi ni konti y’umucuruzi. Checkout itegereje kwemezwa no kubakwa.'
        : 'Reference ntiyabonetse cyangwa ntiyemewe.';
  } catch {
    scanResult.textContent = 'Ntitwabashije kugenzura reference. Ongera ugerageze.';
  } finally {
    button.disabled = false;
  }
});
window.addEventListener('focus', () => { if (!busy) check('inspect'); });
check('inspect');
