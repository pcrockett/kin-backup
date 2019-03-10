pub use self::masterkey::{ EncryptedMasterKey, MasterKey };
pub use self::streams::{ encrypt, decrypt };

mod masterkey;
mod passphrase;
mod streams;

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

pub fn randombytes_into(buf: &mut [u8]) {
    unsafe {
        rust_sodium_sys::randombytes_buf(buf.as_mut_ptr() as *mut _, buf.len());
    }
}
