use ignore::{WalkBuilder, WalkState};
use std::env;
use std::path::Path;
use std::process;
use std::fs;
use std::io;
use std::io::{BufReader, Read};
use std::fs::File;

const HELP_MESSAGE: &str = r#"fr - A simple find-replace tool for the command line

Usage: 
- fr <find_text> <replace_text>
- fr --version
- fr --help

Description:
    fr recursively finds and replaces text in files, starting from the current
    directory. fr uses .gitignore patterns if in a git repository.

Example:
    fr "old_text" "new_text"    # Replace all occurrences of "old_text" with "new_text"

Note:
    - Text matching is literal (no regular expressions)
    - Files matching .gitignore patterns are skipped
    - Only text files are processed
"#;

/// Represents the different possible command line argument outcomes
#[derive(Debug)]
enum CommandArgs<'a> {
    /// Show help message and exit
    Help,
    /// Show version and exit
    Version,
    /// Perform find and replace with the given text
    FindReplace {
        find_text: &'a str,
        replace_text: &'a str,
    },
}

/// Checks if a file is binary by reading the first 1024 bytes and checking for null bytes
/// and high ratio of non-printable characters
/// 
/// # Arguments
/// 
/// * `file_path` - Path to the file to check
/// 
/// # Returns
/// 
/// * `bool` - True if the file is binary, false otherwise
fn is_binary(file_path: &Path) -> bool {
    let Ok(file) = File::open(file_path) else {
        return false;
    };
    
    let mut reader = BufReader::new(file);
    let mut buffer = [0; 1024];
    let bytes_read = reader.read(&mut buffer).unwrap_or(0);
    
    if bytes_read == 0 {
        return false;
    }

    let mut null_bytes = 0;
    let mut non_printable = 0;
    
    for &byte in &buffer[..bytes_read] {
        if byte == 0 {
            null_bytes += 1;
        }
        if !byte.is_ascii() || (byte < 32 && byte != 9 && byte != 10 && byte != 13) {
            non_printable += 1;
        }
    }

    // Consider file binary if:
    // 1. It contains null bytes, or
    // 2. More than 30% of bytes are non-printable
    null_bytes > 0 || (non_printable as f32 / bytes_read as f32) > 0.3
}

/// Performs find and replace operation on a single file.
/// 
/// # Arguments
/// 
/// * `file_path` - Path to the file to perform find and replace on
/// * `find_text` - Text to find in the file
/// * `replace_text` - Text to replace the found text with
fn find_replace_file(file_path: &Path, find_text: &str, replace_text: &str) -> io::Result<()> {
    // Skip if not a file or if find_text is empty
    if !file_path.is_file() || find_text.is_empty() {
        return Ok(());
    }

    // Skip if the file is binary
    if is_binary(file_path) {
        return Ok(());
    }

    // Read the entire file into memory
    let content = fs::read_to_string(file_path)?;
    
    // If the text isn't found, skip writing
    if !content.contains(find_text) {
        return Ok(());
    }

    // Perform the replacement
    let new_content = content.replace(find_text, replace_text);
    
    // Write back to file
    fs::write(file_path, new_content)?;
    
    Ok(())
}

/// Recursively walks through a directory and performs find and replace operations on all files.
/// 
/// # Arguments
/// 
/// * `starting_directory` - Root directory to start the search from
/// * `find_text` - Text to find in files
/// * `replace_text` - Text to replace the found text with
fn walk_find_replace(starting_directory: &Path, find_text: &str, replace_text: &str) {
    let builder = WalkBuilder::new(starting_directory);
    builder.build_parallel().run(|| {
        Box::new(move |result| {
            if let Ok(dent) = result {
                let path = dent.path();
                if let Err(e) = find_replace_file(path, find_text, replace_text) {
                    eprintln!("Error processing {}: {}", path.display(), e);
                }
            }
            WalkState::Continue
        })
    });
}

/// Parses command line arguments and returns the appropriate command.
/// 
/// # Arguments
/// 
/// * `args` - Vector of command line arguments
/// 
/// # Returns
/// 
/// * `Result<CommandArgs, String>` - On success, returns the parsed command.
///   On failure, returns an error message.
fn parse_arguments<'a>(args: &'a [String]) -> Result<CommandArgs<'a>, String> {
    if args.len() == 2 && args[1] == "--help" {
        return Ok(CommandArgs::Help);
    }

    if args.len() == 2 && args[1] == "--version" {
        return Ok(CommandArgs::Version);
    }
    
    if args.len() != 3 {
        return Err(format!("{}\nExpected 2 arguments, got {}", 
            HELP_MESSAGE, 
            args.len().saturating_sub(1)));
    }

    if args[1].is_empty() {
        return Err("Find text cannot be empty".to_string());
    }

    Ok(CommandArgs::FindReplace {
        find_text: &args[1],
        replace_text: &args[2],
    })
}

