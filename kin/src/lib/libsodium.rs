use failure::{ format_err };
use rust_sodium_sys;
use std::ptr;

// stream encryption docs:
// https://download.libsodium.org/doc/secret-key_cryptography/secretstream

const KEY_SIZE: usize = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_KEYBYTES as usize;
const HEADER_SIZE: usize = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_HEADERBYTES as usize;
const A_SIZE: usize = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_ABYTES as usize;

pub fn init() -> Result<(), failure::Error> {

    unsafe {
        let result = rust_sodium_sys::sodium_init();

        if result == 0 {
            return Ok(());
        } else {
            return Err(
                failure::err_msg("error initializing libsodium")
            );
        }
    }
}

pub struct SymmetricKey {
    data: [u8; KEY_SIZE]
}

impl SymmetricKey {

    pub fn new() -> SymmetricKey {

        let mut key = SymmetricKey {
            data: [0; KEY_SIZE]
        };

        unsafe {
            rust_sodium_sys::crypto_secretstream_xchacha20poly1305_keygen(key.data.as_mut_ptr());
        }

        key
    }

    pub fn decode_base64(base64_contents: &String) -> Result<SymmetricKey, failure::Error> {

        let decoded = base64::decode(base64_contents)?;

        if decoded.len() != KEY_SIZE {
            return Err(
                format_err!("base64 data is an invalid length (was {} bytes, should be {})", decoded.len(), KEY_SIZE)
            );
        }

        let mut key = SymmetricKey {
            data: [0; KEY_SIZE]
        };

        for index in 0..KEY_SIZE {
            key.data[index] = decoded[index];
        }

        Ok(key)
    }

    pub fn encode_base64(&self) -> String {
        base64::encode(&self.data)
    }
}

pub fn randombytes_into(buf: &mut [u8]) {
    unsafe {
        rust_sodium_sys::randombytes_buf(buf.as_mut_ptr() as *mut _, buf.len());
    }
}

pub fn encrypt(plaintext: Vec<u8>, key: &SymmetricKey) -> Vec<u8> {

    // Intentionally taking ownership of plaintext. We want to discourage the
    // user from using the plain text after encryption.

    let mut header: [u8; HEADER_SIZE] = [0; HEADER_SIZE];
    let mut state = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state {
        _pad: [0; 8],
        k: [0; KEY_SIZE],
        nonce: [0; 12]
    };

    let ciphertext_len = plaintext.len() + A_SIZE;
    let mut ciphertext = vec![0; ciphertext_len];

    unsafe {

        rust_sodium_sys::crypto_secretstream_xchacha20poly1305_init_push(
            &mut state,
            header.as_mut_ptr(),
            key.data.as_ptr()
        );

        rust_sodium_sys::crypto_secretstream_xchacha20poly1305_push(
            &mut state,
            ciphertext.as_mut_ptr(),
            ptr::null_mut(),
            plaintext.as_ptr(),
            plaintext.len() as u64,
            ptr::null(),
            0,
            rust_sodium_sys::crypto_secretstream_xchacha20poly1305_tag_final()
        );
    }

    ciphertext
}
