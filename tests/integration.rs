use std::process::Command;

/// Helper: Construct a command for the kelvin binary (uses a test data directory)
fn kelvin_cmd() -> Command {
    let cmd = Command::new(env!("CARGO_BIN_EXE_kelvin"));
    // To specify the configuration and data directory for testing
    // Set HOME to a temporary directory
    cmd
}

#[test]
fn help_displays() {
    let output = kelvin_cmd()
        .arg("--help")
        .output()
        .expect("Failed to execute kelvin");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("thermodynamic task manager"));
}

#[test]
fn version_displays() {
    let output = kelvin_cmd()
        .arg("--version")
        .output()
        .expect("Failed to execute kelvin");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kelvin"));
}

#[test]
fn add_and_list_workflow() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    // Add a task (Melted)
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "Test integration task"])
        .output()
        .expect("Failed to execute kelvin add");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Added task"));

    // List tasks
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["list"])
        .output()
        .expect("Failed to execute kelvin list");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Test integration task"));
}

#[test]
fn add_iced_and_list_iced() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    // Add an Iced task
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "Future task", "-d", "7d"])
        .output()
        .expect("Failed to execute kelvin add");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Iced"));

    // Not displayed in the default list
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["list"])
        .output()
        .expect("Failed to execute kelvin list");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Future task"));

    // Displayed with --iced
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["list", "--iced"])
        .output()
        .expect("Failed to execute kelvin list --iced");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Future task"));
}

#[test]
fn burn_removes_from_list() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    // Add a task
    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "Burnme"])
        .output()
        .expect("Failed to execute kelvin add");

    // Burn
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["burn", "1"])
        .output()
        .expect("Failed to execute kelvin burn");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Burned"));

    // Removed from the list
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["list"])
        .output()
        .expect("Failed to execute kelvin list");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Burnme"));
}

#[test]
fn cool_restores_task() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    // add -> burn -> cool
    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "Coolme"])
        .output()
        .unwrap();

    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["burn", "1"])
        .output()
        .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["cool", "1"])
        .output()
        .expect("Failed to execute kelvin cool");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Cooled"));

    // Return to the list
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["list"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Coolme"));
}

#[test]
fn show_task_details() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "Show me"])
        .output()
        .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["show", "1"])
        .output()
        .expect("Failed to execute kelvin show");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Show me"));
    assert!(stdout.contains("ID:"));
    assert!(stdout.contains("State:"));
}

#[test]
fn freeze_and_warm() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    // add -> freeze -> warm
    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "Freezeme"])
        .output()
        .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["freeze", "1", "-d", "7d"])
        .output()
        .expect("Failed to execute kelvin freeze");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Froze"));

    // warm back to Melted
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["warm", "1"])
        .output()
        .expect("Failed to execute kelvin warm");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Warmed"));
}

#[test]
fn edit_task_title() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "Old title"])
        .output()
        .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["edit", "1", "-t", "New title"])
        .output()
        .expect("Failed to execute kelvin edit");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("New title"));

    // Verify with list
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["list"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("New title"));
    assert!(!stdout.contains("Old title"));
}
