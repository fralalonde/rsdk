use flate2::bufread::GzDecoder;
use log::debug;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tar::Archive;
use zip::ZipArchive;

pub fn extract_tgz(file: &Path, work_dir: &Path) -> color_eyre::Result<()> {
    let archive_file = File::open(file)?;
    let input = BufReader::new(archive_file);

    let decoder = GzDecoder::new(input);
    let mut archive = Archive::new(decoder);
    #[cfg(unix)]
    archive.set_preserve_permissions(true);
    // #[cfg(unix)]
    // archive.set_preserve_mtime(true);
    // // #[cfg(unix)]
    // archive.set_ignore_zeros(true);

    archive.unpack(work_dir)?;
    Ok(())
}

pub fn extract_zip(file: &Path, work_dir: &Path) -> color_eyre::Result<()> {
    debug!("unzipping");
    let archive_file = File::open(file)?;
    let mut archive = ZipArchive::new(archive_file)?;
    archive.extract(work_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn extract_zip_deflate_roundtrip() {
        let tmp = std::env::temp_dir().join(format!("rsdk-ziptest-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let zip_path = tmp.join("test.zip");
        {
            let file = File::create(&zip_path).unwrap();
            let mut w = zip::ZipWriter::new(file);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            w.start_file("hello.txt", opts).unwrap();
            w.write_all(b"hello zip").unwrap();
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            w.start_file("sub/nested.txt", opts).unwrap();
            w.write_all(b"nested file").unwrap();
            w.finish().unwrap();
        }

        let out = tmp.join("out");
        extract_zip(&zip_path, &out).unwrap();
        assert_eq!(
            std::fs::read_to_string(out.join("hello.txt")).unwrap(),
            "hello zip"
        );
        assert_eq!(
            std::fs::read_to_string(out.join("sub/nested.txt")).unwrap(),
            "nested file"
        );
        std::fs::remove_dir_all(&tmp).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn extract_zip_preserves_symlinks() {
        // JDK distro zips ship symlinks (e.g. bin/java -> ../lib/jexec). zip 8's
        // extract() must recreate them as real symlinks, not regular files.
        let tmp = std::env::temp_dir().join(format!("rsdk-ziptest-symlink-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let zip_path = tmp.join("test.zip");
        {
            let file = File::create(&zip_path).unwrap();
            let mut w = zip::ZipWriter::new(file);
            let file_opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            w.start_file("jdk/lib/jexec", file_opts).unwrap();
            w.write_all(b"jexec binary").unwrap();
            w.add_symlink(
                "jdk/bin/java",
                "../lib/jexec",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            w.finish().unwrap();
        }

        let out = tmp.join("out");
        extract_zip(&zip_path, &out).unwrap();
        let link = out.join("jdk/bin/java");
        assert!(
            std::fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink(),
            "expected {} to be a symlink",
            link.display()
        );
        assert_eq!(
            std::fs::read_link(&link).unwrap(),
            std::path::PathBuf::from("../lib/jexec")
        );
        std::fs::remove_dir_all(&tmp).unwrap();
    }
}
