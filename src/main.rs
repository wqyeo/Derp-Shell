use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{self, Command};
use std::env::{self, var};
use std::io::ErrorKind;

use phf::phf_map;

static COMMAND_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "echo" => "builtin",
    "type" => "builtin",
    "exit" => "builtin",
    "cd" => "builtin",
    "pwd" => "builtin"
};

// Just use `home_dir()` since this is targeted for linux, not windows.
#[allow(deprecated)]
fn change_directory(dir: &str) -> Result<(), std::io::Error> {
    let mut path = PathBuf::from(dir);
    if dir.starts_with("~") {
        let home_dir = env::home_dir().expect("Failed to get home directory!");
        path = PathBuf::from(dir.replace("~", home_dir.to_str().unwrap()));
    }
    
    match env::set_current_dir(&path) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}


fn print_working_directory() -> String {
    match env::current_dir() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(e) => format!("Error PWD: {}", e),
    }
}

/// Helper function to check if a command is in the `path` folder.
/// Returns the direct path to the command if found, None otherwise.
fn try_get_command_in_path<'a>(path: &'a str, command: &'a str) -> Option<String> {
    let directory_path = format!("{}{}", path, command);
    if std::path::Path::new(&directory_path).is_file() {
        return Some(directory_path)
    }

    let file_path = format!("{}/{}", path, command);
    if std::path::Path::new(&file_path).is_file() {
        return Some(file_path)
    } 

    return None
}

fn eval_type(command: &str) -> String { 
    let trimmed_command = command.trim();
    if COMMAND_MAP.contains_key(trimmed_command) {
        return format!("{} is a shell {}", trimmed_command, COMMAND_MAP[trimmed_command])
    }

    if let Ok(path) = var("PATH") {
        let custom_path = path.trim_end_matches('/').to_owned() + "/";
        let paths: Vec<&str> = custom_path.split(':').collect();
        for path in paths {
            if let Some(command) = try_get_command_in_path(path, trimmed_command) {
                return format!("{} is {}", trimmed_command, command);
            }
        }
    }

    format!("{}: not found", trimmed_command)
}

/// Attempt to execute the given command at the executable path with optional input,
/// then return output.
fn execute_command(executable_path: &str, input: Option<&str>) -> String {
    let mut command_path = Command::new(executable_path);

    if let Some(input) = input {
        command_path.arg(input);
    }

    if let Ok(output) = command_path.output() {
        return String::from_utf8_lossy(&output.stdout).to_string().trim().to_string();
    }

    format!("{}: failed to run with output!", executable_path)
}

/// Attempt to find if command is a valid PATH executable,
/// then execute it with given inputs (optional), then return output.
fn eval_command(command: &str) -> String {
    let mut command_parts = command.splitn(2, ' ');

    if let Some(command) = command_parts.next() {
        let args = command_parts.next();
        if let Ok(path) = var("PATH") {
            let custom_path = path.trim_end_matches('/').to_owned() + "/";
            let paths: Vec<&str> = custom_path.split(':').collect();
            for path in paths {
                if let Some(command) = try_get_command_in_path(path, command) {
                    return execute_command(&command, args);
                } 
            }
        }
    }
    format!("{}: command not found", command)
}

/// If None is returned, continue reading...
fn eval(input: &str) -> Option<String> {
    let trimmed_input = input.trim();

    if trimmed_input.starts_with("exit") {
        if let Some(code) = trimmed_input.get(5..) {
            process::exit(code.parse().unwrap());
        } else {
            println!("No exit code given; Defaulting to 0.");
            process::exit(0);
        }
    }

    if trimmed_input.starts_with("echo ") {
        return Some(trimmed_input[5..].to_string());
    }

    if trimmed_input.starts_with("type ") {
        if let Some(command) = trimmed_input.get(5..) {
            return Some(eval_type(command))
        }
    }

    if trimmed_input.starts_with("pwd") {
        return Some(print_working_directory());
    }

    if trimmed_input.starts_with("cd ") {
        if let Some(path) = trimmed_input.get(3..) {
            return match change_directory(path) {
                Ok(_) => None,
                Err(e) => {
                    if e.kind() == ErrorKind::NotFound {
                        Some(format!("{}: No such file or directory", path))
                    } else {
                        Some(format!("Error in CD: {}", e))
                    }
                }
            };
        }
    }

    Some(eval_command(trimmed_input))
}

fn read() -> String {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    input
}

fn print_introduction() {
    println!("##############");
    println!("# DERP SHELL #");
    println!("##############");
    println!("\r\nMade for fun, joy and research; Dont use it as an actual shell lol.")
}

fn main() {
    print_introduction();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        
        let input = read();

        let output = eval(&input);
        if let Some(output_message) = output {
            println!("{}", output_message);
        }
    }
}
