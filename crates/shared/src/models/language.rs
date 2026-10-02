use serde::{Deserialize, Serialize};
use wasm_memory::{ContainerVariantType, FunctionType, WasmFunctionCall};

use super::runner::RunnerError;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum Language {
    #[default]
    Cpp,
    Rust,
}

impl Language {
    pub fn route(self) -> &'static str {
        match self {
            Self::Cpp => "c++",
            Self::Rust => "rust",
        }
    }

    pub fn fuel_limit(self, fuel: Option<i64>) -> Option<i64> {
        match self {
            Self::Cpp => fuel,
            // Four times the C++ reference budget; containers copy guest memory.
            Self::Rust => fuel.map(|fuel| fuel.saturating_mul(4).clamp(100_000, 1 << 48)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RustSignature {
    name: String,
    arguments: Vec<String>,
    result: String,
}

fn unsupported() -> RunnerError {
    RunnerError::UnsupportedSignature {
        message: "Rust submissions require one function signature shared by every test.".into(),
    }
}

fn rust_type(ty: FunctionType) -> Result<String, RunnerError> {
    let inner = match ty {
        FunctionType::Int(_) => "i32",
        FunctionType::Long(_) => "i64",
        FunctionType::Float(_) => "f32",
        FunctionType::Double(_) => "f64",
        FunctionType::Char(_) => "char",
        FunctionType::Bool(_) => "bool",
        FunctionType::String(_) => "String",
    };
    let variant = match ty {
        FunctionType::Int(v)
        | FunctionType::Long(v)
        | FunctionType::Float(v)
        | FunctionType::Double(v)
        | FunctionType::Char(v)
        | FunctionType::Bool(v)
        | FunctionType::String(v) => v,
    };
    Ok(match variant {
        ContainerVariantType::Single => inner.to_string(),
        ContainerVariantType::List => format!("Vec<{inner}>"),
        ContainerVariantType::Grid | ContainerVariantType::Graph => {
            format!("Vec<Vec<{inner}>>")
        }
    })
}

fn rust_default(ty: &str) -> &'static str {
    match ty {
        "i32" | "i64" => "0",
        "f32" | "f64" => "0.0",
        "bool" => "false",
        "char" => "' '",
        "String" => "String::new()",
        _ => "Vec::new()",
    }
}

fn wasm_param_type(ty: &str) -> &'static str {
    match ty {
        "i64" => "i64",
        "f32" => "f32",
        "f64" => "f64",
        _ => "i32",
    }
}

fn decode_expr(ty: &str, raw: &str) -> String {
    match ty {
        "i32" | "i64" | "f32" | "f64" => raw.to_string(),
        "bool" => format!("{raw} != 0"),
        "char" => format!("char::from({raw} as u8)"),
        "String" => format!("__acm_decode_string({raw})"),
        "Vec<i32>" => format!("__acm_decode_i32_list({raw})"),
        "Vec<i64>" => format!("__acm_decode_i64_list({raw})"),
        "Vec<f32>" => format!("__acm_decode_f32_list({raw})"),
        "Vec<f64>" => format!("__acm_decode_f64_list({raw})"),
        "Vec<bool>" => format!("__acm_decode_bool_list({raw})"),
        "Vec<char>" => format!("__acm_decode_char_list({raw})"),
        "Vec<String>" => format!("__acm_decode_string_list({raw})"),
        "Vec<Vec<i32>>" => format!("__acm_decode_i32_grid({raw})"),
        "Vec<Vec<i64>>" => format!("__acm_decode_i64_grid({raw})"),
        "Vec<Vec<f32>>" => format!("__acm_decode_f32_grid({raw})"),
        "Vec<Vec<f64>>" => format!("__acm_decode_f64_grid({raw})"),
        "Vec<Vec<bool>>" => format!("__acm_decode_bool_grid({raw})"),
        "Vec<Vec<char>>" => format!("__acm_decode_char_grid({raw})"),
        "Vec<Vec<String>>" => format!("__acm_decode_string_grid({raw})"),
        _ => raw.to_string(),
    }
}

fn encode_expr(ty: &str, value: &str) -> String {
    match ty {
        "i32" | "i64" | "f32" | "f64" => value.to_string(),
        "bool" => format!("i32::from({value})"),
        "char" => format!("{value} as i32"),
        "String" => format!("__acm_encode_string({value})"),
        "Vec<i32>" => format!("__acm_encode_i32_list({value})"),
        "Vec<i64>" => format!("__acm_encode_i64_list({value})"),
        "Vec<f32>" => format!("__acm_encode_f32_list({value})"),
        "Vec<f64>" => format!("__acm_encode_f64_list({value})"),
        "Vec<bool>" => format!("__acm_encode_bool_list({value})"),
        "Vec<char>" => format!("__acm_encode_char_list({value})"),
        "Vec<String>" => format!("__acm_encode_string_list({value})"),
        "Vec<Vec<i32>>" => format!("__acm_encode_i32_grid({value})"),
        "Vec<Vec<i64>>" => format!("__acm_encode_i64_grid({value})"),
        "Vec<Vec<f32>>" => format!("__acm_encode_f32_grid({value})"),
        "Vec<Vec<f64>>" => format!("__acm_encode_f64_grid({value})"),
        "Vec<Vec<bool>>" => format!("__acm_encode_bool_grid({value})"),
        "Vec<Vec<char>>" => format!("__acm_encode_char_grid({value})"),
        "Vec<Vec<String>>" => format!("__acm_encode_string_grid({value})"),
        _ => value.to_string(),
    }
}

fn valid_ident(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !matches!(
            name,
            "_" | "self"
                | "Self"
                | "super"
                | "crate"
                | "as"
                | "async"
                | "await"
                | "break"
                | "const"
                | "continue"
                | "dyn"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "static"
                | "struct"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
        )
}

impl RustSignature {
    pub fn from_calls<'a>(
        calls: impl IntoIterator<Item = &'a WasmFunctionCall>,
    ) -> Result<Self, RunnerError> {
        let mut signature = None;
        for call in calls {
            if !valid_ident(&call.name) {
                return Err(unsupported());
            }
            let current = Self {
                name: call.name.clone(),
                arguments: call
                    .arguments
                    .iter()
                    .map(|argument| rust_type(argument.ty()))
                    .collect::<Result<_, _>>()?,
                result: rust_type(call.return_type)?,
            };
            if signature
                .as_ref()
                .is_some_and(|previous| previous != &current)
            {
                return Err(unsupported());
            }
            signature = Some(current);
        }
        signature.ok_or_else(unsupported)
    }

    pub fn parameter_names_from_cpp(template: &str) -> Vec<String> {
        let Some(body) = template.find('{') else {
            return Vec::new();
        };
        let header = &template[..body];
        let Some(open) = header.rfind('(') else {
            return Vec::new();
        };
        let Some(relative_close) = header[open + 1..].rfind(')') else {
            return Vec::new();
        };
        let inner = header[open + 1..open + 1 + relative_close].trim();
        if inner.is_empty() {
            return Vec::new();
        }
        split_cpp_params(inner)
            .into_iter()
            .filter_map(last_ident)
            .filter(|name| valid_ident(name))
            .collect()
    }

    fn parameters(&self, names: &[String]) -> String {
        self.arguments
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                let name = names
                    .get(i)
                    .cloned()
                    .filter(|name| valid_ident(name))
                    .unwrap_or_else(|| format!("arg{i}"));
                format!("{name}: {ty}")
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn template(&self) -> String {
        self.template_with_names(&[])
    }

    pub fn template_with_names(&self, names: &[String]) -> String {
        format!(
            "fn r#{}({}) -> {} {{\n    // Return the answer.\n    {}\n}}\n",
            self.name,
            self.parameters(names),
            self.result,
            rust_default(&self.result)
        )
    }

    fn wasm_parameters(&self) -> String {
        self.arguments
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("arg{i}: {}", wasm_param_type(ty)))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn entry_body(&self) -> String {
        let decoded = self
            .arguments
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                format!(
                    "        let __a{i} = {};\n",
                    decode_expr(ty, &format!("arg{i}"))
                )
            })
            .collect::<String>();
        let call_args = (0..self.arguments.len())
            .map(|i| format!("__a{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{decoded}        {}\n",
            encode_expr(
                &self.result,
                &format!("crate::r#{}({call_args})", self.name)
            )
        )
    }

    pub fn wrapper(&self) -> String {
        format!(
            "include!(\"implementation.rs\");\n{CODEC}\nconst _: () = {{\n    #[export_name = \"acm_alloc\"]\n    pub extern \"C\" fn __acm_alloc(n: i32) -> i32 {{ __acm_alloc_bytes(n) }}\n    #[export_name = \"acm_entry\"]\n    pub extern \"C\" fn __acm_entry({}) -> {} {{\n{}    }}\n}};\n",
            self.wasm_parameters(),
            wasm_param_type(&self.result),
            self.entry_body()
        )
    }
}

