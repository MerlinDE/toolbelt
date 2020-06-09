extern crate globwalk;

use std::fs;
use std::io::Error;
use std::path::{Path, PathBuf};

/// Copy files from one directory to another. Use a glob pattern to select the files to be copied.
///
/// # Arguments
///
/// * `source` - a String that holds the source path. It will be converted to a PathBuf.
/// * `destination` - a String that holds the destination path. It will be converted to a PathBuf.
/// * `pattern` - a standard glob pattern (e.g. *.{txt,csv} or **/*) that will be used to choose the files to be copied.
///
pub fn copy_dir_with_pattern(
    source: String,
    destination: String,
    pattern: String,
) -> Result<(), Error> {
    let source_path: PathBuf = PathBuf::from(&source).canonicalize().unwrap();
    let source_with_glob = source_path.join(pattern);
    let destination_path = PathBuf::from(destination);

    let mut existing_paths: Vec<PathBuf> = Vec::new();

    for entry in globwalk::glob(format!("{}", source_with_glob.display())).unwrap() {
        if let Ok(img) = entry {
            let mut destination_sub_path =
                img.path().strip_prefix(&source_path).unwrap().to_path_buf();
            destination_sub_path.set_file_name("");
            let complete_destination_path = destination_path.join(destination_sub_path);

            if !existing_paths.contains(&complete_destination_path) {
                existing_paths.push(complete_destination_path.clone());
                if !complete_destination_path.exists() {
                    // make sure the destination path exists
                    std::fs::create_dir_all(&complete_destination_path)?;
                }
            }

            let destination_file = complete_destination_path.join(&img.file_name());
            std::fs::copy(img.path(), destination_file)?;
        }
    }
    Ok(())
}

#[test]
fn test_copy_dir_with_pattern() {
    const SOURCE_PATH: &str = "test/my_files/";
    const DESTINATION_PATH: &str = "target/dest_files/";
    const PATTERN: &str = "*.{txt,csv,md}";

    // copy or test files
    if let Err(e) = copy_dir_with_pattern(
        String::from(SOURCE_PATH),
        String::from(DESTINATION_PATH),
        String::from(PATTERN),
    ) {
        eprintln!("Error copying files: {:?}", e);
    }

    // check the copy worked
    assert_eq!(Path::new("target/dest_files/file1.txt").exists(), true);
    assert_eq!(Path::new("target/dest_files/file2.csv").exists(), true);
    assert_eq!(
        Path::new("target/dest_files/more_files/file3.md").exists(),
        true
    );

    // clean up
    fs::remove_dir_all(DESTINATION_PATH);
}
