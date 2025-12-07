use std::env;
use std::fs; 
use std::path::Path; 
use std::process::{Command, exit};

static VERSION: &str = env!("CARGO_PKG_VERSION"); 

/// Launches a cargo command with the given command name and arguments.
/// Returns the exit status.
fn run_cargo(cmd: &str, args: &[String]) -> i32 {
    let status = Command::new("cargo")
        .arg(cmd)
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to run cargo {}: {}", cmd, e);
            exit(1);
        });
    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
    status.code().unwrap_or(0)
} 

/// Initialize hotaru in an existing Cargo project.
/// This function creates templates and programfiles directories,
/// and adds a build.rs file.
fn init_project() {
    // Check if we're in a Cargo project
    if !Path::new("Cargo.toml").exists() {
        eprintln!("Error: No Cargo.toml found in current directory.");
        eprintln!("Please run 'hotaru init' inside an existing Cargo project.");
        exit(1);
    }
    
    // Create templates directory
    let templates_path = Path::new("templates");
    if !templates_path.exists() {
        if let Err(e) = fs::create_dir_all(&templates_path) {
            eprintln!("Failed to create templates directory: {}", e);
            exit(1);
        }
        println!("Created templates directory");
    } else {
        println!("Templates directory already exists");
    }
    
    // Create programfiles directory
    let programfiles_path = Path::new("programfiles");
    if !programfiles_path.exists() {
        if let Err(e) = fs::create_dir_all(&programfiles_path) {
            eprintln!("Failed to create programfiles directory: {}", e);
            exit(1);
        }
        println!("Created programfiles directory");
    } else {
        println!("Programfiles directory already exists");
    }
    
    // Create build.rs
    let build_path = Path::new("build.rs");
    if !build_path.exists() {
        fs::write(&build_path, BUILD_RS).unwrap_or_else(|e| {
            eprintln!("Failed to write build.rs: {}", e);
            exit(1);
        });
        println!("Created build.rs");
    } else {
        println!("build.rs already exists");
    }
    
    println!("\nHotaru has been initialized in this project!");
    println!("Make sure to add the following to your Cargo.toml:");
    println!("\n[package]");
    println!("build = \"build.rs\"");
    println!("\n[dependencies]");
    println!("hotaru = \"{VERSION}\"");
    println!("{}", DEPS);
}

/// Creates a new project with the given app name.
/// This function calls `cargo new <app_name>`, then creates a default main.rs,
/// updates Cargo.toml with extra dependencies, and creates a new templates directory
/// at the same level as the src folder.
fn create_new_project(app_name: &str) {
    // Run `cargo new <app_name>`
    let status = Command::new("cargo")
        .arg("new")
        .arg(app_name)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to run cargo new: {}", e);
            exit(1);
        });
    if !status.success() {
        exit(status.code().unwrap_or(1));
    } 

    // Write the new main.rs to the src directory of the new project.
    let src_path = Path::new(app_name).join("src").join("main.rs");
    fs::write(&src_path, MAIN_RS_CONTENT).unwrap_or_else(|e| {
        eprintln!("Failed to write to {}: {}", src_path.display(), e);
        exit(1);
    });
    println!("Created new main.rs at {}", src_path.display()); 

    // Write the build.rs to the src directory of the new project.
    let src_path = Path::new(app_name).join("build.rs");
    fs::write(&src_path, BUILD_RS).unwrap_or_else(|e| {
        eprintln!("Failed to write to {}: {}", src_path.display(), e);
        exit(1);
    });
    println!("Build script created at {}", src_path.display());

    // Update Cargo.toml with the extra dependencies.
    let cargo_toml_path = Path::new(app_name).join("Cargo.toml");

    let cargo_toml_cont = format!(
        r#"[package]
name = "{app_name}"
version = "0.1.0"
edition = "2024" 
build = "build.rs" 

[dependencies]
hotaru = "{VERSION}"
{DEPS}"#, 
    );  
    fs::write(&cargo_toml_path, cargo_toml_cont).unwrap_or_else(|e| {
        eprintln!("Failed to write to {}: {}", src_path.display(), e);
        exit(1);
    }); 
    println!("Updated Cargo.toml with extra dependencies and build script."); 

    // Create a new templates directory at the same level as src.
    let templates_path = Path::new(app_name).join("templates");
    if let Err(e) = fs::create_dir_all(&templates_path) {
        eprintln!("Failed to create templates directory: {}", e);
        exit(1);
    } 
    println!("Created templates directory at {}", templates_path.display()); 

    // Create a new program files directory at the same level as src.
    let programfiles_path = Path::new(app_name).join("programfiles");
    if let Err(e) = fs::create_dir_all(&programfiles_path) {
        eprintln!("Failed to create programfiles directory: {}", e);
        exit(1);
    }
    println!("Created programfiles directory at {}", programfiles_path.display());
} 

