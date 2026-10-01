use anyhow::{bail, Context, Result};

use crate::{ContainerVariant, ContainerVariantType, FunctionType, FunctionValue, WasmMemory};

pub const MAX_BLOB: usize = 32 * 1024 * 1024;

pub fn is_pointer_type(ty: &FunctionType) -> bool {
    !matches!(
        ty,
        FunctionType::Int(ContainerVariantType::Single)
            | FunctionType::Long(ContainerVariantType::Single)
            | FunctionType::Float(ContainerVariantType::Single)
            | FunctionType::Double(ContainerVariantType::Single)
            | FunctionType::Char(ContainerVariantType::Single)
            | FunctionType::Bool(ContainerVariantType::Single)
    )
}

pub fn wrap_payload(payload: &[u8]) -> Result<Vec<u8>> {
    if payload.len() > MAX_BLOB {
        bail!("Rust argument encoding exceeded {MAX_BLOB} bytes");
    }
    let mut out = Vec::with_capacity(4 + payload.len());
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

pub fn encode_payload(value: &FunctionValue) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    encode_value(value, &mut out)?;
    if out.len() > MAX_BLOB {
        bail!("Rust argument encoding exceeded {MAX_BLOB} bytes");
    }
    Ok(out)
}

pub fn decode_payload(ty: &FunctionType, payload: &[u8]) -> Result<FunctionValue> {
    let mut cur = Cursor {
        data: payload,
        pos: 0,
    };
    let value = decode_value(ty, &mut cur)?;
    if cur.pos != payload.len() {
        bail!("Rust result encoding had trailing bytes");
    }
    Ok(value)
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl Cursor<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|end| *end <= self.data.len())
            .context("Rust encoding was truncated")?;
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
}

fn encode_value(value: &FunctionValue, out: &mut Vec<u8>) -> Result<()> {
    match value {
        FunctionValue::Int(v) => encode_container(v, out, encode_i32),
        FunctionValue::Long(v) => encode_container(v, out, encode_i64),
        FunctionValue::Float(v) => encode_container(v, out, encode_f32),
        FunctionValue::Double(v) => encode_container(v, out, encode_f64),
        FunctionValue::Char(v) => encode_container(v, out, encode_char),
        FunctionValue::Bool(v) => encode_container(v, out, encode_bool),
        FunctionValue::String(v) => encode_container(v, out, encode_string),
    }
}

fn encode_container<T: WasmMemory>(
    value: &ContainerVariant<T>,
    out: &mut Vec<u8>,
    encode_item: fn(&T, &mut Vec<u8>) -> Result<()>,
) -> Result<()> {
    match value {
        ContainerVariant::Single(item) => encode_item(item, out),
        ContainerVariant::List(items) => encode_list(items, out, encode_item),
        ContainerVariant::Grid(rows) | ContainerVariant::Graph(rows) => {
            put_u32(out, rows.len() as u32);
            for row in rows {
                encode_list(row, out, encode_item)?;
            }
            Ok(())
        }
    }
}

fn encode_list<T>(
    items: &[T],
    out: &mut Vec<u8>,
    encode_item: fn(&T, &mut Vec<u8>) -> Result<()>,
) -> Result<()> {
    put_u32(out, items.len() as u32);
    for item in items {
        encode_item(item, out)?;
    }
    Ok(())
}

fn decode_value(ty: &FunctionType, cur: &mut Cursor<'_>) -> Result<FunctionValue> {
    Ok(match ty {
        FunctionType::Int(variant) => {
            FunctionValue::Int(decode_container(*variant, cur, decode_i32)?)
        }
        FunctionType::Long(variant) => {
            FunctionValue::Long(decode_container(*variant, cur, decode_i64)?)
        }
        FunctionType::Float(variant) => {
            FunctionValue::Float(decode_container(*variant, cur, decode_f32)?)
        }
        FunctionType::Double(variant) => {
            FunctionValue::Double(decode_container(*variant, cur, decode_f64)?)
        }
        FunctionType::Char(variant) => {
            FunctionValue::Char(decode_container(*variant, cur, decode_char)?)
        }
        FunctionType::Bool(variant) => {
            FunctionValue::Bool(decode_container(*variant, cur, decode_bool)?)
        }
        FunctionType::String(variant) => {
            FunctionValue::String(decode_container(*variant, cur, decode_string)?)
        }
    })
}