fn split_cpp_params(inner: &str) -> Vec<&str> {
    let mut params = Vec::new();
    let mut start = 0;
    let mut depth = 0i32;
    for (i, c) in inner.char_indices() {
        match c {
            '<' | '(' => depth += 1,
            '>' | ')' => depth -= 1,
            ',' if depth == 0 => {
                params.push(inner[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    params.push(inner[start..].trim());
    params
}

fn last_ident(param: &str) -> Option<String> {
    let token = param
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .rfind(|part| !part.is_empty())?;
    Some(token.to_string())
}

const CODEC: &str = r#"
#[allow(dead_code)]
fn __acm_alloc_bytes(n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    let mut v = vec![0u8; n as usize];
    let ptr = v.as_mut_ptr();
    std::mem::forget(v);
    ptr as i32
}
#[allow(dead_code)]
fn __acm_load_u32(p: *const u8) -> u32 {
    u32::from_le_bytes(unsafe { *(p as *const [u8; 4]) })
}
#[allow(dead_code)]
fn __acm_payload(ptr: i32) -> (*const u8, usize) {
    let p = ptr as *const u8;
    let n = __acm_load_u32(p) as usize;
    (unsafe { p.add(4) }, n)
}
#[allow(dead_code)]
fn __acm_pack(payload: &[u8]) -> i32 {
    let total = 4 + payload.len();
    let ptr = __acm_alloc_bytes(total as i32);
    let p = ptr as *mut u8;
    unsafe {
        std::ptr::copy_nonoverlapping((payload.len() as u32).to_le_bytes().as_ptr(), p, 4);
        std::ptr::copy_nonoverlapping(payload.as_ptr(), p.add(4), payload.len());
    }
    ptr
}
#[allow(dead_code)]
fn __acm_decode_i32_list(ptr: i32) -> Vec<i32> {
    __acm_decode_list(ptr, 4, |p| i32::from_le_bytes(unsafe { *(p as *const [u8; 4]) }))
}
#[allow(dead_code)]
fn __acm_decode_i64_list(ptr: i32) -> Vec<i64> {
    __acm_decode_list(ptr, 8, |p| i64::from_le_bytes(unsafe { *(p as *const [u8; 8]) }))
}
#[allow(dead_code)]
fn __acm_decode_f32_list(ptr: i32) -> Vec<f32> {
    __acm_decode_list(ptr, 4, |p| f32::from_le_bytes(unsafe { *(p as *const [u8; 4]) }))
}
#[allow(dead_code)]
fn __acm_decode_f64_list(ptr: i32) -> Vec<f64> {
    __acm_decode_list(ptr, 8, |p| f64::from_le_bytes(unsafe { *(p as *const [u8; 8]) }))
}
#[allow(dead_code)]
fn __acm_decode_bool_list(ptr: i32) -> Vec<bool> {
    __acm_decode_list(ptr, 1, |p| unsafe { *p != 0 })
}
#[allow(dead_code)]
fn __acm_decode_char_list(ptr: i32) -> Vec<char> {
    __acm_decode_list(ptr, 1, |p| unsafe { *p as char })
}
#[allow(dead_code)]
fn __acm_decode_string(ptr: i32) -> String {
    let (p, n) = __acm_payload(ptr);
    __acm_read_string(p, n).0
}
#[allow(dead_code)]
fn __acm_read_string(p: *const u8, available: usize) -> (String, usize) {
    let len = __acm_load_u32(p) as usize;
    let bytes = unsafe { std::slice::from_raw_parts(p.add(4), len) };
    (String::from_utf8_lossy(bytes).into_owned(), 4 + len)
}
#[allow(dead_code)]
fn __acm_decode_string_list(ptr: i32) -> Vec<String> {
    let (p, n) = __acm_payload(ptr);
    __acm_read_string_list(p, n).0
}
#[allow(dead_code)]
fn __acm_read_string_list(mut p: *const u8, _available: usize) -> (Vec<String>, usize) {
    let count = __acm_load_u32(p) as usize;
    p = unsafe { p.add(4) };
    let mut used = 4usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let (s, n) = __acm_read_string(p, 0);
        out.push(s);
        p = unsafe { p.add(n) };
        used += n;
    }
    (out, used)
}
#[allow(dead_code)]
fn __acm_decode_list<T>(ptr: i32, item: usize, read: impl Fn(*const u8) -> T) -> Vec<T> {
    let (p, _) = __acm_payload(ptr);
    let count = __acm_load_u32(p) as usize;
    let mut out = Vec::with_capacity(count);
    let mut q = unsafe { p.add(4) };
    for _ in 0..count {
        out.push(read(q));
        q = unsafe { q.add(item) };
    }
    out
}
#[allow(dead_code)]
fn __acm_decode_i32_grid(ptr: i32) -> Vec<Vec<i32>> {
    __acm_decode_grid(ptr, 4, |p| i32::from_le_bytes(unsafe { *(p as *const [u8; 4]) }))
}
#[allow(dead_code)]
fn __acm_decode_i64_grid(ptr: i32) -> Vec<Vec<i64>> {
    __acm_decode_grid(ptr, 8, |p| i64::from_le_bytes(unsafe { *(p as *const [u8; 8]) }))
}
#[allow(dead_code)]
fn __acm_decode_f32_grid(ptr: i32) -> Vec<Vec<f32>> {
    __acm_decode_grid(ptr, 4, |p| f32::from_le_bytes(unsafe { *(p as *const [u8; 4]) }))
}
#[allow(dead_code)]
fn __acm_decode_f64_grid(ptr: i32) -> Vec<Vec<f64>> {
    __acm_decode_grid(ptr, 8, |p| f64::from_le_bytes(unsafe { *(p as *const [u8; 8]) }))
}
#[allow(dead_code)]
fn __acm_decode_bool_grid(ptr: i32) -> Vec<Vec<bool>> {
    __acm_decode_grid(ptr, 1, |p| unsafe { *p != 0 })
}
#[allow(dead_code)]
fn __acm_decode_char_grid(ptr: i32) -> Vec<Vec<char>> {
    __acm_decode_grid(ptr, 1, |p| unsafe { *p as char })
}
#[allow(dead_code)]
fn __acm_decode_string_grid(ptr: i32) -> Vec<Vec<String>> {
    let (mut p, _) = __acm_payload(ptr);
    let rows = __acm_load_u32(p) as usize;
    p = unsafe { p.add(4) };
    let mut out = Vec::with_capacity(rows);
    for _ in 0..rows {
        let (row, n) = __acm_read_string_list(p, 0);
        out.push(row);
        p = unsafe { p.add(n) };
    }
    out
}
#[allow(dead_code)]
fn __acm_decode_grid<T>(ptr: i32, item: usize, read: impl Fn(*const u8) -> T) -> Vec<Vec<T>> {
    let (mut p, _) = __acm_payload(ptr);
    let rows = __acm_load_u32(p) as usize;
    p = unsafe { p.add(4) };
    let mut out = Vec::with_capacity(rows);
    for _ in 0..rows {
        let count = __acm_load_u32(p) as usize;
        p = unsafe { p.add(4) };
        let mut row = Vec::with_capacity(count);
        for _ in 0..count {
            row.push(read(p));
            p = unsafe { p.add(item) };
        }
        out.push(row);
    }
    out
}
#[allow(dead_code)]
fn __acm_encode_i32_list(v: Vec<i32>) -> i32 {
    let mut p = Vec::new();
    p.extend_from_slice(&(v.len() as u32).to_le_bytes());
    for x in v {
        p.extend_from_slice(&x.to_le_bytes());
    }
    __acm_pack(&p)
}
#[allow(dead_code)]
fn __acm_encode_i64_list(v: Vec<i64>) -> i32 {
    let mut p = Vec::new();
    p.extend_from_slice(&(v.len() as u32).to_le_bytes());
    for x in v {
        p.extend_from_slice(&x.to_le_bytes());
    }
    __acm_pack(&p)
}
#[allow(dead_code)]
fn __acm_encode_f32_list(v: Vec<f32>) -> i32 {
    let mut p = Vec::new();
    p.extend_from_slice(&(v.len() as u32).to_le_bytes());
    for x in v {
        p.extend_from_slice(&x.to_le_bytes());
    }
    __acm_pack(&p)
}
#[allow(dead_code)]
fn __acm_encode_f64_list(v: Vec<f64>) -> i32 {
    let mut p = Vec::new();
    p.extend_from_slice(&(v.len() as u32).to_le_bytes());
    for x in v {
        p.extend_from_slice(&x.to_le_bytes());
    }
    __acm_pack(&p)
}
#[allow(dead_code)]
fn __acm_encode_bool_list(v: Vec<bool>) -> i32 {
    let mut p = Vec::new();
    p.extend_from_slice(&(v.len() as u32).to_le_bytes());
    p.extend(v.into_iter().map(u8::from));
    __acm_pack(&p)
}
#[allow(dead_code)]
fn __acm_encode_char_list(v: Vec<char>) -> i32 {
    let mut p = Vec::new();
    p.extend_from_slice(&(v.len() as u32).to_le_bytes());
    p.extend(v.into_iter().map(|c| c as u8));
    __acm_pack(&p)
}
#[allow(dead_code)]
fn __acm_encode_string(s: String) -> i32 {
    let mut p = Vec::new();
    p.extend_from_slice(&(s.len() as u32).to_le_bytes());
    p.extend_from_slice(s.as_bytes());
    __acm_pack(&p)
}
#[allow(dead_code)]
fn __acm_encode_string_list(v: Vec<String>) -> i32 {
    __acm_pack(&__acm_string_list_bytes(&v))
}
#[allow(dead_code)]
fn __acm_string_list_bytes(v: &[String]) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&(v.len() as u32).to_le_bytes());
    for s in v {
        p.extend_from_slice(&(s.len() as u32).to_le_bytes());
        p.extend_from_slice(s.as_bytes());
    }
    p
}
#[allow(dead_code)]
fn __acm_encode_string_grid(v: Vec<Vec<String>>) -> i32 {
    let mut p = Vec::new();
    p.extend_from_slice(&(v.len() as u32).to_le_bytes());
    for row in v {
        p.extend(__acm_string_list_bytes(&row));
    }
    __acm_pack(&p)
}
#[allow(dead_code)]
fn __acm_encode_i32_grid(v: Vec<Vec<i32>>) -> i32 {
    __acm_encode_grid(v, |x, p| p.extend_from_slice(&x.to_le_bytes()))
}
#[allow(dead_code)]
fn __acm_encode_i64_grid(v: Vec<Vec<i64>>) -> i32 {
    __acm_encode_grid(v, |x, p| p.extend_from_slice(&x.to_le_bytes()))
}
#[allow(dead_code)]
fn __acm_encode_f32_grid(v: Vec<Vec<f32>>) -> i32 {
    __acm_encode_grid(v, |x, p| p.extend_from_slice(&x.to_le_bytes()))
}
#[allow(dead_code)]
fn __acm_encode_f64_grid(v: Vec<Vec<f64>>) -> i32 {
    __acm_encode_grid(v, |x, p| p.extend_from_slice(&x.to_le_bytes()))
}
#[allow(dead_code)]
fn __acm_encode_bool_grid(v: Vec<Vec<bool>>) -> i32 {
    __acm_encode_grid(v, |x, p| p.push(u8::from(x)))
}
#[allow(dead_code)]
fn __acm_encode_char_grid(v: Vec<Vec<char>>) -> i32 {
    __acm_encode_grid(v, |x, p| p.push(x as u8))
}
#[allow(dead_code)]
fn __acm_encode_grid<T>(v: Vec<Vec<T>>, mut write: impl FnMut(T, &mut Vec<u8>)) -> i32 {
    let mut p = Vec::new();
    p.extend_from_slice(&(v.len() as u32).to_le_bytes());
    for row in v {
        p.extend_from_slice(&(row.len() as u32).to_le_bytes());
        for x in row {
            write(x, &mut p);
        }
    }
    __acm_pack(&p)
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_memory::{ContainerVariant, FunctionValue};

    fn call() -> WasmFunctionCall {
        WasmFunctionCall::new(
            "add",
            vec![FunctionValue::Int(ContainerVariant::Single(2))],
            FunctionType::Long(ContainerVariantType::Single),
        )
    }

    #[test]
    fn rust_signatures_validate_every_call_and_identifier() {
        let a = call();
        let signature = RustSignature::from_calls([&a]).unwrap();
        assert!(signature.template().contains("fn r#add(arg0: i32) -> i64"));
        assert!(signature.wrapper().contains("export_name = \"acm_entry\""));
        assert!(signature.wrapper().contains("acm_alloc"));
        let mut b = call();
        b.arguments[0] = FunctionValue::Int(ContainerVariant::List(vec![1]));
        assert!(RustSignature::from_calls([&a, &b]).is_err());
        b = call();
        b.name = "other".into();
        assert!(RustSignature::from_calls([&a, &b]).is_err());
        b.name = "add); injected".into();
        assert!(RustSignature::from_calls([&b]).is_err());
        assert!(RustSignature::from_calls([]).is_err());
        b = call();
        b.return_type = FunctionType::Float(ContainerVariantType::Single);
        assert!(RustSignature::from_calls([&a, &b]).is_err());
    }

    #[test]
    fn rust_signatures_accept_lists_strings_and_grids() {
        let call = WasmFunctionCall::new(
            "max_profit",
            vec![FunctionValue::Int(ContainerVariant::List(vec![1, 2]))],
            FunctionType::Int(ContainerVariantType::Single),
        );
        let signature = RustSignature::from_calls([&call]).unwrap();
        assert!(signature
            .template_with_names(&["prices".into()])
            .contains("fn r#max_profit(prices: Vec<i32>) -> i32"));
        assert!(signature.wrapper().contains("__acm_decode_i32_list"));
        let boards = WasmFunctionCall::new(
            "solve_n_queens",
            vec![FunctionValue::Int(ContainerVariant::Single(4))],
            FunctionType::String(ContainerVariantType::Grid),
        );
        let signature = RustSignature::from_calls([&boards]).unwrap();
        assert!(signature.template().contains("-> Vec<Vec<String>>"));
        assert!(signature.template().contains("Vec::new()"));
        assert_eq!(
            RustSignature::parameter_names_from_cpp(
                "int coin_change(vector<int> coins, int amount) {\n    return 0;\n}\n"
            ),
            vec!["coins", "amount"]
        );
    }

    #[test]
    fn language_defaults_and_fuel_are_bounded() {
        assert_eq!(Language::default(), Language::Cpp);
        assert!(serde_json::from_str::<Language>("\"python\"").is_err());
        assert_eq!(Language::Cpp.fuel_limit(Some(6)), Some(6));
        assert_eq!(Language::Rust.fuel_limit(Some(6)), Some(100_000));
        assert_eq!(Language::Rust.fuel_limit(Some(i64::MAX)), Some(1 << 48));
        assert_eq!(Language::Rust.fuel_limit(None), None);
    }
}
