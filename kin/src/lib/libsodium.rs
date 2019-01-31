use failure::{ bail, format_err };
use rust_sodium_sys;
use std::io;
use std::io::{ Read, Write, ErrorKind };
use std::ptr;

// stream encryption docs:
// https://download.libsodium.org/doc/secret-key_cryptography/secretstream

// authenticated encryption docs:
// https://download.libsodium.org/doc/secret-key_cryptography/authenticated_encryption

// key derivation docs:
// https://download.libsodium.org/doc/key_derivation

// Constants for stream (file) encryption
const MASTER_KEY_SIZE: usize = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_KEYBYTES as usize;
const STREAM_HEADER_SIZE: usize = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_HEADERBYTES as usize;
const A_SIZE: usize = rust_sodium_sys::crypto_secretstream_xchacha20poly1305_ABYTES as usize;

// Constants for encrypting smaller bits of data
const SECRETBOX_KEY_SIZE: usize = rust_sodium_sys::crypto_secretbox_KEYBYTES as usize;
const SECRETBOX_NONCE_SIZE: usize = rust_sodium_sys::crypto_secretbox_NONCEBYTES as usize;
const SECRETBOX_MAC_SIZE: usize = rust_sodium_sys::crypto_secretbox_MACBYTES as usize;
const ENCRYPTED_MASTER_KEY_SIZE: usize = MASTER_KEY_SIZE + SECRETBOX_MAC_SIZE;

const PW_SALT_SIZE: usize = rust_sodium_sys::crypto_pwhash_SALTBYTES as usize;

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

pub struct MasterKey {
    data: Vec<u8>
}

pub struct PasswordSalt {
    data: Vec<u8>
}

pub struct EncryptedMasterKey {
    salt: PasswordSalt,
    encrypted_data: Vec<u8>
}

impl MasterKey {

    pub fn new() -> MasterKey {

        let mut key = MasterKey {
            data: vec![0; MASTER_KEY_SIZE]
        };

        unsafe {
            rust_sodium_sys::crypto_secretstream_xchacha20poly1305_keygen(key.data.as_mut_ptr());
        }

        key
    }

    pub fn decode_base64(base64_contents: &String) -> Result<MasterKey, failure::Error> {

        let decoded = base64::decode(base64_contents)?;

        if decoded.len() != MASTER_KEY_SIZE {
            return Err(
                format_err!("base64 data is an invalid length (was {} bytes, should be {})", decoded.len(), MASTER_KEY_SIZE)
            );
        }

        Ok(
            MasterKey {
                data: decoded
            }
        )
    }

    pub fn encode_base64(&self) -> String {
        base64::encode(&self.data)
    }

    pub fn encrypt(&self, password: &String) -> Result<EncryptedMasterKey, failure::Error> {

        // Need to generate a key from our password
        let c_password = std::ffi::CString::new(password.as_str())
            .expect("Could not convert passphase to a CString");
        let mut key: [u8; SECRETBOX_KEY_SIZE] = [0; SECRETBOX_KEY_SIZE];
        let salt = random_salt();

        let result;
        unsafe {
            result = rust_sodium_sys::crypto_pwhash(
                key.as_mut_ptr(),
                SECRETBOX_KEY_SIZE as u64,
                c_password.as_ptr(),
                c_password.as_bytes().len() as u64,
                salt.data.as_ptr(),
                rust_sodium_sys::crypto_pwhash_OPSLIMIT_SENSITIVE as u64,
                rust_sodium_sys::crypto_pwhash_MEMLIMIT_SENSITIVE as usize,
                rust_sodium_sys::crypto_pwhash_ALG_ARGON2ID13 as i32
            );
        }

        if result != 0 {
            bail!("Ran out of memory during key derivation.");
        }

        let key = key; // No longer mutable

        // Now we have a key. Let's encrypt the master key with the new password-based key.
        let mut nonce: [u8; SECRETBOX_NONCE_SIZE] = [0; SECRETBOX_NONCE_SIZE];
        randombytes_into(&mut nonce);
        let nonce = nonce; // No longer mutable

        let mut cipher_text: [u8; ENCRYPTED_MASTER_KEY_SIZE] = [0; ENCRYPTED_MASTER_KEY_SIZE];

        let result;
        unsafe {
            result = rust_sodium_sys::crypto_secretbox_easy(
                cipher_text.as_mut_ptr(),
                self.data.as_ptr(),
                MASTER_KEY_SIZE as u64,
                nonce.as_ptr(),
                key.as_ptr()
            );
        }

        if result != 0 {
            bail!("Error while encrypting.");
        }

        let cipher_text = cipher_text; // No longer mutable

        Ok(
            EncryptedMasterKey {
                salt: salt,
                encrypted_data: cipher_text.to_vec()
            }
        )
    }
}

impl EncryptedMasterKey {
    pub fn salt(&self) -> String {
        base64::encode(&self.salt.data)
    }

    pub fn encrypted_key(&self) -> String {
        base64::encode(&self.encrypted_data)
    }
}

pub struct EncryptingWriter<'a> {
    output: &'a mut Write,
    state: rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state
}

impl<'a> EncryptingWriter<'a> {

    pub fn new(key: &'a MasterKey, output: &'a mut Write) -> Result<EncryptingWriter<'a>, failure::Error> {

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

fn random_salt() -> PasswordSalt {

    let mut buf: [u8; PW_SALT_SIZE] = [0; PW_SALT_SIZE];
    randombytes_into(&mut buf);
    PasswordSalt { data: buf.to_vec() }
}