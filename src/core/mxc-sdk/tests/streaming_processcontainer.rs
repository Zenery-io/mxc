// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Windows ProcessContainer streaming integration test, in its own
//! Windows-gated file. The sibling `streaming.rs` is `#![cfg(macos)]`, which
//! would otherwise make a `#[cfg(windows)]` test there impossible to compile.
//! Requires an elevated, host-prepped Windows host (see docs/host-prep.md), so
//! it is `#[ignore]`d.

#![cfg(target_os = "windows")]

use mxc_sdk::{
    build_request, spawn_sandbox, spawn_sandbox_with_stdio, SandboxPolicy, SandboxStdio,
    WaitOutcome,
};

fn process_container_policy(readwrite_path: &str) -> SandboxPolicy {
    SandboxPolicy {
        version: "0.7.0-alpha".to_string(),
        filesystem: Some(mxc_sdk::policy::FilesystemSection {
            readwrite_paths: vec![readwrite_path.to_string()],
            readonly_paths: vec![],
            denied_paths: vec![],
            clear_policy_on_exit: None,
        }),
        network: None,
        ui: None,
        timeout_ms: None,
        capture_denials: None,
    }
}

#[test]
#[ignore = "requires an elevated, host-prepped Windows host (see docs/host-prep.md)"]
fn streaming_processcontainer_bidirectional_stdio() {
    use std::io::{Read, Write};

    let policy = process_container_policy("C:\\Windows\\Temp");
    let mut request = build_request(&policy, None).expect("build_request");
    // `cmd /c more` echoes stdin to stdout until EOF, then exits.
    request.set_script("cmd /c more");
    let mut proc = spawn_sandbox(request).expect("spawn");

    let mut stdin = proc.take_stdin().expect("stdin available");
    let mut stdout = proc.take_stdout().expect("stdout available");

    stdin.write_all(b"ping-pong\r\n").expect("write stdin");
    drop(stdin);

    let mut out = String::new();
    stdout.read_to_string(&mut out).expect("read stdout");
    assert!(out.contains("ping-pong"), "got: {:?}", out);

    assert_eq!(proc.wait().expect("wait"), WaitOutcome::Exited(0));
}

#[test]
#[ignore = "requires an elevated, host-prepped Windows host (see docs/host-prep.md)"]
fn streaming_processcontainer_can_inherit_stdio() {
    let test_directory =
        std::env::temp_dir().join(format!("mxc-inherited-stdio-{}", std::process::id()));
    std::fs::create_dir_all(&test_directory).expect("create test directory");
    let path = test_directory.to_string_lossy();
    let mut request = build_request(&process_container_policy(&path), None).expect("build_request");
    request.set_script("cmd /c exit 0");
    let mut process = spawn_sandbox_with_stdio(request, SandboxStdio::Inherit).expect("spawn");

    assert!(process.take_stdin().is_none());
    assert!(process.take_stdout().is_none());
    assert!(process.take_stderr().is_none());
    assert_eq!(process.wait().expect("wait"), WaitOutcome::Exited(0));
    std::fs::remove_dir_all(test_directory).expect("remove test directory");
}
