//! Decryption of an ECMA-376 encrypted xlsx package (SPEC-0031 D2, D3, D5; T-4265).
//!
//! A password-protected xlsx is an OLE compound file with two streams: `EncryptionInfo`, which
//! says how the package was encrypted, and `EncryptedPackage`, the zip encrypted under a key
//! derived from the password (ECMA-376 Part 2, MS-OFFCRYPTO 2.3.4). Two kinds are read:
//! - **Agile** (`EncryptionInfo` version 4.4, what Excel 2010 and later write): an XML
//!   descriptor, a salted and iterated SHA-1 or SHA-2 key derivation (`spinCount` rounds), AES in
//!   CBC over 4,096-byte segments and an HMAC over the whole encrypted stream (MS-OFFCRYPTO
//!   2.3.4.10 to 2.3.4.15).
//! - **Standard** (version 2.2, 3.2 or 4.2 with `fCryptoAPI` and `fAES`, Excel 2007): a binary
//!   header, a fixed 50,000-round SHA-1 derivation and AES in ECB (MS-OFFCRYPTO 2.3.4.5 to
//!   2.3.4.9). It carries no integrity data.
//!
//! Everything else (Extensible encryption, RC4, a certificate-only key encryptor, a cipher or
//! chaining mode other than AES-CBC) is [`XlsxError::UnsupportedEncryption`].
//!
//! Both streams are untrusted (CLAUDE.md directive 5). Every length, count and size is checked
//! before it is used: the spin count before any hashing, the declared package size before any
//! allocation, base64 text before it is decoded, the XML descriptor's element count and depth
//! while it is read. The password is checked with a constant-time comparison before its key
//! decrypts anything, and the Agile HMAC is checked, also in constant time, before any plain text
//! is returned. The password's UTF-16 bytes and every derived key live in zeroized buffers. The
//! primitives are the `RustCrypto` crates (`aes`, `cbc`, `ecb`, `sha1`, `sha2`, `hmac`): no hand-rolled
//! cryptography.
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::cast_possible_truncation)]
// LINT-DEBT-END

use aes::cipher::generic_array::GenericArray;
use aes::cipher::typenum::U16;
use aes::cipher::{BlockCipher, BlockDecryptMut, BlockSizeUser, KeyInit, KeyIvInit};
use aes::{Aes128, Aes192, Aes256};
use base64::Engine as _;
use hmac::{Mac, SimpleHmac};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use sha1::Sha1;
use sha2::digest::core_api::BlockSizeUser as DigestBlockSizeUser;
use sha2::{Digest, Sha256, Sha384, Sha512};
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, Zeroizing};

use super::package::ImportLimits;
use super::xml::start_attribute;
use crate::error::{ImportLimit, XlsxError};

/// The largest `spinCount` MS-OFFCRYPTO allows (2.3.4.10, `CT_PasswordKeyEncryptor`): a larger
/// one is malformed, whatever [`ImportLimits::max_spin_count`] says.
pub const MAX_SPIN_COUNT: u32 = 10_000_000;

/// The longest password, in UTF-16 code units (MS-OFFCRYPTO 2.3.4.7 and 2.3.4.11; SPEC-0031 E3).
pub const MAX_PASSWORD_UNITS: usize = 255;

/// Most bytes an `EncryptionInfo` stream may hold (SPEC-0031 D1). Excel's are under 2 KiB.
pub const MAX_ENCRYPTION_INFO_BYTES: u64 = 1 << 20;

/// The AES block size, the only `blockSize` accepted (SPEC-0031 D3).
const BLOCK: usize = 16;

/// Agile packages are encrypted in segments of this many plain bytes, each with its own IV
/// (MS-OFFCRYPTO 2.3.4.15).
const SEGMENT: usize = 4096;

/// The `EncryptedPackage` stream starts with the plain package size, a little-endian u64
/// (MS-OFFCRYPTO 2.3.4.4).
const SIZE_PREFIX: usize = 8;

/// The Agile descriptor's shape bounds (SPEC-0031 D3): Excel's has 6 elements, 4 deep.
const MAX_XML_ELEMENTS: usize = 64;
const MAX_XML_DEPTH: usize = 8;

/// The longest base64 attribute value read (344 characters encode 256 bytes); the longest
/// legitimate one, a 64-byte hash padded to 80, is 108.
const MAX_BASE64_CHARS: usize = 344;

/// The `uri` of the password key encryptor (MS-OFFCRYPTO 2.3.4.10).
const PASSWORD_KEY_ENCRYPTOR: &str =
    "http://schemas.microsoft.com/office/2006/keyEncryptor/password";

