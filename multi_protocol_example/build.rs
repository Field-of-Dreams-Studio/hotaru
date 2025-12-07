//! This file is introduced in starberry since v0.6.3-rc2 
//! Now starberry run/build/release behaves the same as cargo run/build 
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
