use rust_sodium_sys;
use std::ptr;

// stream encryption docs:
// https://download.libsodium.org/doc/secret-key_cryptography/secretstream

pub const KEY_SIZE: usize = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_KEYBYTES as usize;
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

pub fn generate_encryption_key() -> [u8; KEY_SIZE] {

    let mut buf: [u8; KEY_SIZE] = [0; KEY_SIZE];

    unsafe {
        rust_sodium_sys::crypto_secretstream_xchacha20poly1305_keygen(buf.as_mut_ptr());
    }

    buf
}

pub fn randombytes_into(buf: &mut [u8]) {
    unsafe {
        rust_sodium_sys::randombytes_buf(buf.as_mut_ptr() as *mut _, buf.len());
    }
}

pub fn encrypt(plaintext: Vec<u8>, key: &[u8; KEY_SIZE]) -> Vec<u8> {

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
            key.as_ptr()
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