/// Agile block keys (MS-OFFCRYPTO 2.3.4.13 and 2.3.4.14).
const VERIFIER_INPUT_BLOCK: [u8; 8] = [0xfe, 0xa7, 0xd2, 0x76, 0x3b, 0x4b, 0x9e, 0x79];
const VERIFIER_HASH_BLOCK: [u8; 8] = [0xd7, 0xaa, 0x0f, 0x6d, 0x30, 0x61, 0x34, 0x4e];
const KEY_VALUE_BLOCK: [u8; 8] = [0x14, 0x6e, 0x0b, 0xe7, 0xab, 0xac, 0xd0, 0xd6];
const HMAC_KEY_BLOCK: [u8; 8] = [0x5f, 0xb2, 0xad, 0x01, 0x0c, 0xb9, 0xe1, 0xf6];
const HMAC_VALUE_BLOCK: [u8; 8] = [0xa0, 0x67, 0x7f, 0x02, 0xb2, 0x2c, 0x84, 0x33];

/// Standard encryption's fixed derivation rounds (MS-OFFCRYPTO 2.3.4.7).
const STANDARD_SPIN_COUNT: u32 = 50_000;

/// `EncryptionHeader.Flags` bits (MS-OFFCRYPTO 2.3.1).
const F_CRYPTO_API: u32 = 0x04;
const F_EXTERNAL: u32 = 0x10;
const F_AES: u32 = 0x20;

/// Decrypts an ECMA-376 encrypted package into the zip bytes it holds.
///
/// `info` is the `EncryptionInfo` stream and `package` the `EncryptedPackage` stream, as read
/// from the OLE container ([`super::ole::decrypt_ole_package`] reads them).
///
/// # Errors
/// - [`XlsxError::WrongPassword`] when the password does not open the package, or is longer than
///   255 UTF-16 code units;
/// - [`XlsxError::UnsupportedEncryption`] for an encryption kind not read here, or a `spinCount`
///   past MS-OFFCRYPTO's 10,000,000;
/// - [`XlsxError::Limit`] for a `spinCount` past [`ImportLimits::max_spin_count`], or a package
///   larger than [`ImportLimits::max_uncompressed_bytes`];
/// - [`XlsxError::MalformedEncryption`] for a stream that breaks the format, or an Agile package
///   that fails its integrity check.
pub fn decrypt_package(
    info: &[u8],
    package: &[u8],
    password: &str,
    limits: &ImportLimits,
) -> Result<Vec<u8>, XlsxError> {
    if u64::try_from(info.len()).map_or(true, |n| n > MAX_ENCRYPTION_INFO_BYTES) {
        return Err(malformed("EncryptionInfo is larger than 1 MiB"));
    }
    let (major, minor) = match info {
        [a, b, c, d, ..] => (u16::from_le_bytes([*a, *b]), u16::from_le_bytes([*c, *d])),
        _ => return Err(malformed("EncryptionInfo is shorter than its version")),
    };
    match (major, minor) {
        (4, 4) => {
            let params = AgileParams::parse(info)?;
            check_spin_count(params.spin_count, limits)?;
            let plain_size = check_package(package, limits)?;
            let password = password_bytes(password)?;
            decrypt_agile(&params, package, &password, plain_size)
        }
        (2..=4, 2) => {
            let params = StandardParams::parse(info)?;
            let plain_size = check_package(package, limits)?;
            let password = password_bytes(password)?;
            decrypt_standard(&params, package, &password, plain_size)
        }
        (3 | 4, 3) => Err(unsupported("Extensible encryption")),
        (1, 1) => Err(unsupported("RC4")),
        _ => Err(unsupported(&format!(
            "EncryptionInfo version {major}.{minor}"
        ))),
    }
}

fn malformed(what: &str) -> XlsxError {
    XlsxError::MalformedEncryption(what.to_string())
}

fn unsupported(what: &str) -> XlsxError {
    XlsxError::UnsupportedEncryption(what.to_string())
}

/// Refuses a spin count past the format maximum (malformed) or past the caller's limit, before
/// any hashing (SPEC-0031 D3).
fn check_spin_count(spin_count: u32, limits: &ImportLimits) -> Result<(), XlsxError> {
    if spin_count > MAX_SPIN_COUNT {
        return Err(unsupported(&format!(
            "spinCount {spin_count} past the format's {MAX_SPIN_COUNT}"
        )));
    }
    if spin_count > limits.max_spin_count {
        return Err(XlsxError::Limit {
            limit: ImportLimit::SpinCount,
            max: u64::from(limits.max_spin_count),
        });
    }
    Ok(())
}

/// The plain size the package declares, after its shape checks (SPEC-0031 D3): the ciphertext
/// after the size is whole AES blocks, the declared size fits in it and in the byte budget.
fn check_package(package: &[u8], limits: &ImportLimits) -> Result<usize, XlsxError> {
    let Some((size, ciphertext)) = package.split_first_chunk::<SIZE_PREFIX>() else {
        return Err(malformed("EncryptedPackage is shorter than its size"));
    };
    let declared = u64::from_le_bytes(*size);
    if declared > limits.max_uncompressed_bytes {
        return Err(limits.bytes_exceeded());
    }
    if !ciphertext.len().is_multiple_of(BLOCK) {
        return Err(malformed("EncryptedPackage is not whole AES blocks"));
    }
    match usize::try_from(declared) {
        Ok(n) if n <= ciphertext.len() => Ok(n),
        _ => Err(malformed(
            "EncryptedPackage declares more bytes than it holds",
        )),
    }
}

