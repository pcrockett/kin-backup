pub use self::masterkey::{ EncryptedMasterKey, MasterKey };
pub use self::streams::{ encrypt, decrypt };

mod masterkey;
mod passphrase;
mod streams;

pub fn init() -> Result<(), failure::Error> {

    unsafe {
        let result = libsodium_sys::sodium_init();

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
        libsodium_sys::randombytes_buf(buf.as_mut_ptr() as *mut _, buf.len());
    }
}
