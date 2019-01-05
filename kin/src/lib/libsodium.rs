use failure::{ format_err };
use rust_sodium_sys;
use std::io;
use std::io::{ Read, Write, ErrorKind };
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

pub struct EncryptingWriter<'a> {
    output: &'a mut Write,
    state: rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state
}

impl<'a> EncryptingWriter<'a> {

    pub fn new(key: &'a SymmetricKey, output: &'a mut Write) -> Result<EncryptingWriter<'a>, failure::Error> {

        let mut header: [u8; HEADER_SIZE] = [0; HEADER_SIZE];
        let mut state = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state {
            _pad: [0; 8],
            k: [0; KEY_SIZE],
            nonce: [0; 12]
        };

        unsafe {
            rust_sodium_sys::crypto_secretstream_xchacha20poly1305_init_push(
                &mut state,
                header.as_mut_ptr(),
                key.data.as_ptr()
            );
        }

        output.write(&header)?;

        let writer = EncryptingWriter {
            output: output,
            state: state
        };

        Ok(writer)
    }

    pub fn consume(&mut self, reader: &mut Read, total_length: u64) -> io::Result<()> {

        const BUF_SIZE: u64 = 16384; // 16 KiB
        let mut length_remaining = total_length;

        while length_remaining > 0 {

            if length_remaining <= BUF_SIZE {
                self.consume_data_chunk(reader, length_remaining as usize, true)?;
                length_remaining = 0;
            } else {
                self.consume_data_chunk(reader, BUF_SIZE as usize, false)?;
                length_remaining = length_remaining - BUF_SIZE;
            }
        }

        self.output.flush()?;

        Ok(())
    }

    fn write_encrypted(&mut self, buf: &[u8], is_final: bool) -> io::Result<usize> {

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
                &mut self.state,
                ciphertext.as_mut_ptr(),
                ptr::null_mut(),
                buf.as_ptr(),
                buf.len() as u64,
                ptr::null(),
                0,
                tag
            );

            if result != 0 {
                return Err(
                    io::Error::new(ErrorKind::Other, "libsodium encryption failed")
                );
            }
        }

        self.output.write(ciphertext.as_slice())
    }

    fn consume_data_chunk(&mut self, reader: &mut Read, length_bytes: usize, is_final: bool)
        -> io::Result<()> {

        let mut buf = vec![0; length_bytes];
        reader.read_exact(buf.as_mut_slice())?;
        self.write_encrypted(buf.as_slice(), is_final)?;
        Ok(())
    }
}

pub fn randombytes_into(buf: &mut [u8]) {
    unsafe {
        rust_sodium_sys::randombytes_buf(buf.as_mut_ptr() as *mut _, buf.len());
    }
}
