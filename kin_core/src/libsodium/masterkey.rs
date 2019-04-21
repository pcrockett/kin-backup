use super::passphrase::{ PassphraseDerivedKey, PassphraseSalt };
use failure::{ bail, format_err };
use libsodium_sys;

// authenticated encryption docs:
// https://download.libsodium.org/doc/secret-key_cryptography/authenticated_encryption

pub struct MasterKey {
    data: Vec<u8>
}

pub struct EncryptedMasterKey {
    encrypted_data: Vec<u8>,
    salt: PassphraseSalt,
    nonce: Vec<u8>
}

pub const MASTER_KEY_SIZE: usize = libsodium_sys::crypto_secretstream_xchacha20poly1305_KEYBYTES as usize;
const ENCRYPTED_MASTER_KEY_SIZE: usize = MASTER_KEY_SIZE + SECRETBOX_MAC_SIZE;
const SECRETBOX_MAC_SIZE: usize = libsodium_sys::crypto_secretbox_MACBYTES as usize;
const SECRETBOX_NONCE_SIZE: usize = libsodium_sys::crypto_secretbox_NONCEBYTES as usize;

impl MasterKey {

    pub fn new() -> MasterKey {

        let mut key = MasterKey {
            data: vec![0; MASTER_KEY_SIZE]
        };

        unsafe {
            libsodium_sys::crypto_secretstream_xchacha20poly1305_keygen(key.data.as_mut_ptr());
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

    pub fn encrypt(&self, passphrase: &String) -> Result<EncryptedMasterKey, failure::Error> {

        let key = PassphraseDerivedKey::generate(passphrase)?;

        let mut nonce: [u8; SECRETBOX_NONCE_SIZE] = [0; SECRETBOX_NONCE_SIZE];
        super::randombytes_into(&mut nonce);
        let nonce = nonce; // No longer mutable

        let mut cipher_text: [u8; ENCRYPTED_MASTER_KEY_SIZE] = [0; ENCRYPTED_MASTER_KEY_SIZE];

        let result;
        unsafe {
            result = libsodium_sys::crypto_secretbox_easy(
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
                salt: key.salt,
                encrypted_data: cipher_text.to_vec(),
                nonce: nonce.to_vec()
            }
        )
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
}

impl EncryptedMasterKey {

    pub fn new(encrypted_data: &String, salt: &String, nonce: &String) -> Result<EncryptedMasterKey, failure::Error> {
        
        let encrypted_data = base64::decode(&encrypted_data)?;
        if encrypted_data.len() != ENCRYPTED_MASTER_KEY_SIZE {
            bail!("Invalid encrypted key data.");
        }

        let salt = PassphraseSalt::from(&salt)?;

        let nonce = base64::decode(&nonce)?;
        if nonce.len() != SECRETBOX_NONCE_SIZE {
            bail!("Invalid nonce data.");
        }
        
        Ok(
            EncryptedMasterKey {
                encrypted_data: encrypted_data,
                salt: salt,
                nonce: nonce
            }
        )
    }

    pub fn passphrase_salt(&self) -> String {
        self.salt.encode_base64()
    }

    pub fn data(&self) -> String {
        base64::encode(&self.encrypted_data)
    }

    pub fn nonce(&self) -> String {
        base64::encode(&self.nonce)
    }

    pub fn decrypt(&self, passphrase: &String) -> Result<MasterKey, failure::Error> {

        let key = PassphraseDerivedKey::from(passphrase, &self.salt)?;

        let mut plain_text: [u8; MASTER_KEY_SIZE] = [0; MASTER_KEY_SIZE];

        let result;
        unsafe {
            result = libsodium_sys::crypto_secretbox_open_easy(
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