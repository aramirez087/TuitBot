//! Legacy service-account JWT signing for Google Drive.
//!
//! Used by the `ServiceAccount` auth strategy. New installs use
//! linked-account OAuth via `DriveAuthStrategy::LinkedAccount`.

use sha2::{Digest, Sha256};

use crate::source::SourceError;

/// Build a signed JWT for Google service-account auth.
///
/// Uses RS256 (RSA + SHA-256). The private key is parsed from PEM format.
pub fn build_jwt(claims: &serde_json::Value, private_key_pem: &str) -> Result<String, SourceError> {
    let header = base64_url_encode(
        &serde_json::to_vec(&serde_json::json!({"alg": "RS256", "typ": "JWT"}))
            .map_err(|e| SourceError::Auth(format!("JWT header: {e}")))?,
    );
    let payload = base64_url_encode(
        &serde_json::to_vec(claims).map_err(|e| SourceError::Auth(format!("JWT payload: {e}")))?,
    );

    let signing_input = format!("{header}.{payload}");

    let signature = rsa_sign_sha256(signing_input.as_bytes(), private_key_pem)?;
    let sig_b64 = base64_url_encode(&signature);

    Ok(format!("{signing_input}.{sig_b64}"))
}

/// RSA-SHA256 signing using minimal big-integer arithmetic.
fn rsa_sign_sha256(data: &[u8], pem: &str) -> Result<Vec<u8>, SourceError> {
    let der = pem_to_der(pem)?;
    let hash = Sha256::digest(data);
    let digest_info = build_pkcs1_digest_info(&hash);
    rsa_pkcs1_sign(&der, &digest_info)
}

/// Decode a PEM-encoded RSA private key to DER bytes.
fn pem_to_der(pem: &str) -> Result<Vec<u8>, SourceError> {
    let pem = pem.trim();
    let body: String = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<Vec<_>>()
        .join("");

    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(&body)
        .map_err(|e| SourceError::Auth(format!("PEM decode failed: {e}")))
}

/// Build PKCS#1 v1.5 DigestInfo prefix for SHA-256.
fn build_pkcs1_digest_info(hash: &[u8]) -> Vec<u8> {
    let prefix: &[u8] = &[
        0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01,
        0x05, 0x00, 0x04, 0x20,
    ];
    let mut info = prefix.to_vec();
    info.extend_from_slice(hash);
    info
}

/// Minimal RSA PKCS#1 v1.5 signing from DER-encoded private key.
fn rsa_pkcs1_sign(der: &[u8], digest_info: &[u8]) -> Result<Vec<u8>, SourceError> {
    let rsa_key = parse_pkcs8_rsa(der)?;
    let k = rsa_key.n_bytes.len();

    if digest_info.len() + 11 > k {
        return Err(SourceError::Auth("RSA key too small for signature".into()));
    }

    let mut em = vec![0x00, 0x01];
    let ps_len = k - digest_info.len() - 3;
    em.extend(std::iter::repeat(0xFF).take(ps_len));
    em.push(0x00);
    em.extend_from_slice(digest_info);

    let m = BigUint::from_bytes_be(&em);
    let n = BigUint::from_bytes_be(&rsa_key.n_bytes);
    let d = BigUint::from_bytes_be(&rsa_key.d_bytes);

    let sig = mod_pow(&m, &d, &n);
    let mut sig_bytes = sig.to_bytes_be();

    while sig_bytes.len() < k {
        sig_bytes.insert(0, 0);
    }

    Ok(sig_bytes)
}

// ---------------------------------------------------------------------------
// Minimal big-integer arithmetic for RSA
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct BigUint {
    bytes: Vec<u8>,
}

impl BigUint {
    fn from_bytes_be(b: &[u8]) -> Self {
        let start = b
            .iter()
            .position(|&x| x != 0)
            .unwrap_or(b.len().saturating_sub(1));
        Self {
            bytes: b[start..].to_vec(),
        }
    }