/// Main entry point for the CLI launcher.
/// 
/// # Commands
/// 
/// - `build`: Runs `cargo build` with any extra arguments, then copies templates.
/// - `run`: Runs `cargo run` with any extra arguments.
/// - `release`: Runs `cargo build --release` with any extra arguments, then copies templates.
/// - `new <app_name>`: Creates a new project with the given name, writes a default `main.rs`
///   with Starberry code, updates `Cargo.toml` with dependencies, and creates a new templates directory.
/// - `init`: Initialize hotaru in an existing Cargo project.
/// 
/// # Example Usage
/// 
/// Build a project:
/// 
/// ```bash
/// hotaru build --verbose
/// ```
/// 
/// Run a project:
/// 
/// ```bash
/// hotaru run
/// ```
/// 
/// Build a release version:
/// 
/// ```bash
/// hotaru release --release
/// ```
/// 
/// Create a new project called `my_app`:
/// 
/// ```bash
/// hotaru new my_app
/// ```
fn main() {
    // Skip the program name.
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: hotaru <command> [arguments]");
        eprintln!(r#"Usage: hotaru <build|run|release|new|init|version> [arguments]
- `new <app_name>`: Creates a new project with the given name, a hello world program is provided by default. Dependencies are added to the Cargo.toml file. A templates directory is created at the same level as src.
- `init`: Initialize hotaru in an existing Cargo project. Creates templates/programfiles directories and build.rs.
- `build [arguments]`: Build the Starberry project (Do not use cargo build since it does not copies template). Any other extra arguments are passed to `cargo build`. 
- `run`: Runs the hotaru project. 
- `release`: Build the Starberry project in release mode (Do not use cargo build --release since it does not copies template). Any other extra arguments are passed to `cargo build`.  
- `version`: Prints the version of Starberry. 
"#);
        exit(1);
    }
    
    // Extract the command.
    let command = args.remove(0);
    
    match command.as_str() {
        "build" => {
            // Run cargo build with remaining arguments.
            let exit_code = run_cargo("build", &args); 
            exit(exit_code);
        },
        "run" => {
            // Run cargo run with remaining arguments.
            let exit_code = run_cargo("run", &args);
            exit(exit_code); 
        },
        "release" => {
            // Ensure that --release flag is passed.
            if !args.iter().any(|arg| arg == "--release") {
                args.push("--release".to_string());
            }
            let exit_code = run_cargo("build", &args); 
            exit(exit_code);
        },
        "new" => {
            if args.is_empty() {
                eprintln!("Usage: hotaru new <app_name>");
                exit(1);
            }
            let app_name = &args[0];
            create_new_project(app_name);
        },
        "init" => {
            init_project();
        },
        "version" => {
            println!("Starberry version: {}", VERSION); 
            exit(0); 
        }, 
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!(r#"Usage: hotaru <build|run|release|new|init|version> [arguments]
- `new <app_name>`: Creates a new project with the given name, a hello world program is provided by default. Dependencies are added to the Cargo.toml file. A templates directory is created at the same level as src.
- `init`: Initialize hotaru in an existing Cargo project. Creates templates/programfiles directories and build.rs.
- `build [arguments]`: Build the Starberry project (Do not use cargo build since it does not copies template). Any other extra arguments are passed to `cargo build`. 
- `run`: Runs the hotaru project. 
- `release`: Build the Starberry project in release mode (Do not use cargo build --release since it does not copies template). Any other extra arguments are passed to `cargo build`.  
- `version`: Prints the version of Starberry. 
"#);
            exit(1); 
        }
    }
}

