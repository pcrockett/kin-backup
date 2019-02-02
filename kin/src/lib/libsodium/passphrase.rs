use failure::{ bail };
use rust_sodium_sys;

// key derivation docs:
// https://download.libsodium.org/doc/key_derivation

pub struct PasswordSalt {
    data: Vec<u8>
}

pub struct PasswordDerivedKey {
    pub salt: PasswordSalt,
    data: Vec<u8>
}

const SECRETBOX_KEY_SIZE: usize = rust_sodium_sys::crypto_secretbox_KEYBYTES as usize;
const SALT_SIZE: usize = rust_sodium_sys::crypto_pwhash_SALTBYTES as usize;

impl PasswordSalt {

    fn generate() -> PasswordSalt {
        let mut buf: [u8; SALT_SIZE] = [0; SALT_SIZE];
        super::randombytes_into(&mut buf);
        PasswordSalt { data: buf.to_vec() }
    }

    pub fn from(base64: &String) -> Result<PasswordSalt, failure::Error> {

        let data = base64::decode(&base64)?;
        if data.len() != SALT_SIZE {
            bail!("Invalid salt data.");
        }

        Ok(
            PasswordSalt {
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

impl PasswordDerivedKey {

    pub fn generate(password: &String) -> Result<PasswordDerivedKey, failure::Error> {

        let salt = PasswordSalt::generate();
        PasswordDerivedKey::from(password, &salt)
    }

    pub fn from(password: &String, salt: &PasswordSalt) -> Result<PasswordDerivedKey, failure::Error> {

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
                salt.as_ptr(),
                rust_sodium_sys::crypto_pwhash_OPSLIMIT_SENSITIVE as u64,
                rust_sodium_sys::crypto_pwhash_MEMLIMIT_SENSITIVE as usize,
                rust_sodium_sys::crypto_pwhash_ALG_ARGON2ID13 as i32
            );
        }

        if result != 0 {
            bail!("Ran out of memory during key derivation.");
        }

        Ok(
            PasswordDerivedKey {
                data: key.to_vec(),
                salt: PasswordSalt {
                    data: salt.data.clone()
                }
            }
        )
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
}
