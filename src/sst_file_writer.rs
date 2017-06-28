//! SstFileWriter is used to create sst files that can be added to database later.

use std::ptr;
use std::path::Path;
use std::slice;
use std::mem;
use std::fmt;
use std::str;

use rocks_sys as ll;

use error::Status;
use env::EnvOptions;
use options::Options;
use db::ColumnFamilyHandle;
use types::SequenceNumber;
use to_raw::ToRaw;

use super::Result;

/// ExternalSstFileInfo include information about sst files created
/// using SstFileWriter
pub struct ExternalSstFileInfo {
    raw: *mut ll::rocks_external_sst_file_info_t,
}

impl Drop for ExternalSstFileInfo {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_external_sst_file_info_destroy(self.raw);
        }
    }
}

impl fmt::Debug for ExternalSstFileInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ExternalSstFileInfo {{#{} path: {:?}, key: {:?}...{:?}, entries: {}}}",
            self.sequence_number().0,
            self.file_path(),
            String::from_utf8_lossy(self.smallest_key()),
            String::from_utf8_lossy(self.largest_key()),
            self.num_entries()
        )
    }
}


impl ExternalSstFileInfo {
    fn new() -> ExternalSstFileInfo {
        ExternalSstFileInfo { raw: unsafe { ll::rocks_external_sst_file_info_create() } }
    }

    unsafe fn from_ll(raw: *mut ll::rocks_external_sst_file_info_t) -> ExternalSstFileInfo {
        ExternalSstFileInfo { raw: raw }
    }

    pub fn file_path(&self) -> &str {
        unsafe {
            let mut size = 0;
            let ptr = ll::rocks_external_sst_file_info_get_file_path(self.raw, &mut size);
            str::from_utf8_unchecked(slice::from_raw_parts(ptr as *const _, size))
        }
    }

    pub fn smallest_key(&self) -> &[u8] {
        unsafe {
            let mut size = 0;
            let ptr = ll::rocks_external_sst_file_info_get_smallest_key(self.raw, &mut size);
            slice::from_raw_parts(ptr as *const _, size)
        }
    }

    pub fn largest_key(&self) -> &[u8] {
        unsafe {
            let mut size = 0;
            let ptr = ll::rocks_external_sst_file_info_get_largest_key(self.raw, &mut size);
            slice::from_raw_parts(ptr as *const _, size)
        }
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        unsafe { ll::rocks_external_sst_file_info_get_sequence_number(self.raw).into() }
    }

    pub fn file_size(&self) -> u64 {
        unsafe { ll::rocks_external_sst_file_info_get_file_size(self.raw) }
    }

    pub fn num_entries(&self) -> u64 {
        unsafe { ll::rocks_external_sst_file_info_get_num_entries(self.raw) }
    }

    pub fn version(&self) -> u32 {
        unsafe { ll::rocks_external_sst_file_info_get_version(self.raw) as u32 }
    }
}



/// SstFileWriter is used to create sst files that can be added to database later.
/// All keys in files generated by SstFileWriter will have sequence number = 0
pub struct SstFileWriter {
    raw: *mut ll::rocks_sst_file_writer_t,
    env_options: EnvOptions,
    options: Options,
}

impl Drop for SstFileWriter {
    fn drop(&mut self) {
        unsafe {
            ll::rocks_sst_file_writer_destroy(self.raw);
        }
    }
}

impl SstFileWriter {
    pub fn builder() -> SstFileWriterBuilder {
        SstFileWriterBuilder {
            env_options: None,
            options: None,
            c_comparator: unsafe { ll::rocks_comparator_bytewise() },
            rust_comparator: ptr::null_mut(),
            use_rust_comparator: false,
            column_family: ptr::null_mut(),
            invalidate_page_cache: true,
        }
    }

    /// Prepare SstFileWriter to write into file located at "file_path".
    pub fn open<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        unsafe {
            let mut status = mem::zeroed();
            let path = file_path.as_ref().to_str().expect("file path");
            ll::rocks_sst_file_writer_open(self.raw, path.as_ptr() as *const _, path.len(), &mut status);
            Status::from_ll(status)
        }
    }

    /// Add key, value to currently opened file
    /// REQUIRES: key is after any previously added key according to comparator.
    pub fn add(&self, key: &[u8], value: &[u8]) -> Result<()> {
        unsafe {
            let mut status = mem::zeroed();
            ll::rocks_sst_file_writer_add(
                self.raw,
                key.as_ptr() as *const _,
                key.len(),
                value.as_ptr() as *const _,
                value.len(),
                &mut status,
            );
            Status::from_ll(status)
        }
    }

    /// Finalize writing to sst file and close file.
    ///
    /// An optional ExternalSstFileInfo pointer can be passed to the function
    /// which will be populated with information about the created sst file
    pub fn finish(&self) -> Result<ExternalSstFileInfo> {
        unsafe {
            let mut status = ptr::null_mut();
            let info = ExternalSstFileInfo::new();
            ll::rocks_sst_file_writer_finish(self.raw, info.raw, &mut status);
            Status::from_ll(status).map(|_| info)
        }
    }

    /// Return the current file size.
    pub fn file_size(&self) -> u64 {
        unimplemented!()
    }
}


pub struct SstFileWriterBuilder {
    env_options: Option<EnvOptions>,
    options: Option<Options>,
    c_comparator: *const ll::rocks_c_comparator_t,
    rust_comparator: *mut (),
    use_rust_comparator: bool,
    column_family: *mut ll::rocks_column_family_handle_t,
    invalidate_page_cache: bool,
}

impl SstFileWriterBuilder {
    pub fn column_family(&mut self, cf: &ColumnFamilyHandle) -> &mut Self {
        self.column_family = cf.raw();
        self
    }

    pub fn build(&mut self) -> SstFileWriter {
        let env_options = self.env_options.take().unwrap_or_default();
        let options = self.options.take().unwrap_or_default();
        let ptr = if self.use_rust_comparator {
            unsafe {
                ll::rocks_sst_file_writer_create_from_rust_comparator(
                    env_options.raw(),
                    options.raw(),
                    self.rust_comparator as *const _,
                    self.column_family,
                    self.invalidate_page_cache as u8,
                )
            }
        } else {
            unsafe {
                ll::rocks_sst_file_writer_create_from_c_comparator(
                    env_options.raw(),
                    options.raw(),
                    self.c_comparator as *const _,
                    self.column_family,
                    self.invalidate_page_cache as u8,
                )
            }
        };
        SstFileWriter {
            raw: ptr,
            env_options: env_options,
            options: options,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sst_file_create() {
        let sst_dir = ::tempdir::TempDir::new_in(".", "sst").unwrap();

        let writer = SstFileWriter::builder().build();
        writer.open(sst_dir.path().join("./23333.sst")).unwrap();
        for i in 0..999 {
            let key = format!("B{:010}", i);
            let value = format!("ABCDEFGH{:x}IJKLMN", i);
            writer.add(key.as_bytes(), value.as_bytes()).unwrap();
        }
        let info = writer.finish().unwrap();
        println!("info => {:?}", info);
        assert_eq!(info.num_entries(), 999);
        // assert_eq!(info.version(), 2);
    }

    #[test]
    fn sst_file_create_error() {
        let sst_dir = ::tempdir::TempDir::new_in(".", "sst").unwrap();

        let writer = SstFileWriter::builder().build();
        writer.open(sst_dir.path().join("./23333.sst")).unwrap();
        assert!(writer.add(b"0000001", b"hello world").is_ok());
        let ret = writer.add(b"0000000", b"hello rust");
        assert!(ret.is_err()); // "Keys must be added in order"
    }
}
