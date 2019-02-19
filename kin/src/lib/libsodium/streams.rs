use super::masterkey::{ MasterKey, MASTER_KEY_SIZE };
use failure::{ bail };
use std::io;
use std::io::{ Read, Write, ErrorKind };
use std::ptr;

// stream encryption docs:
// https://download.libsodium.org/doc/secret-key_cryptography/secretstream

pub struct DecryptingWriter<'a> {
    output: &'a mut Write,
    state: rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state,
    key: &'a MasterKey
}

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

fn init_encrypt(key: &MasterKey, output: &mut Write) -> Result<rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state, failure::Error> {

    let mut header: [u8; STREAM_HEADER_SIZE] = [0; STREAM_HEADER_SIZE];
    let mut state = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state {
        _pad: [0; 8],
        k: [0; MASTER_KEY_SIZE],
        nonce: [0; 12]
    };

    unsafe {
        rust_sodium_sys::crypto_secretstream_xchacha20poly1305_init_push(
            &mut state,
            header.as_mut_ptr(),
            key.as_ptr()
        );
    }

    output.write(&header)?;

    Ok(state)
}

fn read_chunk(buffer: &mut [u8; STREAM_BUF_SIZE as usize], reader: &mut Read) -> Result<usize, std::io::Error> {

    let iterator = reader.bytes();
    let mut byte_count = 0;
    for byte in iterator {
        match byte {
            Ok(b) => buffer[byte_count] = b,
            Err(e) => return Err(e)
        }

        byte_count = byte_count + 1;
        if byte_count >= STREAM_BUF_SIZE as usize {
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

impl<'a> DecryptingWriter<'a> {

    pub fn new(key: &'a MasterKey, output: &'a mut Write) -> DecryptingWriter<'a> {

        let state = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state {
            _pad: [0; 8],
            k: [0; MASTER_KEY_SIZE],
            nonce: [0; 12]
        };

        DecryptingWriter {
            key: key,
            output: output,
            state: state
        }
    }

    pub fn consume(&mut self, reader: &mut Read, total_length: u64) -> io::Result<()> {

        let mut header: [u8; STREAM_HEADER_SIZE] = [0; STREAM_HEADER_SIZE];
        reader.read_exact(&mut header)?;

        unsafe {
            rust_sodium_sys::crypto_secretstream_xchacha20poly1305_init_pull(
                &mut self.state,
                header.as_mut_ptr(),
                self.key.as_ptr()
            );
        }

        let mut length_remaining = total_length - STREAM_HEADER_SIZE as u64;
        const DECRYPT_BUF_SIZE: u64 = STREAM_BUF_SIZE + A_SIZE as u64;

        while length_remaining > 0 {

            if length_remaining <= DECRYPT_BUF_SIZE {
                self.consume_data_chunk(reader, length_remaining as usize)?;
                length_remaining = 0;
            } else {
                self.consume_data_chunk(reader, DECRYPT_BUF_SIZE as usize)?;
                length_remaining = length_remaining - DECRYPT_BUF_SIZE;
            }
        }

        self.output.flush()?;

        Ok(())
    }

    fn consume_data_chunk(&mut self, reader: &mut Read, length_bytes: usize)
        -> io::Result<()> {

        let mut buf = vec![0; length_bytes];
        reader.read_exact(buf.as_mut_slice())?;
        self.write_decrypted(buf.as_slice())?;
        Ok(())
    }

    fn write_decrypted(&mut self, buf: &[u8]) -> io::Result<usize> {

        let plaintext_len = STREAM_BUF_SIZE as usize;
        let mut plaintext = vec![0; plaintext_len];
        let mut tag: u8 = 0;

        let result;
        unsafe {
            result = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_pull(
                &mut self.state,
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
            return Err(
                io::Error::new(ErrorKind::Other, "libsodium decryption failed")
            );
        }

        self.output.write(plaintext.as_slice())
    }
}