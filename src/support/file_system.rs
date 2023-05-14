use anyhow::Result;
use std::fs;
use std::path::Path;


fn confirm_path(path: &str) -> Result<()> {
    if Path::new(path).exists() {
        return Ok(());
    }
    fs::create_dir(path)?;
    Ok(())
}