const MAIN_RS_CONTENT: &'static str = r#"use hotaru::prelude::*;
use hotaru::http::*;

#[tokio::main]
async fn main() {
    APP.clone().run().await;
}

LApp!(APP = App::new().build());

endpoint!{
    APP.url("/"),

    /// Hello world function
    pub hello_world <HTTP> {
        text_response("Hello, world!")
    }
}
"#;  

const DEPS: &'static str = r#"ctor = "0.4.0"
once_cell = "1.17"
tokio = { version = "1", features = ["full"] }
"#; 

const BUILD_RS: &'static str = r###"//! This file is introduced in hotaru since v0.6.3-rc2 
//! Now hotaru run/build/release behaves the same as cargo run/build 
//! This file will copy and paste all templates & programfiles into the binary's dir 
//! Specially, in a workspace, it will copy and paste them into the root of workspace for direct `cargo run` 
//! The correct places to put those file is inside the crate not at the root of workspace 

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Get current crate's directory
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    
    // Determine if we're in a workspace by checking for a Cargo.toml in the parent
    // that contains [workspace]
    let is_in_workspace = manifest_dir.parent()
        .map(|parent| {
            let workspace_toml = parent.join("Cargo.toml");
            if workspace_toml.exists() {
                match fs::read_to_string(workspace_toml) {
                    Ok(content) => content.contains("[workspace]"),
                    Err(_) => false
                }
            } else {
                false
            }
        })
        .unwrap_or(false);
    
    // Determine target directory based on environment
    let (output_dir, workspace_root) = if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        // Explicit target dir specified
        (PathBuf::from(dir), None)
    } else if is_in_workspace {
        // We're in a workspace, so target is at workspace root
        let workspace_root = manifest_dir.parent()
            .expect("Failed to find workspace root");
        (workspace_root.join("target"), Some(workspace_root))
    } else {
        // Standard non-workspace crate
        (manifest_dir.join("target"), None)
    };
    
    // Get build profile (debug/release)
    let profile = env::var("PROFILE").unwrap();
    let profile_dir = output_dir.join(&profile);
    
    // Determine potential binary locations
    let mut output_dirs = vec![
        profile_dir.clone(),                // target/debug/
        // profile_dir.join(&package_name),    // target/debug/package_name/
    ];
    
    // Also try to copy to exe directory for standalone builds
    if !is_in_workspace {
        output_dirs.push(profile_dir.join("deps"));  // target/debug/deps/
    }
    
    println!("cargo:warning=Package name: {}", package_name);
    println!("cargo:warning=In workspace: {}", is_in_workspace);
    println!("cargo:warning=Manifest directory: {}", manifest_dir.display());
    println!("cargo:warning=Target directory: {}", output_dir.display());
    
    // Define assets to copy
    let assets = ["templates", "programfiles"];
    let mut copied = false;

    // Copy assets to appropriate output directories
    for dir in &output_dirs {
        for asset in &assets {
            let source = manifest_dir.join(asset);
            let destination = dir.join(asset);

            if source.exists() {
                println!("cargo:warning=Copying {} directory to {}...", asset, dir.display());
                if let Err(e) = copy_dir_all(&source, &destination) {
                    println!("cargo:warning=Failed to copy {} to {}: {}", 
                             asset, dir.display(), e);
                } else {
                    println!("cargo:warning=Successfully copied {} to {}", asset, dir.display());
                    copied = true;
                }
            } else {
                println!("cargo:warning=Skipping {} (not found at {})", asset, source.display());
            }
        }
    }
    
    // For workspace projects, also copy to workspace root for convenience
    // when running from the workspace directory
    if let Some(workspace_root) = workspace_root {
        for asset in &assets {
            let source = manifest_dir.join(asset);
            let destination = workspace_root.join(asset);

            if source.exists() {
                println!("cargo:warning=Copying {} to workspace root...", asset);
                if let Err(e) = copy_dir_all(&source, &destination) {
                    println!("cargo:warning=Failed to copy to workspace root: {}", e);
                } else {
                    println!("cargo:warning=Successfully copied to workspace root");
                    copied = true;
                }
            }
        }
    }

    if !copied {
        println!("cargo:warning=No assets were copied. Verify that 'templates' or 'programfiles' directories exist.");
    }
    
    // Create a resource locator module to help find resources at runtime
    generate_resource_locator(&manifest_dir, &profile_dir, is_in_workspace);
    
    // Tell Cargo to rerun if any of these directories change
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=programfiles");
}