/// The password as the key derivations hash it: UTF-16LE, case-sensitive, at most 255 code
/// units (MS-OFFCRYPTO 2.3.4.7, 2.3.4.11). A longer one is refused, not truncated (SPEC-0031 E3,
/// UNVERIFIED until T-4263): no file Excel writes can need it.
fn password_bytes(password: &str) -> Result<Zeroizing<Vec<u8>>, XlsxError> {
    let mut bytes = Zeroizing::new(Vec::with_capacity(2 * password.len()));
    for unit in password.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
        if bytes.len() > 2 * MAX_PASSWORD_UNITS {
            return Err(XlsxError::WrongPassword);
        }
    }
    Ok(bytes)
}

// --- Hashing -------------------------------------------------------------------------------

/// A hash algorithm Agile encryption may name (MS-OFFCRYPTO 2.3.4.10); Standard uses SHA-1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HashAlgorithm {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

impl HashAlgorithm {
    fn from_name(name: &str) -> Result<Self, XlsxError> {
        match name {
            "SHA1" => Ok(Self::Sha1),
            "SHA256" => Ok(Self::Sha256),
            "SHA384" => Ok(Self::Sha384),
            "SHA512" => Ok(Self::Sha512),
            // MD5, MD4, MD2, RIPEMD and WHIRLPOOL are in the schema; Excel never writes them.
            _ => Err(unsupported(&format!("hash algorithm {name}"))),
        }
    }

    /// Bytes of output.
    fn len(self) -> usize {
        match self {
            Self::Sha1 => 20,
            Self::Sha256 => 32,
            Self::Sha384 => 48,
            Self::Sha512 => 64,
        }
    }

    /// The hash of the concatenation of `parts`.
    fn hash(self, parts: &[&[u8]]) -> Zeroizing<Vec<u8>> {
        match self {
            Self::Sha1 => hash_with::<Sha1>(parts),
            Self::Sha256 => hash_with::<Sha256>(parts),
            Self::Sha384 => hash_with::<Sha384>(parts),
            Self::Sha512 => hash_with::<Sha512>(parts),
        }
    }

    /// `H(salt || password)`, then `spin_count` rounds of `H(i_le32 || h)` (MS-OFFCRYPTO
    /// 2.3.4.7, 2.3.4.11).
    fn iterated(self, salt: &[u8], password: &[u8], spin_count: u32) -> Zeroizing<Vec<u8>> {
        match self {
            Self::Sha1 => iterated_with::<Sha1>(salt, password, spin_count),
            Self::Sha256 => iterated_with::<Sha256>(salt, password, spin_count),
            Self::Sha384 => iterated_with::<Sha384>(salt, password, spin_count),
            Self::Sha512 => iterated_with::<Sha512>(salt, password, spin_count),
        }
    }

    /// Whether `tag` is the HMAC of `data` under `key`, compared in constant time.
    fn hmac_matches(self, key: &[u8], data: &[u8], tag: &[u8]) -> bool {
        match self {
            Self::Sha1 => hmac_matches_with::<Sha1>(key, data, tag),
            Self::Sha256 => hmac_matches_with::<Sha256>(key, data, tag),
            Self::Sha384 => hmac_matches_with::<Sha384>(key, data, tag),
            Self::Sha512 => hmac_matches_with::<Sha512>(key, data, tag),
        }
    }
}

fn hash_with<D: Digest>(parts: &[&[u8]]) -> Zeroizing<Vec<u8>> {
    let mut hasher = D::new();
    for part in parts {
        hasher.update(part);
    }
    Zeroizing::new(hasher.finalize().to_vec())
}

fn iterated_with<D: Digest>(salt: &[u8], password: &[u8], spin_count: u32) -> Zeroizing<Vec<u8>> {
    let mut h = D::new()
        .chain_update(salt)
        .chain_update(password)
        .finalize();
    for i in 0..spin_count {
        let next = D::new()
            .chain_update(i.to_le_bytes())
            .chain_update(&h)
            .finalize();
        h.as_mut_slice().zeroize();
        h = next;
    }
    let out = Zeroizing::new(h.to_vec());
    h.as_mut_slice().zeroize();
    out
}

fn hmac_matches_with<D: Digest + DigestBlockSizeUser>(key: &[u8], data: &[u8], tag: &[u8]) -> bool {
    let Ok(mut mac) = <SimpleHmac<D> as Mac>::new_from_slice(key) else {
        return false;
    };
    mac.update(data);
    mac.verify_slice(tag).is_ok()
}

