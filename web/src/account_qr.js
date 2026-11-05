import QRCode from 'qrcode';

const referencePattern = /^[A-Za-z0-9_-]{43}$/;

export function qrContent(reference) {
  if (typeof reference !== 'string' || !referencePattern.test(reference)) {
    throw new Error('Invalid account QR reference');
  }
  return reference;
}

export async function renderAccountQr(canvas, reference, render = QRCode.toCanvas) {
  await render(canvas, qrContent(reference), {
    errorCorrectionLevel: 'M',
    margin: 4,
    width: 224,
    color: { dark: '#071a25', light: '#ffffff' },
  });
}

export async function resolveAccountQr(reference, fetchImpl = fetch) {
  const content = qrContent(reference);
  const response = await fetchImpl(`/api/qr/${content}`, { cache: 'no-store' });
  if (!response.ok) return null;
  const result = await response.json();
  if (result?.account_kind === 'person' && result.flow === 'person_transfer_preview') return result;
  if (result?.account_kind === 'merchant' && result.flow === 'unavailable') return result;
  return null;
}
