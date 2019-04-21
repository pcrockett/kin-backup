use failure::{ bail };
use libsodium_sys;

// key derivation docs:
// https://download.libsodium.org/doc/key_derivation

pub struct PassphraseSalt {
    data: Vec<u8>
}

pub struct PassphraseDerivedKey {
    pub salt: PassphraseSalt,
    data: Vec<u8>
}

const SECRETBOX_KEY_SIZE: usize = libsodium_sys::crypto_secretbox_KEYBYTES as usize;
const SALT_SIZE: usize = libsodium_sys::crypto_pwhash_SALTBYTES as usize;

impl PassphraseSalt {

    fn generate() -> PassphraseSalt {
        let mut buf: [u8; SALT_SIZE] = [0; SALT_SIZE];
        super::randombytes_into(&mut buf);
        PassphraseSalt { data: buf.to_vec() }
    }

    pub fn from(base64: &String) -> Result<PassphraseSalt, failure::Error> {

        let data = base64::decode(&base64)?;
        if data.len() != SALT_SIZE {
            bail!("Invalid salt data.");
        }

        Ok(
            PassphraseSalt {
                data: data
            }
        )
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn encode_base64(&self) -> String {
        base64::encode(&self.data)
    }
}

impl PassphraseDerivedKey {

    pub fn generate(passphrase: &String) -> Result<PassphraseDerivedKey, failure::Error> {

        let salt = PassphraseSalt::generate();
        PassphraseDerivedKey::from(passphrase, &salt)
    }

    pub fn from(passphrase: &String, salt: &PassphraseSalt) -> Result<PassphraseDerivedKey, failure::Error> {

        let c_passphrase = std::ffi::CString::new(passphrase.as_str())
            .expect("Could not convert passphase to a CString");
        let mut key: [u8; SECRETBOX_KEY_SIZE] = [0; SECRETBOX_KEY_SIZE];

        let result;
        unsafe {
            result = libsodium_sys::crypto_pwhash(
                key.as_mut_ptr(),
                SECRETBOX_KEY_SIZE as u64,
                c_passphrase.as_ptr(),
                c_passphrase.as_bytes().len() as u64,
                salt.as_ptr(),
                libsodium_sys::crypto_pwhash_OPSLIMIT_SENSITIVE as u64,
                libsodium_sys::crypto_pwhash_MEMLIMIT_SENSITIVE as usize,
                libsodium_sys::crypto_pwhash_ALG_ARGON2ID13 as i32
            );
        }

        if result != 0 {
            bail!("Ran out of memory during key derivation.");
        }

        Ok(
            PassphraseDerivedKey {
                data: key.to_vec(),
                salt: PassphraseSalt {
                    data: salt.data.clone()
                }
            }
        )
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
}