/// `bytes` cut or padded with `fill` to `len` bytes (MS-OFFCRYPTO 2.3.4.11: keys pad with 0x36;
/// 2.3.4.12: IVs too).
fn fit(bytes: &[u8], len: usize, fill: u8) -> Zeroizing<Vec<u8>> {
    let mut out = Zeroizing::new(vec![fill; len]);
    let n = bytes.len().min(len);
    out[..n].copy_from_slice(&bytes[..n]);
    out
}

// --- AES -----------------------------------------------------------------------------------

/// Decrypts `data` (whole blocks) in place with AES in CBC mode, no padding.
fn aes_cbc_decrypt(key: &[u8], iv: &[u8], data: &mut [u8]) -> Result<(), XlsxError> {
    match key.len() {
        16 => cbc_with::<Aes128>(key, iv, data),
        24 => cbc_with::<Aes192>(key, iv, data),
        32 => cbc_with::<Aes256>(key, iv, data),
        _ => Err(malformed("AES key of an unsupported size")),
    }
}

/// Decrypts `data` (whole blocks) in place with AES in ECB mode, no padding.
fn aes_ecb_decrypt(key: &[u8], data: &mut [u8]) -> Result<(), XlsxError> {
    match key.len() {
        16 => ecb_with::<Aes128>(key, data),
        24 => ecb_with::<Aes192>(key, data),
        32 => ecb_with::<Aes256>(key, data),
        _ => Err(malformed("AES key of an unsupported size")),
    }
}

fn cbc_with<C>(key: &[u8], iv: &[u8], data: &mut [u8]) -> Result<(), XlsxError>
where
    C: BlockCipher + BlockDecryptMut + BlockSizeUser<BlockSize = U16> + KeyInit,
{
    let mut cipher = cbc::Decryptor::<C>::new_from_slices(key, iv)
        .map_err(|_| malformed("AES key or IV of the wrong size"))?;
    decrypt_blocks(&mut cipher, data)
}

fn ecb_with<C>(key: &[u8], data: &mut [u8]) -> Result<(), XlsxError>
where
    C: BlockCipher + BlockDecryptMut + BlockSizeUser<BlockSize = U16> + KeyInit,
{
    let mut cipher = ecb::Decryptor::<C>::new_from_slice(key)
        .map_err(|_| malformed("AES key of the wrong size"))?;
    decrypt_blocks(&mut cipher, data)
}

fn decrypt_blocks<C: BlockDecryptMut + BlockSizeUser<BlockSize = U16>>(
    cipher: &mut C,
    data: &mut [u8],
) -> Result<(), XlsxError> {
    if !data.len().is_multiple_of(BLOCK) {
        return Err(malformed("ciphertext is not whole AES blocks"));
    }
    for block in data.chunks_exact_mut(BLOCK) {
        cipher.decrypt_block_mut(GenericArray::from_mut_slice(block));
    }
    Ok(())
}

// --- Agile ---------------------------------------------------------------------------------

/// The cipher parameters shared by `keyData` and the password key encryptor
/// (`CT_KeyData`, `CT_PasswordKeyEncryptor`; MS-OFFCRYPTO 2.3.4.10).
#[derive(Debug, Clone, PartialEq, Eq)]
struct CipherParams {
    salt: Vec<u8>,
    hash: HashAlgorithm,
    /// Key bytes: 16, 24 or 32.
    key_len: usize,
}

impl CipherParams {
    /// Reads and checks the attributes of `element` (SPEC-0031 D3): AES in CBC, block size 16,
    /// 128, 192 or 256 key bits, a salt of 1 to 64 bytes matching `saltSize`.
    fn read(element: &BytesStart<'_>) -> Result<Self, XlsxError> {
        let cipher = required(element, "cipherAlgorithm")?;
        if cipher != "AES" {
            return Err(unsupported(&format!("cipher {cipher}")));
        }
        let chaining = required(element, "cipherChaining")?;
        if chaining != "ChainingModeCBC" {
            return Err(unsupported(&format!("chaining mode {chaining}")));
        }
        if number(element, "blockSize")? != BLOCK as u32 {
            return Err(malformed("blockSize is not 16"));
        }
        let key_len = match number(element, "keyBits")? {
            bits @ (128 | 192 | 256) => bits as usize / 8,
            _ => return Err(malformed("keyBits is not 128, 192 or 256")),
        };
        let hash = HashAlgorithm::from_name(&required(element, "hashAlgorithm")?)?;
        if number(element, "hashSize")? as usize != hash.len() {
            return Err(malformed("hashSize does not match hashAlgorithm"));
        }
        let salt = base64_attribute(element, "saltValue")?;
        if salt.is_empty() || salt.len() > 64 || number(element, "saltSize")? as usize != salt.len()
        {
            return Err(malformed("salt is not 1 to 64 bytes of saltSize"));
        }
        Ok(CipherParams {
            salt,
            hash,
            key_len,
        })
    }
}

