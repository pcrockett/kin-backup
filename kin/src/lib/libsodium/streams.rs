use super::masterkey::{ MasterKey, MASTER_KEY_SIZE };
use failure::{ bail };
use std::io::{ Read, Write };
use std::ptr;

// stream encryption docs:
// https://download.libsodium.org/doc/secret-key_cryptography/secretstream

const STREAM_HEADER_SIZE: usize = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_HEADERBYTES as usize;
const STREAM_BUF_SIZE: u64 = 16384; // 16 KiB
const A_SIZE: usize = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_ABYTES as usize;

pub fn encrypt(key: &MasterKey, input: &mut Read, output: &mut Write) -> Result<(), failure::Error> {

    let mut state = init_encrypt(&key, output)?;
    let mut buffer: [u8; STREAM_BUF_SIZE as usize] = [0; STREAM_BUF_SIZE as usize];

    loop {

        let read_count = read_chunk(&mut buffer, input)?;
        let is_final = read_count < STREAM_BUF_SIZE as usize;

        if is_final {
            let encrypted = encrypt_chunk(&mut state, &buffer[0..read_count], is_final)?;
            output.write(&encrypted)?;
            output.flush()?;
            break;
        } else {
            let encrypted = encrypt_chunk(&mut state, &buffer, is_final)?;
            output.write(&encrypted)?;
        }

    }

    Ok(())
}

pub fn decrypt(key: &MasterKey, input: &mut Read, output: &mut Write) -> Result<(), failure::Error> {

    let mut state = init_decrypt(&key, input)?;

    const DECRYPT_BUF_SIZE: usize = STREAM_BUF_SIZE as usize + A_SIZE;
    let mut buffer: [u8; DECRYPT_BUF_SIZE] = [0; DECRYPT_BUF_SIZE];

    loop {
        let read_count = read_chunk(&mut buffer, input)?;

        if read_count == 0 {
            break; // No more data to decrypt
        }

        let is_final = read_count < DECRYPT_BUF_SIZE;
        if is_final {
            let plaintext = decrypt_chunk(&mut state, &buffer[0..read_count])?;
            output.write(&plaintext)?;
            output.flush()?;
            break;
        } else {
            let plaintext = decrypt_chunk(&mut state, &buffer)?;
            output.write(&plaintext)?;
        }

    }

    Ok(())
}

fn init_encrypt(key: &MasterKey, output: &mut Write) -> Result<rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state, failure::Error> {

    let mut header: [u8; STREAM_HEADER_SIZE] = [0; STREAM_HEADER_SIZE];
    let mut state = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state {
        _pad: [0; 8],
        k: [0; MASTER_KEY_SIZE],
        nonce: [0; 12]
    };

    unsafe {
        let result = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_init_push(
            &mut state,
            header.as_mut_ptr(),
            key.as_ptr()
        );

        if result != 0 {
            bail!("unable to initialize encryption");
        }
    }

    output.write(&header)?;

    Ok(state)
}

fn read_chunk(buffer: &mut [u8], reader: &mut Read) -> Result<usize, std::io::Error> {

    let iterator = reader.bytes();
    let mut byte_count = 0;
    for byte in iterator {
        match byte {
            Ok(b) => buffer[byte_count] = b,
            Err(e) => return Err(e)
        }

        byte_count = byte_count + 1;
        if byte_count >= buffer.len() {
            break;
        }
    }

    Ok(byte_count)
}

fn encrypt_chunk(state: &mut rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state,
    buf: &[u8], is_final: bool) -> Result<Vec<u8>, failure::Error> {

    let ciphertext_len = buf.len() + A_SIZE;
    let mut ciphertext = vec![0; ciphertext_len];

    unsafe {

        let tag: u8;
        if is_final {
            tag = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_tag_final();
        } else {
            tag = 0;
        }

        let result = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_push(
            state,
            ciphertext.as_mut_ptr(),
            ptr::null_mut(),
            buf.as_ptr(),
            buf.len() as u64,
            ptr::null(),
            0,
            tag
        );

        if result != 0 {
            bail!("libsodium encryption failed");
        }
    }

    Ok(ciphertext)
}

fn init_decrypt(key: &MasterKey, reader: &mut Read) -> Result<rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state, failure::Error> {

    let mut state = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state {
        _pad: [0; 8],
        k: [0; MASTER_KEY_SIZE],
        nonce: [0; 12]
    };

    let mut header: [u8; STREAM_HEADER_SIZE] = [0; STREAM_HEADER_SIZE];
    reader.read_exact(&mut header)?;

    unsafe {
        let result = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_init_pull(
            &mut state,
            header.as_mut_ptr(),
            key.as_ptr()
        );

        if result != 0 {
            bail!("unable to initialize decryption");
        }
    }

    Ok(state)
}

fn decrypt_chunk(state: &mut rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state,
    buf: &[u8]) -> Result<Vec<u8>, failure::Error> {

    if buf.len() < A_SIZE {
        bail!("buffer size must be at least {} bytes", A_SIZE);
    }

    let plaintext_len = buf.len() - A_SIZE;
    let mut plaintext = vec![0; plaintext_len];
    let mut tag: u8 = 0;

    let result;
    unsafe {
        result = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_pull(
            state,
            plaintext.as_mut_ptr(),
            ptr::null_mut(),
            &mut tag,
            buf.as_ptr(),
            buf.len() as u64,
            ptr::null(),
            0
        );
    }

    if result != 0 {
        bail!("libsodium decryption failed");
    }

    Ok(plaintext)
}