/// Recursively copies a directory
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            println!("cargo:warning=Copying: {} → {}", entry.path().display(), dest_path.display());
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

/// Generate a helper module for finding resources at runtime
fn generate_resource_locator(manifest_dir: &Path, profile_dir: &Path, is_in_workspace: bool) {
    // Create src directory if it doesn't exist
    let src_dir = manifest_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir).expect("Failed to create src directory");
    }
    
    // Create a helper module for resource location
    let resource_rs = src_dir.join("resource.rs");
    let resource_module = format!(
r#"//! Resource locator module
//! Generated by build.rs - DO NOT EDIT MANUALLY

use std::path::{{Path, PathBuf}};

/// Locate a resource file or directory from any execution context
pub fn locate_resource(resource_path: &str) -> Option<PathBuf> {{
    let resource = Path::new(resource_path);
    
    // Strategy 1: Check current directory
    if resource.exists() {{
        return Some(resource.to_path_buf());
    }}
    
    // Strategy 2: Check relative to executable
    if let Ok(exe_path) = std::env::current_exe() {{
        if let Some(exe_dir) = exe_path.parent() {{
            let exe_relative = exe_dir.join(resource_path);
            if exe_relative.exists() {{
                return Some(exe_relative);
            }}
        }}
    }}
    
    // Strategy 3: Check workspace root (if applicable)
    if {is_in_workspace} {{
        if let Some(workspace_root) = find_workspace_root() {{
            let workspace_relative = workspace_root.join(resource_path);
            if workspace_relative.exists() {{
                return Some(workspace_relative);
            }}
        }}
    }}
    
    None
}}

/// Find the workspace root directory
fn find_workspace_root() -> Option<PathBuf> {{
    let mut current = std::env::current_dir().ok()?;
    
    loop {{
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {{
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {{
                if content.contains("[workspace]") {{
                    return Some(current);
                }}
            }}
        }}
        
        if !current.pop() {{
            break;
        }}
    }}
    
    None
}}
"#, is_in_workspace = is_in_workspace);

    // Only write the file if it doesn't exist or content has changed
    if !resource_rs.exists() || fs::read_to_string(&resource_rs).map_or(true, |c| c != resource_module) {
        fs::write(&resource_rs, resource_module).expect("Failed to write resource.rs file");
        println!("cargo:warning=Generated resource.rs helper module");
        
        // Make sure we include this module in lib.rs or main.rs
        add_resource_module_to_source(manifest_dir);
    }
}

/// Add the resource module to the main source file if not already present
fn add_resource_module_to_source(manifest_dir: &Path) {
    let src_dir = manifest_dir.join("src");
    
    // Check for main.rs first (for binaries)
    let main_rs = src_dir.join("main.rs");
    if main_rs.exists() {
        add_module_to_file(&main_rs);
    } else {
        // Check for lib.rs (for libraries)
        let lib_rs = src_dir.join("lib.rs");
        if lib_rs.exists() {
            add_module_to_file(&lib_rs);
        }
    }
}

/// Add a module declaration to a file if not already present
fn add_module_to_file(file_path: &Path) {
    if let Ok(content) = fs::read_to_string(file_path) {
        if !content.contains("mod resource") && !content.contains("pub mod resource") {
            let module_decl = if content.contains("pub mod") {
                "\npub mod resource;\n"
            } else {
                "\nmod resource;\n"
            };
            
            let new_content = content + module_decl;
            if let Err(e) = fs::write(file_path, new_content) {
                println!("cargo:warning=Failed to update source file: {}", e);
            } else {
                println!("cargo:warning=Added resource module to source file");
            }
        }
    }
} 
"###; 