/// Main execution function that sets up and runs the find and replace operation.
/// 
/// # Returns
/// 
/// * `Result<(), String>` - Ok(()) on success, Err with error message on failure
fn run() -> Result<(), String> {
    let starting_directory =
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let args: Vec<String> = env::args().collect();
    match parse_arguments(&args)? {
        CommandArgs::Help => {
            println!("{}", HELP_MESSAGE);
            Ok(())
        }
        CommandArgs::Version => {
            println!("fr {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        CommandArgs::FindReplace { find_text, replace_text } => {
            walk_find_replace(&starting_directory, find_text, replace_text);
            Ok(())
        }
    }
}

/// Main entry point for the program.
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::PathBuf;
    use std::process::Command;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    fn init_git_repo(dir: &Path) {
        Command::new("git")
            .arg("init")
            .current_dir(dir)
            .output()
            .expect("Failed to initialize git repository");
    }

    #[test]
    fn test_parse_arguments_help() {
        let args = vec!["fr".to_string(), "--help".to_string()];
        match parse_arguments(&args).unwrap() {
            CommandArgs::Help => assert!(true),
            _ => assert!(false, "Expected Help variant"),
        }
    }

    #[test]
    fn test_parse_arguments_find_replace() {
        let args = vec!["fr".to_string(), "find".to_string(), "replace".to_string()];
        match parse_arguments(&args).unwrap() {
            CommandArgs::FindReplace { find_text, replace_text } => {
                assert_eq!(find_text, "find");
                assert_eq!(replace_text, "replace");
            }
            _ => assert!(false, "Expected FindReplace variant"),
        }
    }

    #[test]
    fn test_parse_arguments_invalid() {
        let test_cases = vec![
            vec!["fr".to_string()],
            vec!["fr".to_string(), "find".to_string()],
        ];

        for args in test_cases {
            assert!(parse_arguments(&args).is_err(), "Should fail for args: {:?}", args);
        }
    }

    #[test]
    fn test_parse_arguments_replace_text_empty() {
        let args = vec!["fr".to_string(), "find".to_string(), "".to_string()];
        assert!(parse_arguments(&args).is_ok(), "Should not fail for empty replace text");
    }

    #[test]
    fn test_find_replace_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file(temp_dir.path(), "test.txt", "hello world");
        
        // Test successful replacement
        find_replace_file(&file_path, "hello", "hi").unwrap();
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "hi world");

        // Test no match
        find_replace_file(&file_path, "nonexistent", "new").unwrap();
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "hi world");

        // Test empty find text
        find_replace_file(&file_path, "", "new").unwrap();
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "hi world");
    }

    #[test]
    fn test_find_replace_file_errors() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent.txt");
        
        // Test non-existent file
        assert!(find_replace_file(&nonexistent_path, "find", "replace").is_ok());

        // Test directory
        assert!(find_replace_file(temp_dir.path(), "find", "replace").is_ok());
    }

    #[test]
    fn test_walk_find_replace() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        create_test_file(temp_dir.path(), "file1.txt", "hello world");
        create_test_file(temp_dir.path(), "file2.txt", "hello there");
        create_test_file(temp_dir.path(), "file3.txt", "no match");

        // Create a subdirectory with more files
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        create_test_file(&subdir, "file4.txt", "hello again");

        // Perform find and replace
        walk_find_replace(temp_dir.path(), "hello", "hi");

        // Verify results
        assert_eq!(fs::read_to_string(temp_dir.path().join("file1.txt")).unwrap(), "hi world");
        assert_eq!(fs::read_to_string(temp_dir.path().join("file2.txt")).unwrap(), "hi there");
        assert_eq!(fs::read_to_string(temp_dir.path().join("file3.txt")).unwrap(), "no match");
        assert_eq!(fs::read_to_string(subdir.join("file4.txt")).unwrap(), "hi again");
    }

    #[test]
    fn test_walk_find_replace_with_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize git repository
        init_git_repo(temp_dir.path());

        // Create .gitignore
        create_test_file(temp_dir.path(), ".gitignore", "ignored.txt\n*.log");
        
        // Create test files
        create_test_file(temp_dir.path(), "file.txt", "hello world");
        create_test_file(temp_dir.path(), "ignored.txt", "hello ignored");
        create_test_file(temp_dir.path(), "test.log", "hello log");

        // Perform find and replace
        walk_find_replace(temp_dir.path(), "hello", "hi");

        // Verify results
        assert_eq!(fs::read_to_string(temp_dir.path().join("file.txt")).unwrap(), "hi world");
        assert_eq!(fs::read_to_string(temp_dir.path().join("ignored.txt")).unwrap(), "hello ignored");
        assert_eq!(fs::read_to_string(temp_dir.path().join("test.log")).unwrap(), "hello log");
    }

    #[test]
    fn test_is_binary() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test text file (should not be binary)
        let text_file = create_test_file(temp_dir.path(), "text.txt", "Hello, world!\n");
        assert!(!is_binary(&text_file));

        // Test text file with some non-printable chars (should not be binary)
        let text_with_chars = create_test_file(temp_dir.path(), "text_with_chars.txt", "Hello\tworld\n\r");
        assert!(!is_binary(&text_with_chars));

        // Test binary file (should be binary)
        let binary_content = vec![0, 1, 2, 3, 4, 5, 0, 0, 0];
        let binary_file = temp_dir.path().join("binary.bin");
        fs::write(&binary_file, binary_content).unwrap();
        assert!(is_binary(&binary_file));

        // Test file with high ratio of non-printable chars (should be binary)
        let high_ratio = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
        let high_ratio_file = temp_dir.path().join("high_ratio.txt");
        fs::write(&high_ratio_file, high_ratio).unwrap();
        assert!(is_binary(&high_ratio_file));

        // Test empty file (should not be binary)
        let empty_file = create_test_file(temp_dir.path(), "empty.txt", "");
        assert!(!is_binary(&empty_file));

        // Test non-existent file (should not be binary)
        let nonexistent = temp_dir.path().join("nonexistent.txt");
        assert!(!is_binary(&nonexistent));
    }

    #[test]
    fn test_version_flag() {
        let args = vec!["fr".to_string(), "--version".to_string()];
        match parse_arguments(&args).unwrap() {
            CommandArgs::Version => assert!(true),
            _ => assert!(false, "Expected Version variant"),
        }
    }

    #[test]
    fn test_version_output() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("--version")
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.starts_with("fr "));
        assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
    }
}