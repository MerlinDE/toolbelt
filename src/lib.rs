#![crate_name = "toolbelt"]
extern crate globwalk;
#[macro_use]
extern crate log;

use std::fmt::Display;
use std::{
    io::Error,
    path::{Path, PathBuf},
    process::Command,
};

use glob::glob_with;
use glob::MatchOptions;
use inflector::cases::titlecase::to_title_case;

pub fn version() -> u32 {
    ((env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap() & 7) << 19)
        | ((env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap() & 15) << 15)
        | ((env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap() & 15) << 11)
        | ((env!("CARGO_PKG_VERSION_PRE").parse::<u32>().unwrap_or(0) & 511) << 0)
}

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

    for entry in globwalk::glob(format!("{}", source_with_glob.display()))
        .unwrap()
        .flatten()
    {
        let mut destination_sub_path = entry
            .path()
            .strip_prefix(&source_path)
            .unwrap()
            .to_path_buf();
        destination_sub_path.set_file_name("");
        let complete_destination_path = destination_path.join(destination_sub_path);

        if !existing_paths.contains(&complete_destination_path) {
            existing_paths.push(complete_destination_path.clone());
            if !complete_destination_path.exists() {
                // make sure the destination path exists
                std::fs::create_dir_all(&complete_destination_path)?;
            }
        }

        let destination_file = complete_destination_path.join(&entry.file_name());
        std::fs::copy(entry.path(), destination_file)?;
    }
    Ok(())
}

#[test]
fn test_copy_dir_with_pattern() {
    use std::fs;

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

    for entry in globwalk::glob(source_with_glob.to_str().unwrap())
        .unwrap()
        .flatten()
    {
        // TODO: preserve source directory structure at destination
        let mut nib_path = PathBuf::from(destination);
        nib_path = nib_path.join(entry.file_name());
        nib_path.set_extension("nib");
        debug!("{:?}", &nib_path);

        debug!(
            "Compile xib from {:?} to {:?}",
            entry.path().display(),
            nib_path.display()
        );
        let _compile_xibs = Command::new("ibtool")
            .arg("--compile")
            .arg(nib_path)
            .arg(entry.path())
            .output()
            // .unwrap();
            .map_err(|_| "Failed to run compile xibs.".to_string());
    }
}

/// Sign a package using codesign from Xcode
///
/// # Arguments
///
/// * `package` - Path to the package's root folder
///
/// Attention: No error handling in place yet.
///
pub fn codesign(package: &Path) {
    let _signer = Command::new("codesign")
        .arg("--force")
        .arg("--sign")
        .arg("-")
        .arg(package)
        .output()
        // .unwrap();
        .map_err(|_| "Failed to sign package.".to_string());
}

/// Reads a SDK path from an environment variable and returns a PathBuf pointing to it.
///
/// # Arguments
///
/// * `sdk_name` – A string containing the name of the environment variable that shall contain the SDK path
///
/// # Example
///
/// ```no run
/// use toolbelt::get_sdk_path;
/// let sdk_path = get_sdk_path(env!("THE_SDK"));
/// ```
pub fn get_sdk_path(sdk_name: &str) -> PathBuf {
    let sdk_path: PathBuf = sdk_name
        .to_string()
        .parse()
        .unwrap_or_else(|_| panic!("{} env variable configuration error.", sdk_name));

    if !sdk_path.exists() {
        eprintln!(
            "Please download & unpack the SDK into {}",
            sdk_path.display()
        );
        std::process::exit(1);
    }

    sdk_path
}

pub enum IncludeDirFormat {
    PLAIN,
    CLANG,
}

/// Returns an expanded list of header directories based on a list of paths incl. glob patterns
///
/// # Arguments
///
/// * `sdk_header_dirs` – List of glob patterns for directories to includem
/// * `sdk_path` - Root SDK path. Header directories will relative to this one
/// * `format` – Format of returned directories. One of
///     * IncludeDirFormat::PLAIN for a plain list
///     * IncludeDirFormat::CLANG for clang style format (starting with `-I`)
///
/// # Example
///
/// ```no run
/// use toolbelt::{get_sdk_path, get_sdk_include_dirs, IncludeDirFormat};
///
/// let sdk_path = get_sdk_path(env!("THE_SDK")).to_str().unwrap();
/// let include_dirs = [
///     "headers/common/**"];
/// get_sdk_include_dirs(include_dirs, sdk_path, IncludeDirFormat::CLANG);
/// ```
pub fn get_sdk_include_dirs<I>(
    sdk_header_dirs: I,
    sdk_path: &str,
    format: IncludeDirFormat,
) -> Vec<String>
where
    I: IntoIterator,
    I::Item: Display,
{
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let mut incl_dirs = Vec::new();
    let sdk = PathBuf::from(sdk_path);

    for hdir in sdk_header_dirs.into_iter() {
        for entry in glob_with(&format!("{}{}", sdk_path, hdir).to_string(), options)
            .expect("Failed to read glob pattern")
        {
            match entry {
                Ok(path) => {
                    let ipath = sdk.join(path);
                    match &format {
                        IncludeDirFormat::CLANG => {
                            incl_dirs.push(format!("-I{}", &ipath.display()))
                        }
                        IncludeDirFormat::PLAIN => incl_dirs.push(format!("{}", &ipath.display())),
                    }
                }
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }

    incl_dirs
}

/// Return the package name from Cargo.toml title case formatted
/// optionally adding the version number
///
/// # Arguments
///
/// * `with_version` – Include version number information
pub fn get_name_from_cargo(with_version: bool) -> String {
    let mut name = to_title_case(env!("CARGO_PKG_NAME"));
    if with_version {
        name += " ";
        name += &*String::from(env!("CARGO_PKG_VERSION"));
    }
    name
}
