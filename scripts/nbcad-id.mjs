/**
 * BLAKE3-derived RFC 9562 UUID v8 (keep in sync with crates/id).
 *
 * blake3(domain || 0x00 || layout_version || preimage)[0..16]
 * byte[0] = LAYOUT_VERSION
 * byte[6] version nibble = 8
 * byte[8] RFC 4122 variant
 */

import { blake3 } from '@noble/hashes/blake3';

export const LAYOUT_VERSION = 1;
export const RFC_VERSION = 8;

export const Domain = {
  Session: 'nbcad.uuid.v1.session',
  Correlation: 'nbcad.uuid.v1.corr',
  Tab: 'nbcad.uuid.v1.tab',
};

let mintCounter = 1n;

function concatBytes(...parts) {
  const length = parts.reduce((sum, part) => sum + part.length, 0);
  const out = new Uint8Array(length);
  let offset = 0;
  for (const part of parts) {
    out.set(part, offset);
    offset += part.length;
  }
  return out;
}

function u64le(value) {
  const out = new Uint8Array(8);
  const view = new DataView(out.buffer);
  view.setBigUint64(0, BigInt(value), true);
  return out;
}

function u32le(value) {
  const out = new Uint8Array(4);
  const view = new DataView(out.buffer);
  view.setUint32(0, value >>> 0, true);
  return out;
}

export function fromPreimage(domainTag, preimage) {
  const payload = concatBytes(
    new TextEncoder().encode(domainTag),
    new Uint8Array([0, LAYOUT_VERSION]),
    preimage instanceof Uint8Array ? preimage : new TextEncoder().encode(preimage),
  );
  const hash = blake3(payload);
  const bytes = hash.slice(0, 16);
  bytes[0] = LAYOUT_VERSION;
  bytes[6] = (bytes[6] & 0x0f) | (RFC_VERSION << 4);
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  return formatUuid(bytes);
}

export function mint(domainTag) {
  const nanos =
    typeof process !== 'undefined' && process.hrtime?.bigint
      ? process.hrtime.bigint()
      : BigInt(Date.now()) * 1_000_000n;
  const pid =
    typeof process !== 'undefined' && typeof process.pid === 'number' ? process.pid : 0;
  const preimage = concatBytes(u64le(nanos), u64le(mintCounter), u32le(pid));
  mintCounter += 1n;
  return fromPreimage(domainTag, preimage);
}

export function formatUuid(bytes) {
  const hex = [...bytes].map((byte) => byte.toString(16).padStart(2, '0')).join('');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

export function parseHyphenated(id) {
  if (typeof id !== 'string' || id.length !== 36) return null;
  if (id[8] !== '-' || id[13] !== '-' || id[18] !== '-' || id[23] !== '-') return null;
  const hex = id.replace(/-/g, '');
  if (!/^[0-9a-fA-F]{32}$/.test(hex)) return null;
  const bytes = new Uint8Array(16);
  for (let i = 0; i < 16; i += 1) {
    bytes[i] = Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

export function isNbcadUuid(id) {
  const bytes = parseHyphenated(id);
  if (!bytes) return false;
  return (bytes[6] >> 4) === RFC_VERSION && (bytes[8] & 0xc0) === 0x80 && bytes[0] === LAYOUT_VERSION;
}

export function isLegacyUuidV4(id) {
  const bytes = parseHyphenated(id);
  if (!bytes) return false;
  return (bytes[6] >> 4) === 4 && (bytes[8] & 0xc0) === 0x80;
}

export function isValidSessionId(id) {
  return isNbcadUuid(id) || isLegacyUuidV4(id);
}
