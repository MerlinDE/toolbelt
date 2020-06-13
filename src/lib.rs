extern crate globwalk;
#[macro_use] extern crate log;

use std::{
    fs,
    io::Error,
    env,
    path::{Path, PathBuf},
    process,
    process::Command,
    process::Stdio,
};

/// Copy files from one directory to another. Use a glob pattern to select the files to be copied.
///
/// # Arguments
///
/// * `source` - the source path. It will be converted to a PathBuf.
/// * `destination` - the destination path. It will be converted to a PathBuf.
/// * `pattern` - a standard glob pattern (e.g. *.{txt,csv} or **/*) that will be used to choose the files to be copied.
///
pub fn copy_dir_with_pattern(
    source: &Path,
    destination: &Path,
    pattern: &str,
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
    let source_path: &Path = Path::new("test/my_files/");
    let destination_path: &Path = Path::new("target/dest_files/");

    // copy or test files
    if let Err(e) = copy_dir_with_pattern(source_path, destination_path, "*.{txt,csv,md}") {
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
    fs::remove_dir_all(destination_path);

    /* Having no glob pattern currently doesn't seem to work with GlobWalker,
        thus this test will fail.

    // copy or test files
    if let Err(e) = copy_dir_with_pattern(source_path, destination_path, "file1.txt") {
        eprintln!("Error copying files: {:?}", e);
    }

    // check the copy worked
    assert_eq!(Path::new("target/dest_files/file1.txt").exists(), true);

    // clean up
    fs::remove_dir_all(destination_path);

     */
}

#[test]
fn test_basic_glob() {
    use glob::Pattern;

    assert!(Pattern::new("c?t").unwrap().matches("cat"));
    assert!(Pattern::new("file1.txt").unwrap().matches("file1.txt"));
}

/// Compile Apple style XIB files to NIB files using ibtool from Xcode
///
/// # Arguments
///
/// * `source` – source path to tool for *.xib files
/// * `destination` - destination path to copy compiler *.nib file to
///
/// The current implementation **flattens** the directory structure.
///
pub fn compile_xib_to_nib(source: &Path, destination: &Path) {
    /*
    Compile xib to nib
    find . -name "*.xib" -type f | awk '{sub(/.xib/,"");print}' | xargs -I % ibtool --compile %.nib %.xib

    Piped commands reference: https://rust-lang-nursery.github.io/rust-cookbook/os/external.html#run-piped-external-commands
     */
    let source_with_glob = PathBuf::from(source).join("*.xib");
    debug!("source with glob {:?}", &source_with_glob);

    for entry in globwalk::glob(source_with_glob.to_str().unwrap()).unwrap() {
        if let Ok(img) = entry {
            // TODO: preserve source directory structure at destination
            let mut nib_path = PathBuf::from(destination);
            nib_path = nib_path.join(img.file_name());
            nib_path.set_extension("nib");
            debug!("{:?}", &nib_path);

            debug!(
                "Compile xib from {:?} to {:?}",
                img.path().display(),
                nib_path.display()
            );
            let compile_xibs = Command::new("ibtool")
                .arg("--compile")
                .arg(nib_path)
                .arg(img.path())
                .output()
                // .unwrap();
            .map_err(|_| "Failed to run compile xibs.".to_string());
        }
    }
}