/// What the Agile descriptor holds (MS-OFFCRYPTO 2.3.4.10).
#[derive(Debug)]
struct AgileParams {
    key_data: CipherParams,
    encrypted_hmac_key: Vec<u8>,
    encrypted_hmac_value: Vec<u8>,
    password: CipherParams,
    spin_count: u32,
    encrypted_verifier_input: Vec<u8>,
    encrypted_verifier_hash: Vec<u8>,
    encrypted_key_value: Vec<u8>,
}

/// The parts of the descriptor found while reading it.
#[derive(Default)]
struct AgileParts {
    key_data: Option<CipherParams>,
    integrity: Option<(Vec<u8>, Vec<u8>)>,
    password: Option<(CipherParams, u32, [Vec<u8>; 3])>,
    /// Whether a certificate (or other) key encryptor was seen, to name why a file without a
    /// password one is refused.
    other_encryptor: bool,
}

impl AgileParams {
    /// Parses the descriptor after the 8-byte header (version 4.4, reserved 0x40).
    fn parse(info: &[u8]) -> Result<Self, XlsxError> {
        let xml = info
            .get(SIZE_PREFIX..)
            .ok_or_else(|| malformed("EncryptionInfo is shorter than its header"))?;
        let xml = xml.strip_prefix(b"\xef\xbb\xbf").unwrap_or(xml);
        let xml = std::str::from_utf8(xml).map_err(|_| malformed("descriptor is not UTF-8"))?;
        let parts = read_descriptor(xml)?;
        let key_data = parts.key_data.ok_or_else(|| malformed("no keyData"))?;
        let Some((password, spin_count, [input, hash, key])) = parts.password else {
            return Err(if parts.other_encryptor {
                unsupported("a key encryptor other than a password")
            } else {
                malformed("no password key encryptor")
            });
        };
        // D5: the integrity data is required; a package without it is refused, not trusted.
        let (encrypted_hmac_key, encrypted_hmac_value) = parts
            .integrity
            .ok_or_else(|| malformed("no dataIntegrity"))?;
        Ok(AgileParams {
            key_data,
            encrypted_hmac_key,
            encrypted_hmac_value,
            password,
            spin_count,
            encrypted_verifier_input: input,
            encrypted_verifier_hash: hash,
            encrypted_key_value: key,
        })
    }
}

/// Reads the descriptor's elements, at most [`MAX_XML_ELEMENTS`] of them nested at most
/// [`MAX_XML_DEPTH`] deep (SPEC-0031 D3).
fn read_descriptor(xml: &str) -> Result<AgileParts, XlsxError> {
    let mut parts = AgileParts::default();
    let mut reader = Reader::from_str(xml);
    let (mut elements, mut depth) = (0usize, 0usize);
    // The `uri` of the key encryptor being read, if inside one.
    let mut encryptor: Option<String> = None;
    loop {
        let event = reader
            .read_event()
            .map_err(|e| malformed(&format!("descriptor XML: {e}")))?;
        let (element, empty) = match &event {
            Event::Start(e) => (e, false),
            Event::Empty(e) => (e, true),
            Event::End(e) => {
                depth = depth.saturating_sub(1);
                if e.local_name().as_ref() == "keyEncryptor" {
                    encryptor = None;
                }
                continue;
            }
            Event::Eof => break,
            _ => continue,
        };
        elements += 1;
        if elements > MAX_XML_ELEMENTS || depth >= MAX_XML_DEPTH {
            return Err(malformed(
                "descriptor XML has too many or too deeply nested elements",
            ));
        }
        if !empty {
            depth += 1;
        }
        match element.local_name().as_ref() {
            "keyData" if parts.key_data.is_none() => {
                parts.key_data = Some(CipherParams::read(element)?);
            }
            "dataIntegrity" if parts.integrity.is_none() => {
                parts.integrity = Some((
                    base64_attribute(element, "encryptedHmacKey")?,
                    base64_attribute(element, "encryptedHmacValue")?,
                ));
            }
            "keyData" | "dataIntegrity" => {
                return Err(malformed("descriptor repeats keyData or dataIntegrity"));
            }
            "keyEncryptor" => {
                let uri = start_attribute(element, "uri")
                    .map_err(|e| malformed(&format!("uri: {e}")))?
                    .unwrap_or_default();
                if uri != PASSWORD_KEY_ENCRYPTOR {
                    parts.other_encryptor = true;
                }
                encryptor = (!empty).then_some(uri);
            }
            "encryptedKey" if encryptor.as_deref() == Some(PASSWORD_KEY_ENCRYPTOR) => {
                if parts.password.is_some() {
                    return Err(malformed("descriptor repeats the password key encryptor"));
                }
                let params = CipherParams::read(element)?;
                let spin_count = number(element, "spinCount")?;
                parts.password = Some((
                    params,
                    spin_count,
                    [
                        base64_attribute(element, "encryptedVerifierHashInput")?,
                        base64_attribute(element, "encryptedVerifierHashValue")?,
                        base64_attribute(element, "encryptedKeyValue")?,
                    ],
                ));
            }
            _ => {}
        }
    }
    Ok(parts)
}

