import { getAddress, getNetworkDetails, isAllowed, isConnected, requestAccess, signMessage } from '@stellar/freighter-api';
import { createWalletService } from './wallet.js';
import { createOwnershipService } from './ownership.js';
import { renderAccountQr, resolveAccountQr } from './account_qr.js';
import { createMerchantApplicationService } from './merchant_application.js';
import { createMerchantReviewService } from './merchant_review.js';
import { createMerchantCatalogService } from './merchant_catalog.js';
import './styles.css';

const service = createWalletService({ getAddress, getNetworkDetails, isAllowed, isConnected, requestAccess, signMessage });
const ownership = createOwnershipService(service);
const merchantApplication = createMerchantApplicationService();
const merchantReview = createMerchantReviewService(service);
const merchantCatalog = createMerchantCatalogService();
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
const merchantApplicationForm = document.querySelector('#merchant-application-form');
const merchantApplicationStatus = document.querySelector('#merchant-application-status');
const merchantName = document.querySelector('#merchant-name');
const applicantName = document.querySelector('#applicant-name');
const companyName = document.querySelector('#company-name');
const reviewForm = document.querySelector('#review-form');
const reviewReference = document.querySelector('#review-reference');
const reviewPreview = document.querySelector('#review-preview');
const reviewName = document.querySelector('#review-name');
const reviewMessage = document.querySelector('#review-message');
const reviewSignButton = document.querySelector('#review-sign-button');
const reviewStatus = document.querySelector('#review-status');
const reviewerPanel = document.querySelector('#reviewer-panel');
const merchantCatalogSection = document.querySelector('#merchant-catalog-section');
const productForm = document.querySelector('#product-form');
const productName = document.querySelector('#product-name');
const productPrice = document.querySelector('#product-price');
const productStatus = document.querySelector('#product-status');
const ownProducts = document.querySelector('#own-products');
const scanProducts = document.querySelector('#scan-products');
const quoteForm = document.querySelector('#quote-form');
const quoteItems = document.querySelector('#quote-items');
const quoteResult = document.querySelector('#quote-result');
const saveDraftButton = document.querySelector('#save-draft-button');
const draftResult = document.querySelector('#draft-result');

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
let activeReview = null;
let latestQuote = null;
let draftKey = null;

function clearOwnership() {
  activeChallenge = null;
  challengePreview.hidden = true;
  challengeMessage.textContent = '';
  signButton.disabled = true;
  ownershipResult.hidden = true;
  quoteForm.hidden = true;
  latestQuote = null;
  saveDraftButton.hidden = true;
  accountReference.textContent = '';
  accountQr.hidden = true;
  merchantApplicationForm.hidden = true;
  merchantApplicationStatus.textContent = '';
  activeReview = null;
  reviewPreview.hidden = true;
  reviewReference.disabled = false;
  reviewStatus.textContent = '';
  reviewerPanel.hidden = true;
  merchantCatalogSection.hidden = true;
  ownProducts.replaceChildren();
  productStatus.textContent = '';
  ownershipMessage.textContent = '';
}

async function refreshReviewerStatus() {
  reviewerPanel.hidden = true;
  const reference = accountReference.textContent;
  try {
    const eligible = await merchantReview.status();
    if (reference === accountReference.textContent && !ownershipResult.hidden) {
      reviewerPanel.hidden = !eligible;
    }
  } catch {
    reviewerPanel.hidden = true;
  }
}

function showProducts(list, products) {
  list.replaceChildren();
  for (const product of products) {
    const item = document.createElement('li');
    const name = document.createElement('span');
    name.textContent = product.name;
    const price = document.createElement('strong');
    price.textContent = `${product.unit_price} RWF`;
    item.append(name, price);
    list.append(item);
  }
}

