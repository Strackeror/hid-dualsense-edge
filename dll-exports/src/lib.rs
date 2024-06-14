use std::path::Path;

use anyhow::Result;
use exe::{ExportDirectory, VecPE};

pub fn get_exports(path: &Path) -> Result<Vec<String>> {
    let pe_file = VecPE::from_disk_file(path)?;
    let vec = ExportDirectory::parse(&pe_file)?
        .get_export_map(&pe_file)?
        .into_keys()
        .map(|name| name.to_string())
        .filter(|n| n != "DllMain")
        .collect();
    Ok(vec)
}