/// The value of attribute `name` of `element`, which must be there.
fn required(element: &BytesStart<'_>, name: &str) -> Result<String, XlsxError> {
    start_attribute(element, name)
        .map_err(|e| malformed(&format!("{name}: {e}")))?
        .ok_or_else(|| malformed(&format!("no {name}")))
}

/// The unsigned 32-bit value of attribute `name`.
fn number(element: &BytesStart<'_>, name: &str) -> Result<u32, XlsxError> {
    let text = required(element, name)?;
    // A spin count past u32 is past the format's maximum too: refused as one, not as text.
    if name == "spinCount" && !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(text.parse().unwrap_or(u32::MAX));
    }
    text.parse()
        .map_err(|_| malformed(&format!("{name} is not a number")))
}

/// The bytes of base64 attribute `name`, its length checked before it is decoded.
fn base64_attribute(element: &BytesStart<'_>, name: &str) -> Result<Vec<u8>, XlsxError> {
    let text = required(element, name)?;
    if text.len() > MAX_BASE64_CHARS {
        return Err(malformed(&format!("{name} is too long")));
    }
    base64::engine::general_purpose::STANDARD
        .decode(text.trim())
        .map_err(|_| malformed(&format!("{name} is not base64")))
}

/// Decrypts `ciphertext` with AES-CBC into a new zeroized buffer of exactly `len` bytes (the
/// leading `len` bytes of the plain text). Too short a ciphertext is malformed.
fn decrypt_field(
    key: &[u8],
    iv: &[u8],
    ciphertext: &[u8],
    len: usize,
) -> Result<Zeroizing<Vec<u8>>, XlsxError> {
    if ciphertext.len() < len {
        return Err(malformed("an encrypted field is shorter than its value"));
    }
    let mut plain = Zeroizing::new(ciphertext.to_vec());
    aes_cbc_decrypt(key, iv, &mut plain)?;
    plain.truncate(len);
    Ok(plain)
}

fn decrypt_agile(
    params: &AgileParams,
    package: &[u8],
    password: &[u8],
    plain_size: usize,
) -> Result<Vec<u8>, XlsxError> {
    let pw = &params.password;
    // MS-OFFCRYPTO 2.3.4.11: the iterated hash, then one key per block key, each cut or padded
    // with 0x36 to the key size.
    let iterated = pw.hash.iterated(&pw.salt, password, params.spin_count);
    let block_key = |block: &[u8]| fit(&pw.hash.hash(&[&iterated, block]), pw.key_len, 0x36);
    let iv = fit(&pw.salt, BLOCK, 0x36);

    // 2.3.4.13: the password is right when the hash of the decrypted verifier input is the
    // decrypted verifier hash. Nothing else is decrypted with its keys before that holds.
    let input = decrypt_field(
        &block_key(&VERIFIER_INPUT_BLOCK),
        &iv,
        &params.encrypted_verifier_input,
        pw.salt.len(),
    )?;
    let expected = decrypt_field(
        &block_key(&VERIFIER_HASH_BLOCK),
        &iv,
        &params.encrypted_verifier_hash,
        pw.hash.len(),
    )?;
    if !bool::from(pw.hash.hash(&[&input]).ct_eq(&expected)) {
        return Err(XlsxError::WrongPassword);
    }
    let kd = &params.key_data;
    let secret = decrypt_field(
        &block_key(&KEY_VALUE_BLOCK),
        &iv,
        &params.encrypted_key_value,
        kd.key_len,
    )?;

    // 2.3.4.14 (D5): the HMAC of the whole stream, checked before any plain text is returned.
    let integrity_iv = |block: &[u8]| fit(&kd.hash.hash(&[&kd.salt, block]), BLOCK, 0x36);
    let hmac_key = decrypt_field(
        &secret,
        &integrity_iv(&HMAC_KEY_BLOCK),
        &params.encrypted_hmac_key,
        kd.hash.len(),
    )?;
    let hmac_value = decrypt_field(
        &secret,
        &integrity_iv(&HMAC_VALUE_BLOCK),
        &params.encrypted_hmac_value,
        kd.hash.len(),
    )?;
    if !kd.hash.hmac_matches(&hmac_key, package, &hmac_value) {
        return Err(malformed(
            "the encrypted package failed its integrity check",
        ));
    }

    // 2.3.4.15: 4,096-byte segments, segment i with IV H(keyData salt || i_le32).
    let mut plain = package.get(SIZE_PREFIX..).unwrap_or_default().to_vec();
    for (i, segment) in plain.chunks_mut(SEGMENT).enumerate() {
        let index = u32::try_from(i).map_err(|_| malformed("too many segments"))?;
        let iv = fit(
            &kd.hash.hash(&[&kd.salt, &index.to_le_bytes()]),
            BLOCK,
            0x36,
        );
        aes_cbc_decrypt(&secret, &iv, segment)?;
    }
    plain.truncate(plain_size);
    Ok(plain)
}