async function refreshMerchantApplication() {
  merchantApplicationForm.hidden = true;
  merchantCatalogSection.hidden = true;
  merchantApplicationStatus.textContent = 'Turagenzura ubusabe bw’umucuruzi...';
  try {
    const application = await merchantApplication.current();
    if (application) {
      if (application.status === 'needs_details') {
        merchantName.value = application.proposed_name;
        merchantName.readOnly = true;
        merchantApplicationForm.hidden = false;
      }
      merchantApplicationStatus.textContent = application.status === 'needs_details'
        ? 'Ubusabe bwa kera buracyakeneye izina ryawe n’irya company niba uyifite; busaba isuzuma rishya.'
        : application.status === 'approved'
        ? `Ubusabe bwa “${application.proposed_name}” bwemejwe n’abasuzuma babiri. Ushobora gushyiraho ibicuruzwa; checkout ntirubakwa.`
        : `Ubusabe bwa “${application.proposed_name}” buracyasuzumwa. QR yawe iracyari iya konti isanzwe.`;
      if (application.status === 'approved' && !ownershipResult.hidden) {
        const reference = accountReference.textContent;
        const products = await merchantCatalog.list(reference);
        if (reference === accountReference.textContent && !ownershipResult.hidden) {
          showProducts(ownProducts, products);
          merchantCatalogSection.hidden = false;
        }
      }
    } else {
      merchantApplicationForm.hidden = false;
      merchantApplicationStatus.textContent = '';
    }
  } catch {
    merchantApplicationStatus.textContent = 'Ntitwabashije kugenzura ubusabe. Ongera ugerageze.';
  }
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
    quoteForm.hidden = true;
    latestQuote = null;
    saveDraftButton.hidden = true;
    accountReference.textContent = '';
    accountQr.hidden = true;
    merchantApplicationForm.hidden = true;
    merchantApplicationStatus.textContent = '';
    activeReview = null;
    reviewPreview.hidden = true;
    reviewReference.disabled = false;
    reviewerPanel.hidden = true;
    merchantCatalogSection.hidden = true;
    ownProducts.replaceChildren();
    return;
  }
  ownershipMessage.textContent = 'Session ya testnet irakora.';
  await showAccountQr(session.qr_reference);
  await refreshMerchantApplication();
  await refreshReviewerStatus();
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
      await refreshMerchantApplication();
      await refreshReviewerStatus();
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
  scanProducts.hidden = true;
  scanProducts.replaceChildren();
  quoteForm.hidden = true;
  quoteItems.replaceChildren();
  quoteResult.textContent = '';
  draftResult.textContent = '';
  latestQuote = null;
  draftKey = null;
  saveDraftButton.hidden = true;
  const button = scanForm.querySelector('button');
  button.disabled = true;
  try {
    const result = await resolveAccountQr(scanReference.value.trim());
    scanResult.textContent = result?.account_kind === 'person'
      ? 'Iyi ni konti isanzwe. Uburyo bwo kohereza amafaranga ntiburubakwa.'
      : result?.account_kind === 'merchant'
        ? 'Iyi ni konti y’umucuruzi. Ibiciro biri muri catalog; checkout ntirubakwa.'
        : 'Reference ntiyabonetse cyangwa ntiyemewe.';
    if (result?.account_kind === 'merchant') {
      const products = await merchantCatalog.list(scanReference.value.trim());
      showProducts(scanProducts, products);
      scanProducts.hidden = false;
      if (products.length && !ownershipResult.hidden) {
        for (const product of products) {
          const row = document.createElement('label');
          row.className = 'quote-item';
          const name = document.createElement('span');
          name.textContent = `${product.name} — ${product.unit_price} RWF`;
          const quantity = document.createElement('input');
          quantity.type = 'number';
          quantity.min = '0';
          quantity.max = '100';
          quantity.step = '1';
          quantity.value = '0';
          quantity.dataset.productId = product.id;
          quantity.setAttribute('aria-label', `Ingano ya ${product.name}`);
          quantity.addEventListener('input', () => { latestQuote = null; draftKey = null; saveDraftButton.hidden = true; quoteResult.textContent = ''; draftResult.textContent = ''; });
          row.append(name, quantity);
          quoteItems.append(row);
        }
        quoteForm.hidden = false;
      }
      if (products.length && ownershipResult.hidden) scanResult.textContent = 'Catalog iraboneka. Injira muri IHUZA+ kugira ngo ubare total y’agateganyo.';
      if (!products.length) scanResult.textContent = 'Iyi ni konti y’umucuruzi; nta bicuruzwa byashyizweho. Checkout ntirubakwa.';
    }
  } catch {
    scanResult.textContent = 'Ntitwabashije kugenzura reference. Ongera ugerageze.';
  } finally {
    button.disabled = false;
  }
});
scanReference.addEventListener('input', () => { latestQuote = null; draftKey = null; saveDraftButton.hidden = true; quoteResult.textContent = ''; draftResult.textContent = ''; });
quoteForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  const button = quoteForm.querySelector('button');
  button.disabled = true;
  quoteResult.textContent = 'Turabara total ku biciro biri kuri server...';
  try {
    const items = [...quoteItems.querySelectorAll('input')].filter((input) => Number(input.value) > 0)
      .map((input) => ({ product_id: input.dataset.productId, quantity: Number(input.value) }));
    const preview = await merchantCatalog.preview(scanReference.value.trim(), items);
    latestQuote = { reference: scanReference.value.trim(), items };
    draftKey = null;
    saveDraftButton.hidden = false;
    quoteResult.textContent = `Total y’agateganyo: ${preview.total} RWF. Nta fagitire cyangwa ubwishyu byakozwe.`;
  } catch {
    latestQuote = null;
    saveDraftButton.hidden = true;
    quoteResult.textContent = 'Total ntiyabonetse. Hitamo ibicuruzwa, wemeze ko winjiye muri IHUZA+ kandi ko merchant akiri wemewe.';
  } finally {
    button.disabled = false;
  }
});
saveDraftButton.addEventListener('click', async () => {
  if (!latestQuote || saveDraftButton.disabled) return;
  saveDraftButton.disabled = true;
  draftResult.textContent = 'Turabika invoice draft...';
  try {
    if (!draftKey) draftKey = [...crypto.getRandomValues(new Uint8Array(32))].map((byte) => byte.toString(16).padStart(2, '0')).join('');
    const draft = await merchantCatalog.createDraft(latestQuote.reference, latestQuote.items, draftKey);
    draftResult.textContent = `Invoice draft ${draft.id} yabitswe: ${draft.total} RWF. Ntiyishyuwe; si fagitire ya EBM.`;
    saveDraftButton.hidden = true;
    latestQuote = null;
  } catch {
    draftResult.textContent = 'Invoice draft ntibashije kubikwa. Ongera ugerageze; urutonde rumwe ntirurema invoice ya kabiri.';
  } finally {
    saveDraftButton.disabled = false;
  }
});
productForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  if (busy || !activeAddress || merchantCatalogSection.hidden) return;
  busy = true;
  const button = productForm.querySelector('button');
  button.disabled = true;
  productStatus.textContent = 'Turabika igicuruzwa...';
  try {
    await merchantCatalog.create(productName.value.trim(), productPrice.value.trim());
    showProducts(ownProducts, await merchantCatalog.list(accountReference.textContent));
    productName.value = '';
    productPrice.value = '';
    productStatus.textContent = 'Igicuruzwa cyabitswe. Checkout ntirubakwa.';
  } catch {
    productStatus.textContent = 'Igicuruzwa nticyabitswe. Genzura izina, igiciro n’uko merchant yemewe.';
  } finally {
    button.disabled = false;
    busy = false;
  }
});
merchantApplicationForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  if (busy) return;
  busy = true;
  const button = merchantApplicationForm.querySelector('button');
  button.disabled = true;
  merchantApplicationStatus.textContent = 'Turabika ubusabe bwawe...';
  try {
    const application = await merchantApplication.submit(merchantName.value.trim(), applicantName.value.trim(), companyName.value.trim() || null);
    merchantApplicationForm.hidden = true;
    merchantApplicationStatus.textContent = `Ubusabe bwa “${application.proposed_name}” bwakiriwe. Buracyasuzumwa; checkout ntirafungurwa.`;
  } catch {
    merchantApplicationStatus.textContent = 'Ubusabe ntibwakiriwe. Reba izina wanditse cyangwa ubusabe busanzwe buriho.';
  } finally {
    button.disabled = false;
    busy = false;
  }
});
reviewForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  if (busy || !activeAddress) return;
  busy = true;
  activeReview = null;
  reviewPreview.hidden = true;
  reviewStatus.textContent = 'Turategura ubutumwa bw’isuzuma...';
  const button = reviewForm.querySelector('button');
  button.disabled = true;
  try {
    const prepared = await merchantReview.prepare(reviewReference.value.trim(), activeAddress);
    activeReview = prepared;
    reviewName.textContent = `Uwasabye: ${prepared.applicant_name}; ubucuruzi: ${prepared.proposed_name}; company: ${prepared.company_name ?? 'nta yo yatanzwe'}. Isuzuma rya testnet gusa.`;
    reviewMessage.textContent = prepared.message;
    reviewPreview.hidden = false;
    reviewReference.disabled = true;
    reviewStatus.textContent = 'Banza usuzume izina n’ubutumwa; sinyira gusa niba ubyemeye.';
  } catch {
    reviewStatus.textContent = 'Isuzuma ntiryabonetse. Reba ko uri reviewer wemewe na reference y’ubusabe.';
  } finally {
    button.disabled = false;
    busy = false;
  }
});
reviewSignButton.addEventListener('click', async () => {
  if (busy || !activeReview) return;
  busy = true;
  reviewSignButton.disabled = true;
  reviewStatus.textContent = 'Soma ubutumwa muri Freighter mbere yo kwemeza umukono.';
  try {
    const result = await merchantReview.submit(activeReview);
    reviewStatus.textContent = result.approved
      ? 'Imikono 2 muri 3 yemejwe. Merchant status irakora kuri iyi reviewer set; checkout ntirubakwa.'
      : `Vote yabitswe (${result.votes}/2). Haracyakenewe undi reviewer.`;
    activeReview = null;
    reviewPreview.hidden = true;
    reviewReference.disabled = false;
  } catch {
    reviewStatus.textContent = 'Umukono cyangwa vote ntibyemejwe. Ongera utegure isuzuma.';
    activeReview = null;
    reviewPreview.hidden = true;
    reviewReference.disabled = false;
  } finally {
    reviewSignButton.disabled = false;
    busy = false;
  }
});
window.addEventListener('focus', () => { if (!busy) check('inspect'); });
check('inspect');
