use wasm_memory::{ContainerVariantType, FunctionType, WasmFunctionCall};

use super::runner::RunnerError;

/// The Python view of a problem's function: `(element kind, list depth)` for
/// every argument and for the result.
#[derive(Debug, PartialEq, Eq)]
pub struct PythonSignature {
    name: String,
    arguments: Vec<PythonType>,
    result: PythonType,
}

/// An element kind understood by the runner harness and how deeply it is
/// nested in lists. `("int", 2)` is `list[list[int]]`.
pub type PythonType = (&'static str, usize);

fn unsupported() -> RunnerError {
    RunnerError::UnsupportedSignature {
        message: "Python submissions require one function signature shared by every test.".into(),
    }
}

pub fn python_type(ty: FunctionType) -> PythonType {
    let (kind, variant) = match ty {
        FunctionType::Int(v) | FunctionType::Long(v) => ("int", v),
        FunctionType::Float(v) | FunctionType::Double(v) => ("float", v),
        FunctionType::Char(v) => ("char", v),
        FunctionType::Bool(v) => ("bool", v),
        FunctionType::String(v) => ("str", v),
    };
    let depth = match variant {
        ContainerVariantType::Single => 0,
        ContainerVariantType::List => 1,
        ContainerVariantType::Grid | ContainerVariantType::Graph => 2,
    };
    (kind, depth)
}

fn annotation((kind, depth): PythonType) -> String {
    let mut name = if kind == "char" { "str" } else { kind }.to_string();
    for _ in 0..depth {
        name = format!("list[{name}]");
    }
    name
}

fn default_value((kind, depth): PythonType) -> &'static str {
    match (kind, depth) {
        (_, 1..) => "[]",
        ("int", _) => "0",
        ("float", _) => "0.0",
        ("bool", _) => "False",
        _ => "\"\"",
    }
}

fn valid_ident(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !matches!(
            name,
            "False"
                | "None"
                | "True"
                | "and"
                | "as"
                | "assert"
                | "async"
                | "await"
                | "break"
                | "class"
                | "continue"
                | "def"
                | "del"
                | "elif"
                | "else"
                | "except"
                | "finally"
                | "for"
                | "from"
                | "global"
                | "if"
                | "import"
                | "in"
                | "is"
                | "lambda"
                | "nonlocal"
                | "not"
                | "or"
                | "pass"
                | "raise"
                | "return"
                | "try"
                | "while"
                | "with"
                | "yield"
        )
}

impl PythonSignature {
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
                    .map(|argument| python_type(argument.ty()))
                    .collect(),
                result: python_type(call.return_type),
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> &[PythonType] {
        &self.arguments
    }

    pub fn result(&self) -> PythonType {
        self.result
    }

    pub fn template(&self) -> String {
        self.template_with_names(&[])
    }

    pub fn template_with_names(&self, names: &[String]) -> String {
        let parameters = self
            .arguments
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                let name = names
                    .get(i)
                    .filter(|name| valid_ident(name) && **name != self.name)
                    .cloned()
                    .unwrap_or_else(|| format!("arg{i}"));
                format!("{name}: {}", annotation(*ty))
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "def {}({parameters}) -> {}:\n    # Return the answer.\n    return {}\n",
            self.name,
            annotation(self.result),
            default_value(self.result)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_memory::{ContainerVariant, FunctionValue};

    fn call() -> WasmFunctionCall {
        WasmFunctionCall::new(
            "coin_change",
            vec![
                FunctionValue::Int(ContainerVariant::List(vec![1, 2])),
                FunctionValue::Long(ContainerVariant::Single(3)),
            ],
            FunctionType::Int(ContainerVariantType::Single),
        )
    }

    #[test]
    fn python_signatures_validate_every_call_and_identifier() {
        let a = call();
        let signature = PythonSignature::from_calls([&a]).unwrap();
        assert_eq!(signature.arguments(), [("int", 1), ("int", 0)]);
        assert_eq!(signature.result(), ("int", 0));
        let mut b = call();
        b.arguments.pop();
        assert!(PythonSignature::from_calls([&a, &b]).is_err());
        b = call();
        b.return_type = FunctionType::Bool(ContainerVariantType::Single);
        assert!(PythonSignature::from_calls([&a, &b]).is_err());
        for name in ["class", "add(): pass\nimport os #", "1st", ""] {
            b = call();
            b.name = name.into();
            assert!(PythonSignature::from_calls([&b]).is_err());
        }
        assert!(PythonSignature::from_calls([]).is_err());
    }

    #[test]
    fn python_templates_use_annotations_and_safe_parameter_names() {
        let signature = PythonSignature::from_calls([&call()]).unwrap();
        assert_eq!(
            signature.template_with_names(&["coins".into(), "lambda".into()]),
            "def coin_change(coins: list[int], arg1: int) -> int:\n    # Return the answer.\n    return 0\n"
        );
        let boards = WasmFunctionCall::new(
            "solve",
            vec![FunctionValue::Char(ContainerVariant::Single('a'))],
            FunctionType::String(ContainerVariantType::Grid),
        );
        assert_eq!(
            PythonSignature::from_calls([&boards]).unwrap().template(),
            "def solve(arg0: str) -> list[list[str]]:\n    # Return the answer.\n    return []\n"
        );
        let ratio = WasmFunctionCall::new(
            "ratio",
            vec![],
            FunctionType::Double(ContainerVariantType::Single),
        );
        assert!(PythonSignature::from_calls([&ratio])
            .unwrap()
            .template()
            .ends_with("-> float:\n    # Return the answer.\n    return 0.0\n"));
    }
}
