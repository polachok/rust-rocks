//! The `RocksDB` prelude entry.

use rocks_sys as ll;

pub use self::ll::version;

pub use error::Status;
pub use db::*;
pub use options::*;
pub use write_batch::WriteBatch;
pub use perf_level::*;
pub use table::*;
pub use slice::{CVec, PinnableSlice};
pub use types::SequenceNumber;
pub use transaction_log::LogFile;
pub use merge_operator::{AssociativeMergeOperator, MergeOperator};
pub use table_properties::{TableProperties, TablePropertiesCollection};
pub use comparator::Comparator;
pub use env::{Env, Logger};

pub use super::Result;

#[test]
fn test_version() {
    let v = version();
    assert!(v >= "5.3.1".into());
}