// --- Standard ------------------------------------------------------------------------------

/// What a Standard `EncryptionInfo` holds (MS-OFFCRYPTO 2.3.4.5, 2.3.2, 2.3.3).
#[derive(Debug)]
struct StandardParams {
    key_len: usize,
    salt: Vec<u8>,
    encrypted_verifier: Vec<u8>,
    encrypted_verifier_hash: Vec<u8>,
}

/// A little-endian u32 reader over a byte slice that reports running out as malformed.
struct Fields<'a>(&'a [u8]);

impl<'a> Fields<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], XlsxError> {
        if self.0.len() < n {
            return Err(malformed("EncryptionInfo is truncated"));
        }
        let (head, tail) = self.0.split_at(n);
        self.0 = tail;
        Ok(head)
    }

    fn u32(&mut self) -> Result<u32, XlsxError> {
        let bytes = self.take(4)?;
        let word = bytes
            .try_into()
            .map_err(|_| malformed("EncryptionInfo is truncated"))?;
        Ok(u32::from_le_bytes(word))
    }
}

impl StandardParams {
    fn parse(info: &[u8]) -> Result<Self, XlsxError> {
        let mut fields = Fields(info);
        fields.take(4)?; // the version, already read
        let flags = fields.u32()?;
        if flags & F_EXTERNAL != 0 {
            return Err(unsupported("Extensible encryption"));
        }
        if flags & F_CRYPTO_API == 0 || flags & F_AES == 0 {
            return Err(unsupported(
                "Standard encryption without AES (RC4 CryptoAPI)",
            ));
        }
        let header_size = fields.u32()? as usize;
        let mut header = Fields(fields.take(header_size)?);
        header.take(8)?; // Flags (a copy) and SizeExtra
        let alg_id = header.u32()?;
        let alg_hash = header.u32()?;
        let key_bits = header.u32()?;
        let key_len = match (alg_id, key_bits) {
            (0x660e, 128) => 16,
            (0x660f, 192) => 24,
            (0x6610, 256) => 32,
            _ => {
                return Err(unsupported(&format!(
                    "Standard algorithm 0x{alg_id:x} of {key_bits} bits"
                )))
            }
        };
        // AlgIDHash is SHA-1 (0x8004), or 0 meaning SHA-1 (MS-OFFCRYPTO 2.3.2).
        if !matches!(alg_hash, 0 | 0x8004) {
            return Err(unsupported(&format!("Standard hash 0x{alg_hash:x}")));
        }
        // EncryptionVerifier (2.3.3): SaltSize must be 16; the verifier hash is SHA-1's 20 bytes
        // encrypted as 32.
        if fields.u32()? != 16 {
            return Err(malformed("Standard salt is not 16 bytes"));
        }
        let salt = fields.take(16)?.to_vec();
        let encrypted_verifier = fields.take(16)?.to_vec();
        if fields.u32()? != 20 {
            return Err(malformed("Standard verifier hash is not 20 bytes"));
        }
        let encrypted_verifier_hash = fields.take(32)?.to_vec();
        Ok(StandardParams {
            key_len,
            salt,
            encrypted_verifier,
            encrypted_verifier_hash,
        })
    }
}

/// The Standard encryption key (MS-OFFCRYPTO 2.3.4.7): 50,000 rounds of SHA-1, a final hash with
/// block 0, then the `0x36` / `0x5c` expansion cut to the key size.
fn standard_key(salt: &[u8], password: &[u8], key_len: usize) -> Zeroizing<Vec<u8>> {
    let sha1 = HashAlgorithm::Sha1;
    let iterated = sha1.iterated(salt, password, STANDARD_SPIN_COUNT);
    let final_hash = sha1.hash(&[&iterated, &0u32.to_le_bytes()]);
    let expand = |fill: u8| {
        let mut buffer = Zeroizing::new([fill; 64]);
        for (b, h) in buffer.iter_mut().zip(final_hash.iter()) {
            *b ^= h;
        }
        sha1.hash(&[&buffer[..]])
    };
    let mut derived = Zeroizing::new(Vec::with_capacity(40));
    derived.extend_from_slice(&expand(0x36));
    derived.extend_from_slice(&expand(0x5c));
    derived.truncate(key_len);
    derived
}