fn decode_container<T: WasmMemory>(
    variant: ContainerVariantType,
    cur: &mut Cursor<'_>,
    decode_item: fn(&mut Cursor<'_>) -> Result<T>,
) -> Result<ContainerVariant<T>> {
    Ok(match variant {
        ContainerVariantType::Single => ContainerVariant::Single(decode_item(cur)?),
        ContainerVariantType::List => ContainerVariant::List(decode_list(cur, decode_item)?),
        ContainerVariantType::Grid => ContainerVariant::Grid(decode_grid(cur, decode_item)?),
        ContainerVariantType::Graph => ContainerVariant::Graph(decode_grid(cur, decode_item)?),
    })
}

fn decode_grid<T>(
    cur: &mut Cursor<'_>,
    decode_item: fn(&mut Cursor<'_>) -> Result<T>,
) -> Result<Vec<Vec<T>>> {
    let rows = cur.u32()? as usize;
    let mut grid = Vec::with_capacity(rows);
    for _ in 0..rows {
        grid.push(decode_list(cur, decode_item)?);
    }
    Ok(grid)
}

fn decode_list<T>(
    cur: &mut Cursor<'_>,
    decode_item: fn(&mut Cursor<'_>) -> Result<T>,
) -> Result<Vec<T>> {
    let count = cur.u32()? as usize;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        items.push(decode_item(cur)?);
    }
    Ok(items)
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn encode_i32(value: &i32, out: &mut Vec<u8>) -> Result<()> {
    out.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn encode_i64(value: &i64, out: &mut Vec<u8>) -> Result<()> {
    out.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn encode_f32(value: &f32, out: &mut Vec<u8>) -> Result<()> {
    out.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn encode_f64(value: &f64, out: &mut Vec<u8>) -> Result<()> {
    out.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn encode_char(value: &char, out: &mut Vec<u8>) -> Result<()> {
    out.push(*value as u8);
    Ok(())
}

fn encode_bool(value: &bool, out: &mut Vec<u8>) -> Result<()> {
    out.push(u8::from(*value));
    Ok(())
}

fn encode_string(value: &String, out: &mut Vec<u8>) -> Result<()> {
    put_u32(out, value.len() as u32);
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn decode_i32(cur: &mut Cursor<'_>) -> Result<i32> {
    Ok(i32::from_le_bytes(cur.take(4)?.try_into().unwrap()))
}

fn decode_i64(cur: &mut Cursor<'_>) -> Result<i64> {
    Ok(i64::from_le_bytes(cur.take(8)?.try_into().unwrap()))
}

fn decode_f32(cur: &mut Cursor<'_>) -> Result<f32> {
    Ok(f32::from_le_bytes(cur.take(4)?.try_into().unwrap()))
}

fn decode_f64(cur: &mut Cursor<'_>) -> Result<f64> {
    Ok(f64::from_le_bytes(cur.take(8)?.try_into().unwrap()))
}

fn decode_char(cur: &mut Cursor<'_>) -> Result<char> {
    Ok(cur.take(1)?[0] as char)
}

fn decode_bool(cur: &mut Cursor<'_>) -> Result<bool> {
    Ok(cur.take(1)?[0] != 0)
}

fn decode_string(cur: &mut Cursor<'_>) -> Result<String> {
    let len = cur.u32()? as usize;
    Ok(String::from_utf8_lossy(cur.take(len)?).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(value: FunctionValue, ty: FunctionType) {
        let payload = encode_payload(&value).unwrap();
        assert_eq!(decode_payload(&ty, &payload).unwrap(), value);
        let blob = wrap_payload(&payload).unwrap();
        assert_eq!(&blob[4..], payload);
    }

    #[test]
    fn rust_abi_roundtrips_containers_and_strings() {
        roundtrip(
            FunctionValue::Int(ContainerVariant::List(vec![1, -2, 3])),
            FunctionType::Int(ContainerVariantType::List),
        );
        roundtrip(
            FunctionValue::Int(ContainerVariant::Grid(vec![vec![1, 2], vec![]])),
            FunctionType::Int(ContainerVariantType::Grid),
        );
        roundtrip(
            FunctionValue::String(ContainerVariant::Single("MCMXCIV".into())),
            FunctionType::String(ContainerVariantType::Single),
        );
        roundtrip(
            FunctionValue::String(ContainerVariant::Grid(vec![
                vec![".Q..".into(), "..Q.".into()],
                vec![],
            ])),
            FunctionType::String(ContainerVariantType::Grid),
        );
        roundtrip(
            FunctionValue::Int(ContainerVariant::List(vec![])),
            FunctionType::Int(ContainerVariantType::List),
        );
        assert!(is_pointer_type(&FunctionType::String(
            ContainerVariantType::Single
        )));
        assert!(!is_pointer_type(&FunctionType::Int(
            ContainerVariantType::Single
        )));
    }
}
