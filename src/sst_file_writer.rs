use std::ptr;

use rocks_sys as ll;

use env::EnvOptions;
use options::Options;
use db::ColumnFamilyHandle;


// SstFileWriter is used to create sst files that can be added to database later
// All keys in files generated by SstFileWriter will have sequence number = 0
pub struct SstFileWriter {
    raw: *mut ll::rocks_sst_file_writer_t,
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
    pub fn build(&mut self) -> SstFileWriter {
        SstFileWriter {
            raw: unsafe {
                if self.use_rust_comparator {
                    ll::rocks_sst_file_writer_create_from_rust_comparator(
                        self.env_options.take().unwrap_or_default().raw(),
                        self.options.take().unwrap_or_default().raw(),
                        self.rust_comparator as *const _,
                        self.column_family,
                        self.invalidate_page_cache as u8)
                } else {
                    ll::rocks_sst_file_writer_create_from_c_comparator(self.env_options
                                                                           .take()
                                                                           .unwrap_or_default()
                                                                           .raw(),
                                                                       self.options
                                                                           .take()
                                                                           .unwrap_or_default()
                                                                           .raw(),
                                                                       self.c_comparator as
                                                                       *const _,
                                                                       self.column_family,
                                                                       self.invalidate_page_cache as
                                                                       u8)
                }
            },
        }
    }
}