    fn to_bytes_be(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    fn is_zero(&self) -> bool {
        self.bytes.iter().all(|&b| b == 0)
    }

    fn bit_len(&self) -> usize {
        if self.is_zero() {
            return 0;
        }
        let top = self.bytes[0];
        (self.bytes.len() - 1) * 8 + (8 - top.leading_zeros() as usize)
    }

    fn bit(&self, i: usize) -> bool {
        let byte_idx = self.bytes.len() - 1 - i / 8;
        if byte_idx >= self.bytes.len() {
            return false;
        }
        (self.bytes[byte_idx] >> (i % 8)) & 1 == 1
    }

    fn mul_mod(a: &BigUint, b: &BigUint, m: &BigUint) -> BigUint {
        let a_len = a.bytes.len();
        let b_len = b.bytes.len();
        let mut result = vec![0u32; a_len + b_len];

        for i in (0..a_len).rev() {
            let mut carry: u32 = 0;
            for j in (0..b_len).rev() {
                let prod = (a.bytes[i] as u32) * (b.bytes[j] as u32) + result[i + j + 1] + carry;
                result[i + j + 1] = prod & 0xFF;
                carry = prod >> 8;
            }
            result[i] += carry;
        }

        let bytes: Vec<u8> = result.iter().map(|&x| x as u8).collect();
        let val = BigUint::from_bytes_be(&bytes);
        BigUint::modulo(&val, m)
    }

    fn modulo(a: &BigUint, m: &BigUint) -> BigUint {
        if a.bytes.len() < m.bytes.len() {
            return a.clone();
        }
        let mut remainder = BigUint::from_bytes_be(&[0]);
        let total_bits = a.bytes.len() * 8;

        for i in (0..total_bits).rev() {
            remainder = BigUint::shift_left_one(&remainder);
            if a.bit(i) {
                let last = remainder.bytes.len() - 1;
                remainder.bytes[last] |= 1;
            }
            if BigUint::gte(&remainder, m) {
                remainder = BigUint::sub(&remainder, m);
            }
        }
        remainder
    }

    fn shift_left_one(a: &BigUint) -> BigUint {
        let mut result = vec![0u8; a.bytes.len() + 1];
        let mut carry = 0u8;
        for i in (0..a.bytes.len()).rev() {
            let val = (a.bytes[i] as u16) * 2 + carry as u16;
            result[i + 1] = val as u8;
            carry = (val >> 8) as u8;
        }
        result[0] = carry;
        BigUint::from_bytes_be(&result)
    }

    fn gte(a: &BigUint, b: &BigUint) -> bool {
        if a.bytes.len() != b.bytes.len() {
            return a.bytes.len() > b.bytes.len();
        }
        a.bytes >= b.bytes
    }

    fn sub(a: &BigUint, b: &BigUint) -> BigUint {
        let len = a.bytes.len().max(b.bytes.len());
        let mut result = vec![0i16; len];
        let a_off = len - a.bytes.len();
        let b_off = len - b.bytes.len();

        for i in 0..a.bytes.len() {
            result[a_off + i] += a.bytes[i] as i16;
        }
        for i in 0..b.bytes.len() {
            result[b_off + i] -= b.bytes[i] as i16;
        }

        for i in (1..len).rev() {
            if result[i] < 0 {
                result[i] += 256;
                result[i - 1] -= 1;
            }
        }

        let bytes: Vec<u8> = result.iter().map(|&x| x as u8).collect();
        BigUint::from_bytes_be(&bytes)
    }
}

fn mod_pow(base: &BigUint, exp: &BigUint, modulus: &BigUint) -> BigUint {
    let bits = exp.bit_len();
    if bits == 0 {
        return BigUint::from_bytes_be(&[1]);
    }

    let mut result = BigUint::from_bytes_be(&[1]);
    let mut b = BigUint::modulo(base, modulus);

    for i in 0..bits {
        if exp.bit(i) {
            result = BigUint::mul_mod(&result, &b, modulus);
        }
        b = BigUint::mul_mod(&b, &b, modulus);
    }
    result
}

// ---------------------------------------------------------------------------
// ASN.1/DER parsing for PKCS#8 RSA keys
// ---------------------------------------------------------------------------

struct RsaKeyParts {
    n_bytes: Vec<u8>,
    d_bytes: Vec<u8>,
}

fn parse_pkcs8_rsa(der: &[u8]) -> Result<RsaKeyParts, SourceError> {
    let (_, inner) = parse_der_sequence(der)
        .map_err(|_| SourceError::Auth("invalid PKCS#8 outer SEQUENCE".into()))?;

    let rest =
        skip_der_element(inner).map_err(|_| SourceError::Auth("invalid PKCS#8 version".into()))?;

    let rest =
        skip_der_element(rest).map_err(|_| SourceError::Auth("invalid PKCS#8 algorithm".into()))?;

    let (_, pkcs1_der) = parse_der_octet_string(rest)
        .map_err(|_| SourceError::Auth("invalid PKCS#8 octet string".into()))?;

    parse_pkcs1_rsa(pkcs1_der)
}

fn parse_pkcs1_rsa(der: &[u8]) -> Result<RsaKeyParts, SourceError> {
    let (_, inner) =
        parse_der_sequence(der).map_err(|_| SourceError::Auth("invalid PKCS#1 SEQUENCE".into()))?;

    let rest =
        skip_der_element(inner).map_err(|_| SourceError::Auth("invalid PKCS#1 version".into()))?;

    let (rest, n_bytes) =
        parse_der_integer(rest).map_err(|_| SourceError::Auth("invalid PKCS#1 modulus".into()))?;

    let rest =
        skip_der_element(rest).map_err(|_| SourceError::Auth("invalid PKCS#1 exponent".into()))?;

    let (_rest, d_bytes) = parse_der_integer(rest)
        .map_err(|_| SourceError::Auth("invalid PKCS#1 private exponent".into()))?;

    Ok(RsaKeyParts { n_bytes, d_bytes })
}

fn parse_der_length(data: &[u8]) -> Result<(usize, &[u8]), ()> {
    if data.is_empty() {
        return Err(());
    }
    if data[0] < 0x80 {
        Ok((data[0] as usize, &data[1..]))
    } else {
        let num_bytes = (data[0] & 0x7F) as usize;
        if num_bytes == 0 || num_bytes > 4 || data.len() < 1 + num_bytes {
            return Err(());
        }
        let mut len: usize = 0;
        for i in 0..num_bytes {
            len = (len << 8) | data[1 + i] as usize;
        }
        Ok((len, &data[1 + num_bytes..]))
    }
}

fn parse_der_sequence(data: &[u8]) -> Result<(&[u8], &[u8]), ()> {
    if data.is_empty() || data[0] != 0x30 {
        return Err(());
    }
    let (len, rest) = parse_der_length(&data[1..])?;
    if rest.len() < len {
        return Err(());
    }
    Ok((&rest[len..], &rest[..len]))
}

fn parse_der_octet_string(data: &[u8]) -> Result<(&[u8], &[u8]), ()> {
    if data.is_empty() || data[0] != 0x04 {
        return Err(());
    }
    let (len, rest) = parse_der_length(&data[1..])?;
    if rest.len() < len {
        return Err(());
    }
    Ok((&rest[len..], &rest[..len]))
}

fn parse_der_integer(data: &[u8]) -> Result<(&[u8], Vec<u8>), ()> {
    if data.is_empty() || data[0] != 0x02 {
        return Err(());
    }
    let (len, rest) = parse_der_length(&data[1..])?;
    if rest.len() < len {
        return Err(());
    }
    let mut bytes = rest[..len].to_vec();
    if bytes.len() > 1 && bytes[0] == 0 {
        bytes.remove(0);
    }
    Ok((&rest[len..], bytes))
}

fn skip_der_element(data: &[u8]) -> Result<&[u8], ()> {
    if data.is_empty() {
        return Err(());
    }
    let (len, rest) = parse_der_length(&data[1..])?;
    if rest.len() < len {
        return Err(());
    }
    Ok(&rest[len..])
}

/// URL-safe Base64 encoding without padding.
fn base64_url_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}
