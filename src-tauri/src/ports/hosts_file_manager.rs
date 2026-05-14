use crate::domain::networking::host_entry::{HostFileEntry, HostFileUpdateResult};
use crate::shared::result::app_result::AppResult;

pub trait HostsFileManager: Send + Sync {
    fn apply_entry(&self, entry: HostFileEntry) -> AppResult<HostFileUpdateResult>;
}