fn decrypt_standard(
    params: &StandardParams,
    package: &[u8],
    password: &[u8],
    plain_size: usize,
) -> Result<Vec<u8>, XlsxError> {
    let key = standard_key(&params.salt, password, params.key_len);
    // 2.3.4.9: the password is right when SHA-1 of the decrypted verifier is the decrypted
    // verifier hash's first 20 bytes.
    let mut verifier = Zeroizing::new(params.encrypted_verifier.clone());
    aes_ecb_decrypt(&key, &mut verifier)?;
    let mut expected = Zeroizing::new(params.encrypted_verifier_hash.clone());
    aes_ecb_decrypt(&key, &mut expected)?;
    let actual = HashAlgorithm::Sha1.hash(&[&verifier]);
    if !bool::from(actual.ct_eq(&expected[..20])) {
        return Err(XlsxError::WrongPassword);
    }
    let mut plain = package.get(SIZE_PREFIX..).unwrap_or_default().to_vec();
    aes_ecb_decrypt(&key, &mut plain)?;
    plain.truncate(plain_size);
    Ok(plain)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    /// msoffcrypto-tool 6.1.0's doctest for `ECMA376Standard.makekey_from_password`
    /// (`method/ecma376_standard.py`): password `Password1234_`, AES-128, the salt below.
    #[test]
    fn standard_key_derivation_known_answer() {
        let salt = b"\xe8\x82fI\x0c[\xd1\xee\xbd+C\x94\xe3\xf80\xef";
        let password = password_bytes("Password1234_").unwrap();
        let key = standard_key(salt, &password, 16);
        assert_eq!(&key[..], b"@\xb1:q\xf9\x0b\x96n7T\x08\xf2\xd1\x81\xa1\xaa");
    }

    /// msoffcrypto-tool 6.1.0's doctest for `ECMA376Standard.verifykey`, with the key above.
    #[test]
    fn standard_verifier_known_answer() {
        let key = b"@\xb1:q\xf9\x0b\x96n7T\x08\xf2\xd1\x81\xa1\xaa";
        let mut verifier = b"Qos.\x96o\xac\x17\xb1\xc5\xd7\xd8\xcc6\xc9(".to_vec();
        let mut hash =
            b"+ah\xda\xbe)\x11\xad+\xd3|\x17Ft\\\x14\xd3\xcf\x1b\xb1@\xa4\x8fNo=#\x88\x08r\xb1j"
                .to_vec();
        aes_ecb_decrypt(key, &mut verifier).unwrap();
        aes_ecb_decrypt(key, &mut hash).unwrap();
        assert_eq!(&HashAlgorithm::Sha1.hash(&[&verifier])[..], &hash[..20]);
    }

    /// msoffcrypto-tool 6.1.0's doctest for `ECMA376Agile.makekey_from_password`: SHA-512,
    /// spinCount 100,000, AES-256, the intermediate key decrypted from `encryptedKeyValue`.
    #[test]
    fn agile_key_derivation_known_answer() {
        let salt = b"Lr]E\xdca\x0f\x93\x94\x12\xa0M\xa7\x91\x04f";
        let encrypted_key = b"\xa1l\xd5\x16Zz\xb9\xd2q\x11>\xd3\x86\xa7\x8c\xf4\x96\x92\xe8\xe5'\xb0\xc5\xfc\x00U\xed\x08\x0b|\xb9K";
        let password = password_bytes("Password1234_").unwrap();
        let hash = HashAlgorithm::Sha512;
        let iterated = hash.iterated(salt, &password, 100_000);
        let key = fit(&hash.hash(&[&iterated, &KEY_VALUE_BLOCK]), 32, 0x36);
        let secret = decrypt_field(&key, &fit(salt, BLOCK, 0x36), encrypted_key, 32).unwrap();
        assert_eq!(
            &secret[..],
            b"@ f\t\xd9\xfa\xad\xf2K\x07j\xeb\xf2\xc45\xb7B\x92\xc8\xb8\xa7\xaa\x81\xbcg\x9b\xe8\x97\x11\xb0*\xc2"
        );
    }

    #[test]
    fn a_short_key_pads_with_0x36_and_a_long_one_is_cut() {
        assert_eq!(&fit(&[1, 2], 4, 0x36)[..], &[1, 2, 0x36, 0x36]);
        assert_eq!(&fit(&[1, 2, 3], 2, 0x36)[..], &[1, 2]);
    }

    #[test]
    fn a_password_past_255_units_is_refused_not_truncated() {
        assert_eq!(password_bytes(&"a".repeat(255)).unwrap().len(), 510);
        assert_eq!(
            password_bytes(&"a".repeat(256)).err(),
            Some(XlsxError::WrongPassword)
        );
        // A character outside the BMP is two units.
        assert_eq!(
            password_bytes(&"\u{1F600}".repeat(128)).err(),
            Some(XlsxError::WrongPassword)
        );
    }
}
