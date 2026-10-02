use anyhow::Result;
use serde::{Deserialize, Serialize};
use wasmtime::*;

use crate::{AllocatorFunc, WasmMemory};

fn restore_fuel_after_bonus(fuel_before: u64, fuel_after: u64, requested_bonus: u64) -> u64 {
    let granted_bonus = u64::MAX.saturating_sub(fuel_before).min(requested_bonus);
    let fuel_with_bonus = fuel_before.saturating_add(granted_bonus);
    let consumed = fuel_with_bonus.saturating_sub(fuel_after);
    fuel_after.saturating_sub(granted_bonus.saturating_sub(consumed))
}

#[derive(thiserror::Error, Debug)]
enum FunctionError {
    #[error("Expected a function with name \"{0}\", but it was not found.")]
    NamedFunction(String),
    #[error("Expected the required function export \"{0}\", but it was not found or had an incompatible type.")]
    RequiredFunction(String),
    #[error("Expected the required memory export \"{0}\", but it was not found.")]
    Memory(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ContainerVariantType {
    Graph,
    Grid,
    List,
    Single,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ContainerVariant<T: WasmMemory> {
    Graph(Vec<Vec<T>>),
    Grid(Vec<Vec<T>>),
    List(Vec<T>),
    Single(T),
}

impl<T> ContainerVariant<T>
where
    T: WasmMemory,
{
    fn into_memory<S>(
        self,
        store: &mut Store<S>,
        memory: &Memory,
        allocator: &AllocatorFunc,
    ) -> Result<usize> {
        match self {
            ContainerVariant::Graph(graph) => graph.into_memory(store, memory, allocator, None),
            ContainerVariant::Grid(grid) => grid.into_memory(store, memory, allocator, None),
            ContainerVariant::List(list) => list.into_memory(store, memory, allocator, None),
            ContainerVariant::Single(single) => single.into_memory(store, memory, allocator, None),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FunctionValue {
    String(ContainerVariant<String>),
    Int(ContainerVariant<i32>),
    Long(ContainerVariant<i64>),
    Float(ContainerVariant<f32>),
    Double(ContainerVariant<f64>),
    Char(ContainerVariant<char>),
    Bool(ContainerVariant<bool>),
}

impl FunctionValue {
    fn into_memory<S>(
        self,
        store: &mut Store<S>,
        memory: &Memory,
        allocator: &AllocatorFunc,
    ) -> Result<usize> {
        let next_offset = match self {
            FunctionValue::String(s) => s.into_memory(store, memory, allocator)?,
            FunctionValue::Int(i) => i.into_memory(store, memory, allocator)?,
            FunctionValue::Float(f) => f.into_memory(store, memory, allocator)?,
            FunctionValue::Char(c) => c.into_memory(store, memory, allocator)?,
            FunctionValue::Bool(b) => b.into_memory(store, memory, allocator)?,
            FunctionValue::Long(l) => l.into_memory(store, memory, allocator)?,
            FunctionValue::Double(d) => d.into_memory(store, memory, allocator)?,
        };

        Ok(next_offset)
    }

    pub fn ty(&self) -> FunctionType {
        match self {
            FunctionValue::String(ContainerVariant::Graph(_)) => {
                FunctionType::String(ContainerVariantType::Graph)
            }
            FunctionValue::String(ContainerVariant::Grid(_)) => {
                FunctionType::String(ContainerVariantType::Grid)
            }
            FunctionValue::String(ContainerVariant::List(_)) => {
                FunctionType::String(ContainerVariantType::List)
            }
            FunctionValue::String(ContainerVariant::Single(_)) => {
                FunctionType::String(ContainerVariantType::Single)
            }
            FunctionValue::Int(ContainerVariant::Graph(_)) => {
                FunctionType::Int(ContainerVariantType::Graph)
            }
            FunctionValue::Int(ContainerVariant::Grid(_)) => {
                FunctionType::Int(ContainerVariantType::Grid)
            }
            FunctionValue::Int(ContainerVariant::List(_)) => {
                FunctionType::Int(ContainerVariantType::List)
            }
            FunctionValue::Int(ContainerVariant::Single(_)) => {
                FunctionType::Int(ContainerVariantType::Single)
            }
            FunctionValue::Long(ContainerVariant::Graph(_)) => {
                FunctionType::Long(ContainerVariantType::Graph)
            }
            FunctionValue::Long(ContainerVariant::Grid(_)) => {
                FunctionType::Long(ContainerVariantType::Grid)
            }
            FunctionValue::Long(ContainerVariant::List(_)) => {
                FunctionType::Long(ContainerVariantType::List)
            }
            FunctionValue::Long(ContainerVariant::Single(_)) => {
                FunctionType::Long(ContainerVariantType::Single)
            }
            FunctionValue::Float(ContainerVariant::Graph(_)) => {
                FunctionType::Float(ContainerVariantType::Graph)
            }
            FunctionValue::Float(ContainerVariant::Grid(_)) => {
                FunctionType::Float(ContainerVariantType::Grid)
            }
            FunctionValue::Float(ContainerVariant::List(_)) => {
                FunctionType::Float(ContainerVariantType::List)
            }
            FunctionValue::Float(ContainerVariant::Single(_)) => {
                FunctionType::Float(ContainerVariantType::Single)
            }
            FunctionValue::Double(ContainerVariant::Graph(_)) => {
                FunctionType::Double(ContainerVariantType::Graph)
            }
            FunctionValue::Double(ContainerVariant::Grid(_)) => {
                FunctionType::Double(ContainerVariantType::Grid)
            }
            FunctionValue::Double(ContainerVariant::List(_)) => {
                FunctionType::Double(ContainerVariantType::List)
            }
            FunctionValue::Double(ContainerVariant::Single(_)) => {
                FunctionType::Double(ContainerVariantType::Single)
            }
            FunctionValue::Char(ContainerVariant::Graph(_)) => {
                FunctionType::Char(ContainerVariantType::Graph)
            }
            FunctionValue::Char(ContainerVariant::Grid(_)) => {
                FunctionType::Char(ContainerVariantType::Grid)
            }
            FunctionValue::Char(ContainerVariant::List(_)) => {
                FunctionType::Char(ContainerVariantType::List)
            }
            FunctionValue::Char(ContainerVariant::Single(_)) => {
                FunctionType::Char(ContainerVariantType::Single)
            }
            FunctionValue::Bool(ContainerVariant::Graph(_)) => {
                FunctionType::Bool(ContainerVariantType::Graph)
            }
            FunctionValue::Bool(ContainerVariant::Grid(_)) => {
                FunctionType::Bool(ContainerVariantType::Grid)
            }
            FunctionValue::Bool(ContainerVariant::List(_)) => {
                FunctionType::Bool(ContainerVariantType::List)
            }
            FunctionValue::Bool(ContainerVariant::Single(_)) => {
                FunctionType::Bool(ContainerVariantType::Single)
            }
        }
    }

    pub fn scaling_factor(&self) -> f32 {
        match self {
            FunctionValue::String(ContainerVariant::Single(s)) => s.len() as f32,
            FunctionValue::String(ContainerVariant::List(l)) => l.len() as f32,
            FunctionValue::String(ContainerVariant::Grid(g)) => (g[0].len() * g.len()) as f32,
            FunctionValue::String(ContainerVariant::Graph(g)) => g.len() as f32, // for now we just do number of nodes, should factor edges though

            FunctionValue::Int(ContainerVariant::Single(s)) => s.abs() as f32,
            FunctionValue::Int(ContainerVariant::List(l)) => l.len() as f32,
            FunctionValue::Int(ContainerVariant::Grid(g)) => (g[0].len() * g.len()) as f32,
            FunctionValue::Int(ContainerVariant::Graph(g)) => g.len() as f32,

            FunctionValue::Long(ContainerVariant::Single(s)) => s.abs() as f32,
            FunctionValue::Long(ContainerVariant::List(l)) => l.len() as f32,
            FunctionValue::Long(ContainerVariant::Grid(g)) => (g[0].len() * g.len()) as f32,
            FunctionValue::Long(ContainerVariant::Graph(g)) => g.len() as f32,

            FunctionValue::Float(ContainerVariant::Single(s)) => s.abs(),
            FunctionValue::Float(ContainerVariant::List(l)) => l.len() as f32,
            FunctionValue::Float(ContainerVariant::Grid(g)) => (g[0].len() * g.len()) as f32,
            FunctionValue::Float(ContainerVariant::Graph(g)) => g.len() as f32,

            FunctionValue::Double(ContainerVariant::Single(s)) => s.abs() as f32,
            FunctionValue::Double(ContainerVariant::List(l)) => l.len() as f32,
            FunctionValue::Double(ContainerVariant::Grid(g)) => (g[0].len() * g.len()) as f32,
            FunctionValue::Double(ContainerVariant::Graph(g)) => g.len() as f32,

            FunctionValue::Char(ContainerVariant::Single(_)) => 1.0,
            FunctionValue::Char(ContainerVariant::List(l)) => l.len() as f32,
            FunctionValue::Char(ContainerVariant::Grid(g)) => (g[0].len() * g.len()) as f32,
            FunctionValue::Char(ContainerVariant::Graph(g)) => g.len() as f32,

            FunctionValue::Bool(ContainerVariant::Single(_)) => 1.0,
            FunctionValue::Bool(ContainerVariant::List(l)) => l.len() as f32,
            FunctionValue::Bool(ContainerVariant::Grid(g)) => (g[0].len() * g.len()) as f32,
            FunctionValue::Bool(ContainerVariant::Graph(g)) => g.len() as f32,
        }
    }
}

impl PartialEq for FunctionValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare singleton doubles with less precision because there can be annoying
            // precision issues when solving problems otherwise.
            (
                FunctionValue::Double(ContainerVariant::Single(left)),
                FunctionValue::Double(ContainerVariant::Single(right)),
            ) => (*left - *right).abs() < 1e-9,

            (FunctionValue::String(left), FunctionValue::String(right)) => left == right,
            (FunctionValue::Int(left), FunctionValue::Int(right)) => left == right,
            (FunctionValue::Long(left), FunctionValue::Long(right)) => left == right,
            (FunctionValue::Char(left), FunctionValue::Char(right)) => left == right,
            (FunctionValue::Bool(left), FunctionValue::Bool(right)) => left == right,

            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum FunctionType {
    String(ContainerVariantType),
    Int(ContainerVariantType),
    Long(ContainerVariantType),
    Float(ContainerVariantType),
    Double(ContainerVariantType),
    Char(ContainerVariantType),
    Bool(ContainerVariantType),
}

impl FunctionType {
    // Deserializes memory according to this type descriptor.
    #[allow(clippy::wrong_self_convention)]
    fn from_memory<S>(
        &self,
        store: &mut Store<S>,
        memory: &Memory,
        offset: usize,
    ) -> Result<FunctionValue> {
        let res = match self {
            FunctionType::String(ContainerVariantType::Graph) => FunctionValue::String(
                ContainerVariant::Graph(Vec::<Vec<String>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::String(ContainerVariantType::Grid) => FunctionValue::String(
                ContainerVariant::Grid(Vec::<Vec<String>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::String(ContainerVariantType::List) => FunctionValue::String(
                ContainerVariant::List(Vec::<String>::from_memory(store, memory, offset)?),
            ),
            FunctionType::String(ContainerVariantType::Single) => FunctionValue::String(
                ContainerVariant::Single(String::from_memory(store, memory, offset)?),
            ),

            FunctionType::Int(ContainerVariantType::Graph) => FunctionValue::Int(
                ContainerVariant::Graph(Vec::<Vec<i32>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Int(ContainerVariantType::Grid) => FunctionValue::Int(
                ContainerVariant::Grid(Vec::<Vec<i32>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Int(ContainerVariantType::List) => FunctionValue::Int(
                ContainerVariant::List(Vec::<i32>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Int(ContainerVariantType::Single) => FunctionValue::Int(
                ContainerVariant::Single(i32::from_memory(store, memory, offset)?),
            ),

            FunctionType::Long(ContainerVariantType::Graph) => FunctionValue::Long(
                ContainerVariant::Graph(Vec::<Vec<i64>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Long(ContainerVariantType::Grid) => FunctionValue::Long(
                ContainerVariant::Grid(Vec::<Vec<i64>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Long(ContainerVariantType::List) => FunctionValue::Long(
                ContainerVariant::List(Vec::<i64>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Long(ContainerVariantType::Single) => FunctionValue::Long(
                ContainerVariant::Single(i64::from_memory(store, memory, offset)?),
            ),

            FunctionType::Float(ContainerVariantType::Graph) => FunctionValue::Float(
                ContainerVariant::Graph(Vec::<Vec<f32>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Float(ContainerVariantType::Grid) => FunctionValue::Float(
                ContainerVariant::Grid(Vec::<Vec<f32>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Float(ContainerVariantType::List) => FunctionValue::Float(
                ContainerVariant::List(Vec::<f32>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Float(ContainerVariantType::Single) => FunctionValue::Float(
                ContainerVariant::Single(f32::from_memory(store, memory, offset)?),
            ),

            FunctionType::Double(ContainerVariantType::Graph) => FunctionValue::Double(
                ContainerVariant::Graph(Vec::<Vec<f64>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Double(ContainerVariantType::Grid) => FunctionValue::Double(
                ContainerVariant::Grid(Vec::<Vec<f64>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Double(ContainerVariantType::List) => FunctionValue::Double(
                ContainerVariant::List(Vec::<f64>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Double(ContainerVariantType::Single) => FunctionValue::Double(
                ContainerVariant::Single(f64::from_memory(store, memory, offset)?),
            ),

            FunctionType::Char(ContainerVariantType::Graph) => FunctionValue::Char(
                ContainerVariant::Graph(Vec::<Vec<char>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Char(ContainerVariantType::Grid) => FunctionValue::Char(
                ContainerVariant::Grid(Vec::<Vec<char>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Char(ContainerVariantType::List) => FunctionValue::Char(
                ContainerVariant::List(Vec::<char>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Char(ContainerVariantType::Single) => FunctionValue::Char(
                ContainerVariant::Single(char::from_memory(store, memory, offset)?),
            ),

            FunctionType::Bool(ContainerVariantType::Graph) => FunctionValue::Bool(
                ContainerVariant::Graph(Vec::<Vec<bool>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Bool(ContainerVariantType::Grid) => FunctionValue::Bool(
                ContainerVariant::Grid(Vec::<Vec<bool>>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Bool(ContainerVariantType::List) => FunctionValue::Bool(
                ContainerVariant::List(Vec::<bool>::from_memory(store, memory, offset)?),
            ),
            FunctionType::Bool(ContainerVariantType::Single) => FunctionValue::Bool(
                ContainerVariant::Single(bool::from_memory(store, memory, offset)?),
            ),
        };

        Ok(res)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WasmFunctionCall {
    pub name: String,
    pub arguments: Vec<FunctionValue>,
    pub return_type: FunctionType,
}

impl WasmFunctionCall {
    const PAGE_OFFSET: usize = 4;

    pub fn new(name: &str, arguments: Vec<FunctionValue>, return_type: FunctionType) -> Self {
        WasmFunctionCall {
            name: name.into(),
            arguments,
            return_type,
        }
    }

    /// Calls the fixed Rust export without C++ allocation or container layouts.
    pub fn call_integers<S>(
        self,
        store: &mut Store<S>,
        instance: &Instance,
    ) -> Result<(FunctionValue, u64)> {
        let params = self
            .arguments
            .iter()
            .map(|arg| match arg {
                FunctionValue::Int(ContainerVariant::Single(value)) => Ok(Val::I32(*value)),
                FunctionValue::Long(ContainerVariant::Single(value)) => Ok(Val::I64(*value)),
                _ => anyhow::bail!("Rust supports only scalar i32 and i64 arguments"),
            })
            .collect::<Result<Vec<_>>>()?;
        let result = match self.return_type {
            FunctionType::Int(ContainerVariantType::Single) => Val::I32(0),
            FunctionType::Long(ContainerVariantType::Single) => Val::I64(0),
            _ => anyhow::bail!("Rust supports only scalar i32 and i64 results"),
        };
        // Rust cdylibs can have a reactor initializer. Count its work against the limit.
        let before = store.get_fuel()?;
        if instance.get_export(&mut *store, "_initialize").is_some() {
            instance
                .get_typed_func::<(), ()>(&mut *store, "_initialize")?
                .call(&mut *store, ())?;
        }
        let function = instance
            .get_func(&mut *store, "acm_entry")
            .ok_or_else(|| FunctionError::RequiredFunction("acm_entry".into()))?;
        let mut results = [result];
        function.call(&mut *store, &params, &mut results)?;
        let output = match results[0] {
            Val::I32(value) => FunctionValue::Int(ContainerVariant::Single(value)),
            Val::I64(value) => FunctionValue::Long(ContainerVariant::Single(value)),
            _ => anyhow::bail!("Rust export has an incompatible result type"),
        };
        Ok((output, before.saturating_sub(store.get_fuel()?)))
    }

    /// Calls the generated Rust export. Scalars use wasm parameters; containers
    /// use the compact encoding in [`crate::rust_abi`].
    pub fn call_rust<S>(
        self,
        store: &mut Store<S>,
        instance: &Instance,
    ) -> Result<(FunctionValue, u64)> {
        let pointer_args = self
            .arguments
            .iter()
            .any(|arg| crate::is_pointer_type(&arg.ty()));
        let pointer_result = crate::is_pointer_type(&self.return_type);
        if !pointer_args
            && !pointer_result
            && self.arguments.iter().all(|arg| {
                matches!(
                    arg,
                    FunctionValue::Int(ContainerVariant::Single(_))
                        | FunctionValue::Long(ContainerVariant::Single(_))
                )
            })
            && matches!(
                self.return_type,
                FunctionType::Int(ContainerVariantType::Single)
                    | FunctionType::Long(ContainerVariantType::Single)
            )
        {
            return self.call_integers(store, instance);
        }

        const ARG_ALLOC_FUEL_DEFAULT: u64 = 100_000_000_000;
        let fuel_before_arg_setup = store.get_fuel()?;
        let arg_setup_bonus = u64::MAX
            .saturating_sub(fuel_before_arg_setup)
            .min(ARG_ALLOC_FUEL_DEFAULT);
        store.set_fuel(fuel_before_arg_setup.saturating_add(arg_setup_bonus))?;

        if instance.get_export(&mut *store, "_initialize").is_some() {
            instance
                .get_typed_func::<(), ()>(&mut *store, "_initialize")?
                .call(&mut *store, ())?;
        }

        let mut params = Vec::with_capacity(self.arguments.len());
        if pointer_args || pointer_result {
            let allocator: AllocatorFunc = instance
                .get_typed_func(&mut *store, "acm_alloc")
                .map_err(|_| FunctionError::RequiredFunction("acm_alloc".into()))?;
            let memory = instance
                .get_memory(&mut *store, "memory")
                .ok_or_else(|| FunctionError::Memory("memory".into()))?;
            for arg in &self.arguments {
                params.push(rust_param(store, &memory, &allocator, arg)?);
            }
        } else {
            for arg in &self.arguments {
                params.push(rust_scalar_param(arg)?);
            }
        }

        let fuel_after_arg_setup = store.get_fuel()?;
        store.set_fuel(restore_fuel_after_bonus(
            fuel_before_arg_setup,
            fuel_after_arg_setup,
            arg_setup_bonus,
        ))?;
        let initial_fuel = store.get_fuel()?;

        let function = instance
            .get_func(&mut *store, "acm_entry")
            .ok_or_else(|| FunctionError::RequiredFunction("acm_entry".into()))?;
        let mut results = [rust_result_placeholder(self.return_type)];
        function.call(&mut *store, &params, &mut results)?;
        let remaining_fuel = store.get_fuel()?;
        let output = if pointer_result {
            let memory = instance
                .get_memory(&mut *store, "memory")
                .ok_or_else(|| FunctionError::Memory("memory".into()))?;
            decode_rust_result(store, &memory, results[0].unwrap_i32(), self.return_type)?
        } else {
            rust_scalar_result(self.return_type, &results[0])?
        };
        Ok((output, initial_fuel.saturating_sub(remaining_fuel)))
    }

    // Returns the return value of the function, along with the fuel consumed *purely* by the
    // invocation of that funcion, not the memory allocation of passing the arguments.
    pub fn call<S>(
        self,
        mut store: &mut Store<S>,
        instance: &Instance,
    ) -> Result<(FunctionValue, u64)> {
        let allocator: AllocatorFunc = instance
            .get_typed_func(&mut store, "malloc")
            .map_err(|_| FunctionError::RequiredFunction("malloc".to_string()))?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| FunctionError::Memory("memory".to_string()))?;

        memory.grow(&mut store, Self::PAGE_OFFSET as u64)?;

        let mut params = vec![];
        let mut results = vec![];

        // If the return value is not a simple singleton, we instead put the address for the
        // return value to be written to as the first argument
        match self.return_type {
            FunctionType::Int(ContainerVariantType::Single)
            | FunctionType::Char(ContainerVariantType::Single)
            | FunctionType::Bool(ContainerVariantType::Single) => {
                results.push(Val::I32(0));
            }

            FunctionType::Long(ContainerVariantType::Single) => {
                results.push(Val::I64(0));
            }

            FunctionType::Float(ContainerVariantType::Single) => {
                results.push(Val::F32(0));
            }

            FunctionType::Double(ContainerVariantType::Single) => {
                results.push(Val::F64(0));
            }

            _ => params.push(Val::I32(0_i32)),
        }

        // because we need to allocate for some args, it's possible to improperly run out of fuel
        const ARG_ALLOC_FUEL_DEFAULT: u64 = 100_000_000_000;
        let fuel_before_arg_setup = store.get_fuel()?;
        let arg_setup_bonus = u64::MAX
            .saturating_sub(fuel_before_arg_setup)
            .min(ARG_ALLOC_FUEL_DEFAULT);
        store.set_fuel(fuel_before_arg_setup.saturating_add(arg_setup_bonus))?;

        let init = instance.get_typed_func::<_, ()>(&mut store, "_initialize")?;
        init.call(&mut store, ())?;

        for arg in self.arguments {
            match arg {
                FunctionValue::Int(ContainerVariant::Single(i)) => params.push(Val::I32(i)),
                FunctionValue::Long(ContainerVariant::Single(l)) => params.push(Val::I64(l)),
                FunctionValue::Float(ContainerVariant::Single(f)) => {
                    params.push(Val::F32(f.to_bits()))
                }
                FunctionValue::Double(ContainerVariant::Single(d)) => {
                    params.push(Val::F64(d.to_bits()))
                }
                FunctionValue::Char(ContainerVariant::Single(c)) => params.push(Val::I32(c as i32)),
                FunctionValue::Bool(ContainerVariant::Single(b)) => params.push(Val::I32(b as i32)),

                _ => {
                    let address = arg.into_memory(store, &memory, &allocator)?;
                    params.push(Val::I32(address as i32));
                }
            }
        }

        let fuel_after_arg_setup = store.get_fuel()?;
        store.set_fuel(restore_fuel_after_bonus(
            fuel_before_arg_setup,
            fuel_after_arg_setup,
            arg_setup_bonus,
        ))?;
        let initial_fuel = store.get_fuel()?;

        let mut out_params = vec![];
        match self.return_type {
            FunctionType::Int(ContainerVariantType::Single) => out_params.push(ValType::I32),
            FunctionType::Long(ContainerVariantType::Single) => out_params.push(ValType::I64),
            FunctionType::Float(ContainerVariantType::Single) => out_params.push(ValType::F32),
            FunctionType::Double(ContainerVariantType::Single) => out_params.push(ValType::F64),
            FunctionType::Char(ContainerVariantType::Single) => out_params.push(ValType::I32),
            FunctionType::Bool(ContainerVariantType::Single) => out_params.push(ValType::I32),
            _ => {}
        }

        // find name, should match in mangled string
        let externs = instance
            .exports(&mut store)
            .filter(|e| e.name().contains(&self.name))
            .map(Export::into_extern)
            .collect::<Vec<_>>();

        let mut found_func = false;

        for ext in &externs {
            if let Extern::Func(func) = ext {
                let ty = func.ty(&mut *store);
                let expected_params = ty.params().collect::<Vec<_>>();
                let expected_results = ty.results().collect::<Vec<_>>();
                let actual_params = params
                    .iter()
                    .map(|param| param.ty(&mut *store))
                    .collect::<std::result::Result<Vec<_>, _>>()?;

                if val_types_match(&expected_params, &actual_params)
                    && val_types_match(&expected_results, &out_params)
                {
                    func.call(&mut store, &params, &mut results)?;
                    found_func = true;
                    break;
                }
            }
        }

        if !found_func {
            return Err(FunctionError::NamedFunction(self.name.clone()).into());
        }

        let remaining_fuel = store.get_fuel()?;

        // If the return type is a simple singleton, we can simply take the value directly from the
        // return value. Otherwise, we must read it from memory, with the address given by the
        // first parameter.
        let return_value = match self.return_type {
            FunctionType::Int(ContainerVariantType::Single) => {
                FunctionValue::Int(ContainerVariant::Single(results[0].unwrap_i32()))
            }

            FunctionType::Long(ContainerVariantType::Single) => {
                FunctionValue::Long(ContainerVariant::Single(results[0].unwrap_i64()))
            }

            FunctionType::Char(ContainerVariantType::Single) => FunctionValue::Char(
                ContainerVariant::Single(results[0].unwrap_i32() as u8 as char),
            ),

            FunctionType::Bool(ContainerVariantType::Single) => {
                FunctionValue::Bool(ContainerVariant::Single(results[0].unwrap_i32() != 0))
            }

            FunctionType::Float(ContainerVariantType::Single) => {
                FunctionValue::Float(ContainerVariant::Single(results[0].unwrap_f32()))
            }

            FunctionType::Double(ContainerVariantType::Single) => {
                FunctionValue::Double(ContainerVariant::Single(results[0].unwrap_f64()))
            }

            _ => self
                .return_type
                .from_memory(store, &memory, params[0].unwrap_i32() as usize)?,
        };

        Ok((return_value, initial_fuel.saturating_sub(remaining_fuel)))
    }
}

fn rust_scalar_param(value: &FunctionValue) -> Result<Val> {
    Ok(match value {
        FunctionValue::Int(ContainerVariant::Single(v)) => Val::I32(*v),
        FunctionValue::Long(ContainerVariant::Single(v)) => Val::I64(*v),
        FunctionValue::Float(ContainerVariant::Single(v)) => Val::F32(v.to_bits()),
        FunctionValue::Double(ContainerVariant::Single(v)) => Val::F64(v.to_bits()),
        FunctionValue::Char(ContainerVariant::Single(v)) => Val::I32(*v as i32),
        FunctionValue::Bool(ContainerVariant::Single(v)) => Val::I32(i32::from(*v)),
        _ => anyhow::bail!("expected a Rust scalar argument"),
    })
}

fn rust_param<S>(
    store: &mut Store<S>,
    memory: &Memory,
    allocator: &AllocatorFunc,
    value: &FunctionValue,
) -> Result<Val> {
    if crate::is_pointer_type(&value.ty()) {
        let blob = crate::wrap_payload(&crate::encode_payload(value)?)?;
        let address = allocator.call(&mut *store, blob.len() as i32)? as usize;
        memory.write(&mut *store, address, &blob)?;
        Ok(Val::I32(address as i32))
    } else {
        rust_scalar_param(value)
    }
}

fn rust_result_placeholder(ty: FunctionType) -> Val {
    match ty {
        FunctionType::Long(ContainerVariantType::Single) => Val::I64(0),
        FunctionType::Float(ContainerVariantType::Single) => Val::F32(0),
        FunctionType::Double(ContainerVariantType::Single) => Val::F64(0),
        _ => Val::I32(0),
    }
}

fn rust_scalar_result(ty: FunctionType, value: &Val) -> Result<FunctionValue> {
    Ok(match ty {
        FunctionType::Int(ContainerVariantType::Single) => {
            FunctionValue::Int(ContainerVariant::Single(value.unwrap_i32()))
        }
        FunctionType::Long(ContainerVariantType::Single) => {
            FunctionValue::Long(ContainerVariant::Single(value.unwrap_i64()))
        }
        FunctionType::Float(ContainerVariantType::Single) => {
            FunctionValue::Float(ContainerVariant::Single(value.unwrap_f32()))
        }
        FunctionType::Double(ContainerVariantType::Single) => {
            FunctionValue::Double(ContainerVariant::Single(value.unwrap_f64()))
        }
        FunctionType::Char(ContainerVariantType::Single) => {
            FunctionValue::Char(ContainerVariant::Single(value.unwrap_i32() as u8 as char))
        }
        FunctionType::Bool(ContainerVariantType::Single) => {
            FunctionValue::Bool(ContainerVariant::Single(value.unwrap_i32() != 0))
        }
        _ => anyhow::bail!("expected a Rust scalar result"),
    })
}

fn decode_rust_result<S>(
    store: &mut Store<S>,
    memory: &Memory,
    ptr: i32,
    ty: FunctionType,
) -> Result<FunctionValue> {
    if ptr == 0 {
        anyhow::bail!("Rust result pointer was null");
    }
    let mut len_buf = [0u8; 4];
    memory.read(&mut *store, ptr as usize, &mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    if len > crate::MAX_BLOB {
        anyhow::bail!("Rust result encoding exceeded {} bytes", crate::MAX_BLOB);
    }
    let mut payload = vec![0u8; len];
    memory.read(&mut *store, ptr as usize + 4, &mut payload)?;
    crate::decode_payload(&ty, &payload)
}

fn val_types_match(expected: &[ValType], actual: &[ValType]) -> bool {
    expected.len() == actual.len()
        && expected
            .iter()
            .zip(actual)
            .all(|(expected, actual)| val_type_matches(expected, actual))
}

fn val_type_matches(expected: &ValType, actual: &ValType) -> bool {
    matches!(
        (expected, actual),
        (ValType::I32, ValType::I32)
            | (ValType::I64, ValType::I64)
            | (ValType::F32, ValType::F32)
            | (ValType::F64, ValType::F64)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call() -> WasmFunctionCall {
        WasmFunctionCall::new(
            "answer",
            vec![],
            FunctionType::Int(ContainerVariantType::Single),
        )
    }

    #[test]
    fn integer_call_needs_no_cpp_exports_and_enforces_fuel() {
        let mut config = Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config).unwrap();
        for (body, success) in [
            ("local.get 0 i64.extend_i32_s", true),
            ("(loop br 0) i64.const 0", false),
        ] {
            let module = Module::new(
                &engine,
                format!("(module (func (export \"acm_entry\") (param i32) (result i64) {body}))"),
            )
            .unwrap();
            let mut store = Store::new(&engine, ());
            store.set_fuel(100).unwrap();
            let instance = Instance::new(&mut store, &module, &[]).unwrap();
            let input = WasmFunctionCall::new(
                "ignored",
                vec![FunctionValue::Int(ContainerVariant::Single(-7))],
                FunctionType::Long(ContainerVariantType::Single),
            );
            let result = input.call_integers(&mut store, &instance);
            if success {
                let (value, fuel) = result.unwrap();
                assert_eq!(value, FunctionValue::Long(ContainerVariant::Single(-7)));
                assert!(fuel > 0);
            } else {
                assert_eq!(
                    result.unwrap_err().downcast_ref::<Trap>(),
                    Some(&Trap::OutOfFuel)
                );
            }
        }
    }

    #[test]
    fn integer_call_rejects_container_signatures() {
        let engine = Engine::default();
        let module = Module::new(&engine, "(module)").unwrap();
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[]).unwrap();
        let input = WasmFunctionCall::new(
            "answer",
            vec![FunctionValue::Int(ContainerVariant::List(vec![1]))],
            FunctionType::Int(ContainerVariantType::Single),
        );
        assert!(input
            .call_integers(&mut store, &instance)
            .unwrap_err()
            .to_string()
            .contains("scalar"));
    }

    #[test]
    fn call_reports_missing_allocator_export() {
        let engine = Engine::default();
        let module = Module::new(&engine, "(module (memory (export \"memory\") 1))").unwrap();
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        let error = call().call(&mut store, &instance).unwrap_err();

        assert!(error
            .to_string()
            .contains("required function export \"malloc\""));
    }

    #[test]
    fn call_reports_missing_memory_export() {
        let engine = Engine::default();
        let module = Module::new(
            &engine,
            "(module (func (export \"malloc\") (param i32) (result i32) i32.const 0))",
        )
        .unwrap();
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[]).unwrap();

        let error = call().call(&mut store, &instance).unwrap_err();

        assert!(error
            .to_string()
            .contains("required memory export \"memory\""));
    }

    #[test]
    fn rust_container_call_decodes_a_list_sum() {
        let wat = r#"
            (module
              (memory (export "memory") 2)
              (global $heap (mut i32) (i32.const 16))
              (func (export "acm_alloc") (param i32) (result i32)
                (local $p i32)
                (local.set $p (global.get $heap))
                (global.set $heap (i32.add (global.get $heap) (local.get 0)))
                (local.get $p))
              (func (export "acm_entry") (param i32) (result i32)
                (local $count i32) (local $i i32) (local $sum i32) (local $p i32)
                (local.set $p (i32.add (local.get 0) (i32.const 4)))
                (local.set $count (i32.load (local.get $p)))
                (local.set $p (i32.add (local.get $p) (i32.const 4)))
                (loop $more
                  (if (i32.eq (local.get $i) (local.get $count))
                    (then (return (local.get $sum))))
                  (local.set $sum (i32.add (local.get $sum) (i32.load (local.get $p))))
                  (local.set $p (i32.add (local.get $p) (i32.const 4)))
                  (local.set $i (i32.add (local.get $i) (i32.const 1)))
                  (br $more))
                (local.get $sum)))
        "#;
        let mut config = Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config).unwrap();
        let module = Module::new(&engine, wat).unwrap();
        let mut store = Store::new(&engine, ());
        store.set_fuel(10_000).unwrap();
        let instance = Instance::new(&mut store, &module, &[]).unwrap();
        let input = WasmFunctionCall::new(
            "sum",
            vec![FunctionValue::Int(ContainerVariant::List(vec![1, 2, 3, 4]))],
            FunctionType::Int(ContainerVariantType::Single),
        );
        let (value, fuel) = input.call_rust(&mut store, &instance).unwrap();
        assert_eq!(value, FunctionValue::Int(ContainerVariant::Single(10)));
        assert!(fuel > 0);
    }

    #[test]
    fn restoring_bonus_preserves_fuel_at_overflow_boundary() {
        assert_eq!(
            restore_fuel_after_bonus(u64::MAX - 5, u64::MAX - 2, 100),
            u64::MAX - 5
        );
    }
}
