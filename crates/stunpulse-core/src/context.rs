use core::str;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use tokio::sync::RwLock;
use wasmtime::{component::ResourceTable, Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{
    pipe::{MemoryInputPipe, MemoryOutputPipe},
    preview1::{self, WasiP1Ctx},
    WasiCtx, WasiCtxBuilder, WasiView,
};

use crate::{error::Error, task::Task};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ModuleIdentifier {
    pub version: String,
    pub name: String,
}

#[derive(Clone)]
pub struct Context {
    engine: Engine,
    modules: Arc<RwLock<HashMap<ModuleIdentifier, Module>>>,
}

struct State {
    pub context: WasiP1Ctx,
}

impl WasiView for State {
    fn ctx(&mut self) -> &mut WasiCtx {
        self.context.ctx()
    }
    fn table(&mut self) -> &mut ResourceTable {
        self.context.table()
    }
}

impl Context {
    pub fn new() -> Result<Self, Error> {
        let mut config = Config::new();
        config.async_support(true);
        let engine = Engine::new(&config)?;

        Ok(Self {
            engine,
            modules: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn load_module(
        &self,
        identifier: ModuleIdentifier,
        file: PathBuf,
    ) -> Result<(), Error> {
        let module = Module::from_file(&self.engine, file)?;
        let mut modules = self.modules.write().await;

        modules.insert(identifier, module);

        Ok(())
    }

    pub async fn run_task(&self, task: Task) -> Result<(), Error> {
        let modules = self.modules.read().await;
        let identifier = ModuleIdentifier {
            version: "1".into(),
            name: "main".into(),
        };

        if let Some(module) = modules.get(&identifier) {
            let mut linker: Linker<State> = Linker::new(&self.engine);
            let stdin = MemoryInputPipe::new(serde_json::to_string(&task)?);
            let stdout = MemoryOutputPipe::new(1024 * 1024 * 16); // 16MB
            let stderr = MemoryOutputPipe::new(1024 * 1024 * 16); // 16MB

            let mut store = Store::new(
                &self.engine,
                State {
                    context: WasiCtxBuilder::new()
                        .stdin(stdin.clone())
                        .stdout(stdout.clone())
                        .stderr(stderr.clone())
                        .build_p1(),
                },
            );

            preview1::add_to_linker_async(&mut linker, |state| &mut state.context)?;
            linker.module_async(&mut store, "", &module).await?;

            let instance = linker.instantiate_async(&mut store, &module).await?;
            let function = instance.get_typed_func::<(), ()>(&mut store, &task.name)?;

            function.call_async(&mut store, ()).await?;

            let bytes = stdout.contents();
            let output = str::from_utf8(&bytes)?;

            tracing::info!("Output: {}", output);
        } else {
            tracing::warn!("Module with identifier not found: {:?}", identifier)
        }

        Ok(())
    }
}
