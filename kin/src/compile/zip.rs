use kin_core::Error;
use std::fs::{ File, OpenOptions };
use std::io::{ Cursor, Read, Write };
use std::path::PathBuf;
use zip::write::FileOptions;
use zip::{ CompressionMethod, ZipArchive, ZipWriter as InternalZipWriter };

pub fn extract(compressed_data: &Vec<u8>, dest_directory: &PathBuf) -> Result<(), Error> {

    let reader = Cursor::new(compressed_data);
    let mut archive = ZipArchive::new(reader)
        .expect("unable to unzip decrypt executable archive");

    for i in 0..archive.len() {

        let mut source_file = archive.by_index(i)
            .expect(&format!("unable to extract file {} from decrypt executable archive", i));

        let dest_path = dest_directory.join(source_file.name());
        let mut dest_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&dest_path)
            .expect(&format!("unable to create file {}", dest_path.to_str().unwrap()));

        std::io::copy(&mut source_file, &mut dest_file)?;

        let mode = source_file.unix_mode();
        if mode.is_some() {
            platform::set_mode(&dest_path, mode.unwrap())
                .expect(&format!("unable to set mode on {}", dest_path.to_str().unwrap()));
        }
    }

    Ok(())
}

pub struct ZipWriter {
    internal: InternalZipWriter<File>
}

impl ZipWriter {

    pub fn new(archive_path: &PathBuf) -> Result<ZipWriter, Error> {

        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(archive_path)?;

        let writer = ZipWriter {
            internal: InternalZipWriter::new(file)
        };

        Ok(writer)
    }

    pub fn add_dir(&mut self, archive_path: &str) -> Result<(), Error> {
        self.internal.add_directory(archive_path, FileOptions::default())?;
        Ok(())
    }

    pub fn add_file(&mut self, src_path: &PathBuf, archive_path: &str) -> Result<(), Error> {

        let mut file = OpenOptions::new()
            .read(true)
            .open(src_path)?;

        let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
        self.internal.start_file(archive_path, options)?;

        const BUF_SIZE: usize = 16384; // 16 KiB
        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
        let mut amount_read = file.read(&mut buf)?;

        while amount_read > 0 {
            self.internal.write_all(&buf[0..amount_read])?;
            amount_read = file.read(&mut buf)?;
        }

        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), Error> {
        self.internal.finish()?;
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use kin_core::Error;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    pub fn set_mode(path: &PathBuf, mode: u32) -> Result<(), Error> {

        // Set read-only and execute permissions for user, group, and others.
        let perms = PermissionsExt::from_mode(mode);
        std::fs::set_permissions(path, perms)?;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use kin_core::Error;
    use std::path::PathBuf;

    pub fn set_mode(path: &PathBuf, mode: u32) -> Result<(), Error> {
        // This doesn't make sense in Windows; do nothing.
        Ok(())
    }
}
