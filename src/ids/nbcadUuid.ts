/**
 * BLAKE3-derived RFC 9562 UUID v8. Keep packing in sync with crates/id
 * and scripts/nbcad-id.mjs.
 */
import { blake3 } from '@noble/hashes/blake3';

export const LAYOUT_VERSION = 1;
export const RFC_VERSION = 8;

export const Domain = {
  Session: 'nbcad.uuid.v1.session',
  Correlation: 'nbcad.uuid.v1.corr',
  Tab: 'nbcad.uuid.v1.tab',
} as const;

export type DomainTag = (typeof Domain)[keyof typeof Domain];

let mintCounter = 1n;

function concatBytes(...parts: Uint8Array[]): Uint8Array {
  const length = parts.reduce((sum, part) => sum + part.length, 0);
  const out = new Uint8Array(length);
  let offset = 0;
  for (const part of parts) {
    out.set(part, offset);
    offset += part.length;
  }
  return out;
}

function u64le(value: bigint): Uint8Array {
  const out = new Uint8Array(8);
  new DataView(out.buffer).setBigUint64(0, value, true);
  return out;
}

function u32le(value: number): Uint8Array {
  const out = new Uint8Array(4);
  new DataView(out.buffer).setUint32(0, value >>> 0, true);
  return out;
}

export function fromPreimage(domainTag: DomainTag, preimage: Uint8Array | string): string {
  const payload = concatBytes(
    new TextEncoder().encode(domainTag),
    new Uint8Array([0, LAYOUT_VERSION]),
    typeof preimage === 'string' ? new TextEncoder().encode(preimage) : preimage,
  );
  const hash = blake3(payload);
  const bytes = hash.slice(0, 16);
  bytes[0] = LAYOUT_VERSION;
  bytes[6] = (bytes[6] & 0x0f) | (RFC_VERSION << 4);
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  return formatUuid(bytes);
}

export function mint(domainTag: DomainTag): string {
  const nanos = BigInt(Date.now()) * 1_000_000n;
  const preimage = concatBytes(u64le(nanos), u64le(mintCounter), u32le(0));
  mintCounter += 1n;
  return fromPreimage(domainTag, preimage);
}

export function formatUuid(bytes: Uint8Array): string {
  const hex = [...bytes].map((byte) => byte.toString(16).padStart(2, '0')).join('');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

export function isNbcadUuid(id: string): boolean {
  if (id.length !== 36) return false;
  if (id[8] !== '-' || id[13] !== '-' || id[18] !== '-' || id[23] !== '-') return false;
  const hex = id.replace(/-/g, '');
  if (!/^[0-9a-fA-F]{32}$/.test(hex)) return false;
  const bytes = new Uint8Array(16);
  for (let i = 0; i < 16; i += 1) {
    bytes[i] = Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }
  return (bytes[6] >> 4) === RFC_VERSION && (bytes[8] & 0xc0) === 0x80 && bytes[0] === LAYOUT_VERSION;
}
