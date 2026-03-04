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
        .args(["add", "Future task", "-t", "7d"])
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
        .args(["freeze", "1", "-t", "7d"])
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

// ── Thaw-date ownership tests ─────────────────────────────────────────────────

/// `edit` must reject the `--thaw` / `-t` flag; thaw date is owned by `freeze`.
#[test]
fn edit_rejects_date_flag() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    // Create a task first so the binary has something to work with.
    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "Some task"])
        .output()
        .unwrap();

    // Attempt to pass --thaw to edit; clap should reject this as an unknown flag.
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["edit", "1", "--thaw", "7d"])
        .output()
        .expect("Failed to execute kelvin edit");

    assert!(
        !output.status.success(),
        "edit --thaw should fail; thaw date must not be editable via `edit`"
    );
}

/// `freeze` sets the task state to Iced and records the thaw date.
#[test]
fn freeze_sets_thaw_date_and_iced_state() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "Freeze me"])
        .output()
        .unwrap();

    // Freeze with an explicit date far in the future so it stays Iced.
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["freeze", "1", "--thaw", "30d"])
        .output()
        .expect("Failed to execute kelvin freeze");

    assert!(output.status.success(), "freeze should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Froze"), "output should confirm freeze");
    assert!(stdout.contains("Iced"), "state should be Iced after freeze");

    // The task must appear under --iced, not the default list.
    let iced_out = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["list", "--iced"])
        .output()
        .unwrap();
    let iced_stdout = String::from_utf8_lossy(&iced_out.stdout);
    assert!(
        iced_stdout.contains("Freeze me"),
        "frozen task should appear under --iced"
    );

    let default_out = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["list"])
        .output()
        .unwrap();
    let default_stdout = String::from_utf8_lossy(&default_out.stdout);
    assert!(
        !default_stdout.contains("Freeze me"),
        "frozen task should NOT appear in the default list"
    );
}

/// Editing an Iced task's title must not alter its thaw date.
#[test]
fn edit_title_does_not_change_thaw_date() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    // Add then freeze with a known date.
    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "Original title"])
        .output()
        .unwrap();

    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["freeze", "1", "--thaw", "30d"])
        .output()
        .unwrap();

    // Capture the thaw date from `show` before editing.
    let before = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["show", "1"])
        .output()
        .unwrap();
    let before_stdout = String::from_utf8_lossy(&before.stdout);
    let thaw_line_before = before_stdout
        .lines()
        .find(|l| l.contains("Thaw Date:"))
        .expect("show output should contain 'Thaw Date:'")
        .to_string();

    // Edit only the title.
    let edit_out = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["edit", "1", "--title", "Renamed title"])
        .output()
        .expect("Failed to execute kelvin edit");
    assert!(edit_out.status.success(), "edit --title should succeed");

    // Thaw date must be unchanged.
    let after = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["show", "1"])
        .output()
        .unwrap();
    let after_stdout = String::from_utf8_lossy(&after.stdout);
    let thaw_line_after = after_stdout
        .lines()
        .find(|l| l.contains("Thaw Date:"))
        .expect("show output should contain 'Thaw Date:'")
        .to_string();

    assert_eq!(
        thaw_line_before, thaw_line_after,
        "edit must not change the thaw date"
    );

    // Title must be updated.
    assert!(
        after_stdout.contains("Renamed title"),
        "title should be updated"
    );
    assert!(
        !after_stdout.contains("Original title"),
        "old title should be gone"
    );
}

// ── burn --completely tests ───────────────────────────────────────────────────

/// `burn --completely` permanently deletes the task (not visible even in --all).
#[test]
fn burn_completely_deletes_task() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    // Add a task
    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "DeleteMe"])
        .output()
        .unwrap();

    // burn --completely
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["burn", "1", "--completely"])
        .output()
        .expect("Failed to execute kelvin burn --completely");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Completely burned"),
        "output should confirm complete deletion"
    );

    // Must not appear even in --all
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["list", "--all"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("DeleteMe"),
        "completely burned task must not appear in --all"
    );
}

/// `burn -c` (short flag) is equivalent to `burn --completely`.
#[test]
fn burn_c_short_flag_deletes_task() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "ShortFlagTask"])
        .output()
        .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["burn", "1", "-c"])
        .output()
        .expect("Failed to execute kelvin burn -c");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Completely burned"));

    // Verify gone from storage
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["list", "--all"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("ShortFlagTask"));
}

/// `burn --completely` on an Evaporated task also deletes it permanently.
#[test]
fn burn_completely_evaporated_task() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "AlreadyBurned"])
        .output()
        .unwrap();

    // First burn normally -> Evaporated
    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["burn", "1"])
        .output()
        .unwrap();

    // Now completely delete it
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["burn", "1", "--completely"])
        .output()
        .expect("Failed to execute kelvin burn --completely on Evaporated task");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Completely burned"));

    // Gone from storage
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["list", "--all"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("AlreadyBurned"));
}

#[test]
fn burn_with_note() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "TaskToBurn"])
        .output()
        .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["burn", "1", "-n", "Finished early!"])
        .output()
        .expect("Failed to execute kelvin burn -n");
    assert!(output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["show", "1"])
        .output()
        .expect("Failed to execute kelvin show");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Finished early!"));
}

#[test]
fn cool_clears_note() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join(".config");

    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["add", "TaskToCool"])
        .output()
        .unwrap();

    // Burn with note
    Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["burn", "1", "-n", "Should be cleared"])
        .output()
        .unwrap();

    // Cool it back to Melted
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["cool", "1"])
        .output()
        .expect("Failed to execute kelvin cool");
    assert!(output.status.success());

    // Show and verify note is gone
    let output = Command::new(env!("CARGO_BIN_EXE_kelvin"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", &config_dir)
        .args(["show", "1"])
        .output()
        .expect("Failed to execute kelvin show");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Note:"));
    assert!(!stdout.contains("Should be cleared"));
}
