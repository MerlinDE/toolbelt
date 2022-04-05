# Toolbelt

Basic functionality often needed within my projects.

Currently available:

* `copy_dir_with_pattern()` - Copy files from one directory to another. Use a glob pattern to select the files to be
  copied.
* `compile_xib_to_nib` - Compile Apple style XIB files to NIB files using ibtool from Xcode
* `codesign` - Sign a package using codesign from Xcode
* `get_sdk_path` - Reads a SDK path from an environment variable and returns a PathBuf pointing to it.
* `get_sdk_include_dirs` - Returns an expanded list of header directories based on a list of paths incl. glob patterns
* `get_name_from_cargo` - Return the package name from Cargo.toml title case formatted optionally adding the version
  number
