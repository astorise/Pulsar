use std::collections::HashMap;
use tempfile::TempDir;
use wasmtime::{Engine, Linker, Store};
use wasmtime_wasi::WasiCtx;

mod tachyon_bindings {
    wasmtime::component::bindgen!({
        inline: r#"
            package tachyon:test;

            world orchestrator-test {
            }
        "#
    });
}

#[derive(Default)]
struct MockTachyonHost {
    inference_calls: Vec<String>,
    kv: HashMap<String, Vec<u8>>,
    graph: HashMap<String, Vec<String>>,
    wasi: Option<WasiCtx>,
}

fn test_store() -> anyhow::Result<(
    Engine,
    Linker<MockTachyonHost>,
    Store<MockTachyonHost>,
    TempDir,
)> {
    let workspace = tempfile::tempdir()?;
    let engine = Engine::default();
    let linker = Linker::new(&engine);
    let store = Store::new(
        &engine,
        MockTachyonHost {
            graph: HashMap::from([(
                "complex prompt".to_string(),
                vec!["refactor-parser".to_string()],
            )]),
            wasi: Some(wasmtime_wasi::WasiCtxBuilder::new().build()),
            ..MockTachyonHost::default()
        },
    );
    Ok((engine, linker, store, workspace))
}

#[test_log::test]
fn test_path_traversal_blocked() -> anyhow::Result<()> {
    let (_engine, _linker, _store, workspace) = test_store()?;
    let guard = orchestrator::PathGuard::new(workspace.path());

    let blocked = guard.validate("../../etc/passwd");

    assert!(blocked.is_err());
    assert!(matches!(
        blocked,
        Err(orchestrator::ToolError::UnauthorizedAccess { .. })
    ));
    Ok(())
}

#[test_log::test]
fn test_successful_skill_escalation() -> anyhow::Result<()> {
    let (_engine, _linker, mut store, _workspace) = test_store()?;
    store
        .data_mut()
        .inference_calls
        .push("ResourceExhausted:tier1".to_string());

    assert!(orchestrator::error_needs_escalation(
        &store.data().inference_calls[0]
    ));
    assert_eq!(orchestrator::tiers_for_max(2)[1].model, "qwen-coder-27b");
    assert_eq!(store.data().graph["complex prompt"][0], "refactor-parser");
    assert!(store.data().wasi.is_some());
    Ok(())
}

#[test_log::test]
fn test_rabbit_hole_detection() -> anyhow::Result<()> {
    let (_engine, _linker, mut store, _workspace) = test_store()?;
    store
        .data_mut()
        .kv
        .insert("last_error".to_string(), b"command failed".to_vec());

    assert!(orchestrator::response_needs_escalation(
        "<rabbit_hole_detected>"
    ));
    assert_eq!(store.data().kv["last_error"], b"command failed".to_vec());
    Ok(())
}
