use std::{fs, path::PathBuf};

use crate::{
    config::{FileStorage, HistoryRecord},
    error::{Result, SyncError},
};

/// 实际文件存储
pub struct DiskStorage {
    path: PathBuf,
}

impl DiskStorage {
    /// 创建一个新的存储
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl FileStorage for DiskStorage {
    fn load(&self) -> Result<Vec<HistoryRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let buf = fs::read(&self.path)?;
        serde_json::from_slice(&buf).map_err(SyncError::Json)
    }

    fn save(&self, records: &[HistoryRecord]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let buf = serde_json::to_vec(records)?;
        fs::write(&self.path, &buf).map_err(SyncError::Io)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn test_disk_storage() {
        let json = tempfile::TempPath::from_path("test.json");

        let storage = DiskStorage::new(json.to_path_buf());
        let records = vec![
            HistoryRecord::new_with(
                1,
                PathBuf::from("a.txt"),
                PathBuf::from("a.txt"),
                Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap(),
            ),
            HistoryRecord::new_with(
                2,
                PathBuf::from("b.txt"),
                PathBuf::from("b.txt"),
                Utc.with_ymd_and_hms(2025, 1, 1, 12, 1, 0).unwrap(),
            ),
        ];

        storage.save(&records).unwrap();
        let records_loaded = storage.load().unwrap();
        assert_eq!(records, records_loaded);
    }
}
