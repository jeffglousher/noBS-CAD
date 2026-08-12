//! BLAKE3-derived identifiers packed as RFC 9562 UUID version 8.
//!
//! We control these IDs, so they are **v8** (custom) rather than random v4.
//! Byte 0 is our **internal layout version** so the 122 custom bits can evolve
//! without colliding with a later packing.
//!
//! ```text
//! blake3(domain || 0x00 || layout_version || preimage)[0..16]
//! byte[0]     = LAYOUT_VERSION
//! byte[6]     = version nibble 8
//! byte[8]     = RFC 4122 variant (10xxxxxx)
//! ```

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Internal packing version stored in UUID byte 0. Bump when the custom-bit
/// layout changes; keep [`crate::Domain`] tags stable or introduce a new tag.
pub const LAYOUT_VERSION: u8 = 1;

/// RFC 9562 version nibble for custom UUIDs.
pub const RFC_VERSION: u8 = 8;

static MINT_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Domain-separated preimage tags (hashed, not stored).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    Session,
    Correlation,
    Tab,
}

impl Domain {
    pub fn tag(self) -> &'static [u8] {
        match self {
            Self::Session => b"nbcad.uuid.v1.session",
            Self::Correlation => b"nbcad.uuid.v1.corr",
            Self::Tab => b"nbcad.uuid.v1.tab",
        }
    }
}

/// 16-byte UUID v8 (hyphenated Display).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NbcadUuid([u8; 16]);

impl NbcadUuid {
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    pub fn layout_version(&self) -> u8 {
        self.0[0]
    }

    pub fn rfc_version(&self) -> u8 {
        self.0[6] >> 4
    }
}

impl fmt::Display for NbcadUuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = &self.0;
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
        )
    }
}

/// Deterministic ID from a domain + preimage (tests, derived keys).
pub fn from_preimage(domain: Domain, preimage: &[u8]) -> NbcadUuid {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.tag());
    hasher.update(&[0]);
    hasher.update(&[LAYOUT_VERSION]);
    hasher.update(preimage);
    let hash = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash.as_bytes()[..16]);
    bytes[0] = LAYOUT_VERSION;
    bytes[6] = (bytes[6] & 0x0F) | (RFC_VERSION << 4);
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
    NbcadUuid(bytes)
}

/// Unique ID for this process (time + counter + pid).
pub fn mint(domain: Domain) -> NbcadUuid {
    let mut preimage = [0u8; 20];
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    preimage[0..8].copy_from_slice(&nanos.to_le_bytes());
    preimage[8..16].copy_from_slice(&MINT_COUNTER.fetch_add(1, Ordering::Relaxed).to_le_bytes());
    preimage[16..20].copy_from_slice(&std::process::id().to_le_bytes());
    from_preimage(domain, &preimage)
}

pub fn mint_string(domain: Domain) -> String {
    mint(domain).to_string()
}

/// Parse a hyphenated UUID into 16 bytes.
pub fn parse_hyphenated(id: &str) -> Option<[u8; 16]> {
    let bytes = id.as_bytes();
    if bytes.len() != 36 {
        return None;
    }
    let mut out = [0u8; 16];
    let mut parsed = 0usize;
    for (index, byte) in bytes.iter().enumerate() {
        match index {
            8 | 13 | 18 | 23 => {
                if *byte != b'-' {
                    return None;
                }
            }
            _ => {
                let nibble = hex_nibble(*byte)?;
                if parsed % 2 == 0 {
                    out[parsed / 2] = nibble << 4;
                } else {
                    out[parsed / 2] |= nibble;
                }
                parsed += 1;
            }
        }
    }
    if parsed != 32 {
        return None;
    }
    Some(out)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn rfc_version(bytes: &[u8; 16]) -> u8 {
    bytes[6] >> 4
}

fn rfc_variant_ok(bytes: &[u8; 16]) -> bool {
    bytes[8] & 0xC0 == 0x80
}

/// Canonical noBS ID: UUID v8, RFC variant, internal layout version.
pub fn is_nbcad_uuid(id: &str) -> bool {
    let Some(bytes) = parse_hyphenated(id) else {
        return false;
    };
    rfc_version(&bytes) == RFC_VERSION && rfc_variant_ok(&bytes) && bytes[0] == LAYOUT_VERSION
}

/// Legacy random v4 (older session dirs). Same 8-4-4-4-12 shape; not minted.
pub fn is_legacy_uuid_v4(id: &str) -> bool {
    let Some(bytes) = parse_hyphenated(id) else {
        return false;
    };
    rfc_version(&bytes) == 4 && rfc_variant_ok(&bytes)
}

/// Session directory names: current v8 layout, or legacy v4.
pub fn is_valid_session_id(id: &str) -> bool {
    is_nbcad_uuid(id) || is_legacy_uuid_v4(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_session_preimage_is_v8_layout_1() {
        let id = from_preimage(Domain::Session, b"golden");
        assert_eq!(id.rfc_version(), 8);
        assert_eq!(id.layout_version(), LAYOUT_VERSION);
        assert!(id.to_string().starts_with("01"));
        assert_eq!(id.to_string(), "01732db8-694c-886c-87d8-c2c64537d673");
    }

    #[test]
    fn mint_is_unique_and_valid() {
        let a = mint_string(Domain::Session);
        let b = mint_string(Domain::Session);
        assert_ne!(a, b);
        assert!(is_nbcad_uuid(&a));
        assert!(is_nbcad_uuid(&b));
        assert!(!is_legacy_uuid_v4(&a));
    }

    #[test]
    fn domains_do_not_collide_on_same_preimage() {
        let session = from_preimage(Domain::Session, b"same").to_string();
        let corr = from_preimage(Domain::Correlation, b"same").to_string();
        let tab = from_preimage(Domain::Tab, b"same").to_string();
        assert_ne!(session, corr);
        assert_ne!(session, tab);
        assert_ne!(corr, tab);
    }

    #[test]
    fn validation_rejects_names_and_wrong_version() {
        assert!(!is_valid_session_id("My Document"));
        assert!(!is_valid_session_id("../escape"));
        assert!(!is_valid_session_id(""));
        assert!(!is_nbcad_uuid("123e4567-e89b-12d3-a456-426614174000"));
        assert!(is_legacy_uuid_v4("123e4567-e89b-42d3-a456-426614174000"));
        assert!(is_valid_session_id("123e4567-e89b-42d3-a456-426614174000"));
        assert!(!is_valid_session_id("123e4567-e89b-12d3-a456-426614174000"));
    }
}
