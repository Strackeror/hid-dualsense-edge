use std::path::Path;

use anyhow::Result;
use exe::{ExportDirectory, ThunkData, VecPE};

pub fn get_exports(path: &Path) -> Result<Vec<String>> {
    let pe_file = VecPE::from_disk_file(path)?;
    let mut vec: Vec<String> = ExportDirectory::parse(&pe_file)?
        .get_export_map(&pe_file)?
        .into_keys()
        .map(|name| name.to_string())
        .filter(|n| n != "DllMain")
        .collect();
    vec.sort();
    Ok(vec)
}

pub fn get_linker_commands(path: &Path, make_orig: &[&str]) -> Result<Vec<String>> {
    let pe_file = VecPE::from_disk_file(path)?;
    Ok(ExportDirectory::parse(&pe_file)?
        .get_export_map(&pe_file)?
        .iter()
        .map(|(name, export)| {
            const SYSTEM_ROOT: &str = r"C:\Windows";

            let mut path_str = path.display().to_string();
            if path.starts_with(SYSTEM_ROOT) {
                path_str = format!(
                    r"\\.\GLOBALROOT\SystemRoot{}",
                    path_str.split_at(SYSTEM_ROOT.len()).1
                );
            }

            let mut location = format!("{}.{}", path_str, name);
            if let ThunkData::Ordinal(n) = export {
                location += &format!(",@{n}");
            }
            let mut local_name = (*name).to_owned();
            if make_orig.contains(name) {
                local_name += "_orig";
            }
            format!("/EXPORT:{local_name}={location}")
        })
        .collect())
}
