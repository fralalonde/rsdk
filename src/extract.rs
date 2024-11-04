use std::fs::{File};
use std::{fs, io};
use std::io::BufReader;
use std::path::Path;
use flate2::bufread::GzDecoder;
use log::debug;
use tar::Archive;
use zip::ZipArchive;

pub fn extract_tgz(file: &File, work_dir: &Path) -> anyhow::Result<()> {
    let decompressed = GzDecoder::new(BufReader::new(file));
    let mut archive = Archive::new(decompressed);
    archive.unpack(work_dir)?;
    Ok(())
}

pub fn extract_zip(file: &File, work_dir: &Path) -> anyhow::Result<()> {
    debug!("unzipping");
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut zip_entry = archive.by_index(i)?;
        let outpath = work_dir.join(zip_entry.name());

        // Create directories as needed
        if zip_entry.is_dir() {
            debug!("creating dir {:?}", outpath);
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    debug!("creating parent dir {:?}", parent);
                    fs::create_dir_all(parent)?;
                }
            }
            debug!("creating file {:?}", outpath);
            let mut outfile = File::create(&outpath)?;
            debug!("writing file {:?}", outpath);
            io::copy(&mut zip_entry, &mut outfile)?;
        }
    }
    Ok(())
}
