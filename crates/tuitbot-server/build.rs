use std::path::Path;
use std::{env, fs, process::Command};

fn main() {
    let out_dir = Path::new("dashboard-dist");
    let index = out_dir.join("index.html");

    // Strategy 1: dashboard-dist already populated (crates.io package).
    if index.exists() {
        println!("cargo:rerun-if-changed=dashboard-dist/");
        return;
    }

    // Allow skipping the npm build for Rust-only iteration.
    if env::var("TUITBOT_SKIP_DASHBOARD_BUILD").unwrap_or_default() == "1" {
        write_placeholder(&index, out_dir);
        return;
    }

    let dashboard_root = Path::new("../../dashboard");
    let package_json = dashboard_root.join("package.json");

    // Strategy 2: Build from source.
    if package_json.exists() {
        // Ensure dependencies are installed.
        let npm_ci = Command::new("npm")
            .arg("ci")
            .current_dir(dashboard_root)
            .status();

        if let Ok(status) = npm_ci {
            if !status.success() {
                eprintln!("warning: `npm ci` failed — falling back to placeholder dashboard");
                write_placeholder(&index, out_dir);
                return;
            }
        } else {
            eprintln!("warning: npm not found — falling back to placeholder dashboard");
            write_placeholder(&index, out_dir);
            return;
        }

        let build = Command::new("npm")
            .args(["run", "build"])
            .current_dir(dashboard_root)
            .status();

        if let Ok(status) = build {
            if !status.success() {
                eprintln!("warning: `npm run build` failed — falling back to placeholder");
                write_placeholder(&index, out_dir);
                return;
            }
        } else {
            write_placeholder(&index, out_dir);
            return;
        }

        // Copy build output to dashboard-dist/.
        let build_output = dashboard_root.join("build");
        if build_output.exists() {
            if out_dir.exists() {
                let _ = fs::remove_dir_all(out_dir);
            }
            copy_dir_recursive(&build_output, out_dir)
                .expect("failed to copy dashboard build output");
        } else {
            eprintln!("warning: dashboard build directory not found");
            write_placeholder(&index, out_dir);
        }

        println!("cargo:rerun-if-changed=../../dashboard/src/");
        println!("cargo:rerun-if-changed=../../dashboard/static/");
        println!("cargo:rerun-if-changed=../../dashboard/svelte.config.js");
        println!("cargo:rerun-if-changed=../../dashboard/vite.config.ts");
    } else {
        // Strategy 3: No dashboard source — placeholder.
        write_placeholder(&index, out_dir);
    }
}

fn write_placeholder(index: &Path, out_dir: &Path) {
    fs::create_dir_all(out_dir).expect("failed to create dashboard-dist/");
    fs::write(
        index,
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>Tuitbot</title></head>
<body style="font-family:system-ui;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;background:#0f172a;color:#e2e8f0">
<div style="text-align:center">
<h1>Tuitbot API Server</h1>
<p>Dashboard not bundled. Build from source or install the desktop app.</p>
<p style="color:#64748b;font-size:0.875rem">API available at <code>/api/</code></p>
</div>
</body>
</html>"#,
    )
    .expect("failed to write placeholder index.html");
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}
