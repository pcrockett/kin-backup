use std::fs::{ File, OpenOptions };
use std::io::{ Read, Write };
use std::path::PathBuf;
use zip::write::FileOptions;
use zip::{ CompressionMethod, ZipWriter };

pub struct KinZipWriter {
    internal: ZipWriter<File>
}

impl KinZipWriter {

    pub fn new(archive_path: &PathBuf) -> Result<KinZipWriter, failure::Error> {

        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(archive_path)?;

        let writer = KinZipWriter {
            internal: ZipWriter::new(file)
        };

        Ok(writer)
    }

    pub fn add_dir(&mut self, archive_path: &str) -> Result<(), failure::Error> {
        self.internal.add_directory(archive_path, FileOptions::default())?;
        Ok(())
    }

    pub fn add_file(&mut self, src_path: &PathBuf, archive_path: &str) -> Result<(), failure::Error> {

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

    pub fn finish(&mut self) -> Result<(), failure::Error> {
        self.internal.finish()?;
        Ok(())
    }
}
