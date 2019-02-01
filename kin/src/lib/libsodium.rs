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
const STREAM_BUF_SIZE: u64 = 16384; // 16 KiB
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
    encrypted_data: Vec<u8>,
    salt: PasswordSalt,
    nonce: Vec<u8>
}

struct PasswordDerivedKey {
    data: Vec<u8>,
    salt: PasswordSalt
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

        let key = PasswordDerivedKey::generate(password)?;

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
                key.data.as_ptr()
            );
        }

        if result != 0 {
            bail!("Error while encrypting.");
        }

        let cipher_text = cipher_text; // No longer mutable

        Ok(
            EncryptedMasterKey {
                salt: key.salt,
                encrypted_data: cipher_text.to_vec(),
                nonce: nonce.to_vec()
            }
        )
    }
}

impl EncryptedMasterKey {

    pub fn new(encrypted_data: &String, salt: &String, nonce: &String) -> Result<EncryptedMasterKey, failure::Error> {
        
        let encrypted_data = base64::decode(&encrypted_data)?;
        if encrypted_data.len() != ENCRYPTED_MASTER_KEY_SIZE {
            bail!("Invalid encrypted key data.");
        }

        let salt = base64::decode(&salt)?;
        if salt.len() != PW_SALT_SIZE {
            bail!("Invalid salt data.");
        }

        let nonce = base64::decode(&nonce)?;
        if nonce.len() != SECRETBOX_NONCE_SIZE {
            bail!("Invalid nonce data.");
        }
        
        Ok(
            EncryptedMasterKey {
                encrypted_data: encrypted_data,
                salt: PasswordSalt { data: salt },
                nonce: nonce
            }
        )
    }

    pub fn salt(&self) -> String {
        base64::encode(&self.salt.data)
    }

    pub fn encrypted_key(&self) -> String {
        base64::encode(&self.encrypted_data)
    }

    pub fn nonce(&self) -> String {
        base64::encode(&self.nonce)
    }

    pub fn decrypt(&self, password: &String) -> Result<MasterKey, failure::Error> {

        let key = PasswordDerivedKey::from(password, &self.salt)?;

        let mut plain_text: [u8; MASTER_KEY_SIZE] = [0; MASTER_KEY_SIZE];

        let result;
        unsafe {
            result = rust_sodium_sys::crypto_secretbox_open_easy(
                plain_text.as_mut_ptr(),
                self.encrypted_data.as_ptr(),
                self.encrypted_data.len() as u64,
                self.nonce.as_ptr(),
                key.as_ptr()
            );
        }

        if result != 0 {
            bail!("Unable to decrypt."); // TODO: Be more specific
        }

        Ok(
            MasterKey {
                data: plain_text.to_vec()
            }
        )
    }
}

impl PasswordDerivedKey {

    fn generate(password: &String) -> Result<PasswordDerivedKey, failure::Error> {

        let salt = random_salt();

        Ok(
            PasswordDerivedKey {
                data: PasswordDerivedKey::from(password, &salt)?,
                salt: salt
            }
        )
    }

    fn from(password: &String, salt: &PasswordSalt) -> Result<Vec<u8>, failure::Error> {

        let c_password = std::ffi::CString::new(password.as_str())
            .expect("Could not convert passphase to a CString");
        let mut key: [u8; SECRETBOX_KEY_SIZE] = [0; SECRETBOX_KEY_SIZE];

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

        Ok(key.to_vec())
    }
}

pub struct EncryptingWriter<'a> {
    output: &'a mut Write,
    state: rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state
}

pub struct DecryptingWriter<'a> {
    output: &'a mut Write,
    state: rust_sodium_sys::crypto_secretstream_xchacha20poly1305_state,
    key: &'a MasterKey
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

        let mut length_remaining = total_length;

        while length_remaining > 0 {

            if length_remaining <= STREAM_BUF_SIZE {
                self.consume_data_chunk(reader, length_remaining as usize, true)?;
                length_remaining = 0;
            } else {
                self.consume_data_chunk(reader, STREAM_BUF_SIZE as usize, false)?;
                length_remaining = length_remaining - STREAM_BUF_SIZE;
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
                self.key.data.as_ptr()
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