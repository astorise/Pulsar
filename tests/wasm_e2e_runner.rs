use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use wasi_preview1_component_adapter_provider::{
    WASI_SNAPSHOT_PREVIEW1_ADAPTER_NAME, WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER,
};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};
use wit_component::ComponentEncoder;

struct MockTachyonHost {
    wasi: WasiCtx,
    table: ResourceTable,
    workspace: TempDir,
}

impl WasiView for MockTachyonHost {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

fn workspace_root() -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

fn build_orchestrator_wasm() -> anyhow::Result<PathBuf> {
    let root = workspace_root()?;
    let wasm = root.join("target/wasm32-wasip1/debug/orchestrator.wasm");
    if wasm.exists() {
        return Ok(wasm);
    }

    let status = Command::new("cargo")
        .args(["build", "-p", "orchestrator", "--target", "wasm32-wasip1"])
        .current_dir(&root)
        .status()?;
    anyhow::ensure!(
        status.success(),
        "failed to build orchestrator wasm artifact"
    );
    anyhow::ensure!(wasm.exists(), "orchestrator wasm artifact was not created");
    Ok(wasm)
}

fn component_from_orchestrator_wasm(wasm: &Path, workspace: &TempDir) -> anyhow::Result<PathBuf> {
    let module = std::fs::read(wasm)?;
    let component = ComponentEncoder::default()
        .module(&module)?
        .adapter(
            WASI_SNAPSHOT_PREVIEW1_ADAPTER_NAME,
            WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER,
        )?
        .validate(true)
        .encode()?;
    let component_path = workspace.path().join("orchestrator.component.wasm");
    std::fs::write(&component_path, component)?;
    Ok(component_path)
}

fn test_store() -> anyhow::Result<(Engine, Linker<MockTachyonHost>, Store<MockTachyonHost>)> {
    let workspace = tempfile::tempdir()?;
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    let store = Store::new(
        &engine,
        MockTachyonHost {
            wasi: WasiCtxBuilder::new().build(),
            table: ResourceTable::new(),
            workspace,
        },
    );
    Ok((engine, linker, store))
}

#[test_log::test]
fn test_component_from_file_loads_orchestrator_artifact() -> anyhow::Result<()> {
    let (engine, _linker, store) = test_store()?;
    let wasm = build_orchestrator_wasm()?;
    let component_path = component_from_orchestrator_wasm(&wasm, &store.data().workspace)?;

    let component = Component::from_file(&engine, &component_path)?;

    assert!(component_path.exists());
    drop(component);
    Ok(())
}

#[test_log::test]
fn test_path_traversal_blocked_before_host_call() -> anyhow::Result<()> {
    let (_engine, _linker, store) = test_store()?;
    let guard = orchestrator::PathGuard::new(store.data().workspace.path());

    let blocked = guard.validate("../../etc/passwd");

    assert!(matches!(
        blocked,
        Err(orchestrator::ToolError::UnauthorizedAccess { .. })
    ));
    Ok(())
}

#[test_log::test]
fn test_command_exit_code_and_git_sandbox_are_explicit() -> anyhow::Result<()> {
    assert!(orchestrator::command_exit_code_succeeded(0));
    assert!(!orchestrator::command_exit_code_succeeded(2));

    let blocked = vec!["-c".to_string(), "core.sshCommand=bad".to_string()];
    assert!(!orchestrator::is_allowed_git_command(&blocked));

    let allowed = vec!["status".to_string(), "--short".to_string()];
    assert!(orchestrator::is_allowed_git_command(&allowed));
    Ok(())
}
