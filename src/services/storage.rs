use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write as _},
    path::Path,
};

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};

pub fn save_json<T: Serialize>(path: impl AsRef<Path>, data: &T) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = path.with_extension("json.tmp");

    {
        let mut writer = BufWriter::new(File::create(&tmp_path)?);
        serde_json::to_writer_pretty(&mut writer, data)?;
        writer.flush()?;
    }

    fs::rename(tmp_path, path)?;

    Ok(())
}

pub fn load_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<Option<T>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }

    let reader = BufReader::new(File::open(path)?);

    Ok(Some(serde_json::from_reader(reader)?))
}
