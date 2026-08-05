use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use spec_engine::{Engine, Value, DEFAULT_REGION};

#[derive(Debug, Deserialize)]
pub struct ParseDataItemInput {
    /// 协议类型，如 "dlt645-2007", "csg13", "csg16"
    pub protocol: String,
    /// DI码（数据标识），十六进制字符串，如 "00010000"
    pub di: String,
    /// 数据项内容，十六进制字符串，如 "01020304"
    pub hex_data: String,
    /// 可选的省份/区域，如 "GD", "GX"。不指定则使用默认通用定义
    #[serde(default)]
    pub region: Option<String>,
    /// 可选的方向（上行/下行）
    pub dir: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParseDataItemOutput {
    /// 解析成功
    pub success: bool,
    /// 解析结果（JSON格式）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
    /// 消耗的字节数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumed_bytes: Option<usize>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 解析电力协议数据项内容
///
/// 根据DI码定义，解析对应的数据项内容（hex_data）
///
/// # 输入参数
/// - engine: Engine 实例引用
/// - protocol: 协议类型（dlt645-2007, csg13, csg16等）
/// - di: DI码（数据标识，十六进制字符串）
/// - hex_data: 数据项内容（十六进制字符串）
/// - region: 可选的省份/区域代码
/// - dir: 可选的方向（上行/下行）
///
/// # 返回
/// 解析后的JSON结构
pub fn parse_data_item(engine: &Engine, input: ParseDataItemInput) -> Result<ParseDataItemOutput> {
    // 解析DI码
    let di_code = u32::from_str_radix(&input.di, 16)
        .context("DI码格式错误，应为十六进制字符串")?;

    // 解析hex数据
    let raw_data = hex_to_bytes(&input.hex_data)
        .context("数据项内容格式错误，应为十六进制字符串")?;

    // 确定region
    let region = input.region.as_deref().unwrap_or(DEFAULT_REGION);

    // 调用解析函数
    match engine.parse(&input.protocol, di_code, region, input.dir.as_deref(), &raw_data) {
        Ok((value, consumed)) => {
            let json_value = value_to_json(&value)?;
            Ok(ParseDataItemOutput {
                success: true,
                data: Some(json_value),
                consumed_bytes: Some(consumed),
                error: None,
            })
        }
        Err(e) => Ok(ParseDataItemOutput {
            success: false,
            data: None,
            consumed_bytes: None,
            error: Some(format!("{}", e)),
        }),
    }
}

/// 将十六进制字符串转换为字节数组
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    // 移除空格和常见分隔符
    let hex = hex.replace([' ', '-', ':', '_'], "");
    
    if hex.len() % 2 != 0 {
        anyhow::bail!("十六进制字符串长度必须是偶数");
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .context(format!("无效的十六进制字符: {}", &hex[i..i + 2]))
        })
        .collect()
}

/// 将 spec-engine 的 Value 转换为 serde_json::Value
fn value_to_json(value: &Value) -> Result<JsonValue> {
    match value {
        Value::Int(n) => Ok(JsonValue::Number((*n).into())),
        Value::Float(f) => {
            serde_json::Number::from_f64(*f)
                .map(JsonValue::Number)
                .ok_or_else(|| anyhow::anyhow!("无法转换浮点数: {}", f))
        }
        Value::Str(s) => Ok(JsonValue::String(s.clone())),
        Value::Bytes(b) => Ok(JsonValue::String(hex::encode(b))),
        Value::List(items) => {
            let json_items: Result<Vec<_>> = items.iter().map(value_to_json).collect();
            Ok(JsonValue::Array(json_items?))
        }
        Value::Map(map) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map.iter() {
                json_map.insert(k.clone(), value_to_json(v)?);
            }
            Ok(JsonValue::Object(json_map))
        }
        Value::Node { name, value, .. } => {
            let mut obj = serde_json::Map::new();
            obj.insert("name".to_string(), JsonValue::String(name.clone()));
            obj.insert("value".to_string(), value_to_json(value)?);
            Ok(JsonValue::Object(obj))
        }
        Value::Bit {
            bit_start,
            bit_end,
            bit_value,
            value,
            ..
        } => {
            let mut obj = serde_json::Map::new();
            obj.insert("bit_start".to_string(), JsonValue::Number((*bit_start).into()));
            obj.insert("bit_end".to_string(), JsonValue::Number((*bit_end).into()));
            obj.insert("bit_value".to_string(), JsonValue::Number((*bit_value).into()));
            if let Some(v) = value {
                obj.insert("value".to_string(), value_to_json(v)?);
            }
            Ok(JsonValue::Object(obj))
        }
        Value::Skip => Ok(JsonValue::String("(跳过)".to_string())),
        Value::WithUnit { value, unit } => {
            let mut obj = serde_json::Map::new();
            obj.insert("value".to_string(), value_to_json(value)?);
            obj.insert("unit".to_string(), JsonValue::String(unit.clone()));
            Ok(JsonValue::Object(obj))
        }
        Value::Invalid { reason } => {
            let mut obj = serde_json::Map::new();
            obj.insert("invalid".to_string(), JsonValue::Bool(true));
            obj.insert("reason".to_string(), JsonValue::String(reason.clone()));
            Ok(JsonValue::Object(obj))
        }
        Value::Pn(n) => {
            let mut obj = serde_json::Map::new();
            obj.insert("pn".to_string(), JsonValue::Number((*n).into()));
            Ok(JsonValue::Object(obj))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_bytes() {
        assert_eq!(hex_to_bytes("0102").unwrap(), vec![0x01, 0x02]);
        assert_eq!(hex_to_bytes("01 02").unwrap(), vec![0x01, 0x02]);
        assert_eq!(hex_to_bytes("01-02-03").unwrap(), vec![0x01, 0x02, 0x03]);
        assert!(hex_to_bytes("0").is_err()); // 奇数长度
        assert!(hex_to_bytes("GG").is_err()); // 无效字符
    }
}
