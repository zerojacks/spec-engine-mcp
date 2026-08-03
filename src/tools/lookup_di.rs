use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use spec_engine::{get_spec_catalog, DEFAULT_REGION};

#[derive(Debug, Deserialize)]
pub struct LookupDiInput {
    /// 协议类型，如 "dlt645-2007"
    pub protocol: String,
    /// DI码，十六进制字符串，如 "00010000"
    pub di: String,
    /// 可选的省份/区域
    #[serde(default)]
    pub region: Option<String>,
    /// 可选的方向（上行/下行）
    #[serde(default)]
    pub direction: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LookupDiOutput {
    /// 查找成功
    pub success: bool,
    /// DI信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub di_info: Option<DiInfo>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DiInfo {
    /// DI码（十六进制）
    pub di: String,
    /// DI名称
    pub name: String,
    /// 协议
    pub protocol: String,
    /// 区域
    pub region: String,
    /// 方向
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// 字段结构概要
    pub structure: JsonValue,
    /// Schema文档说明（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_docs: Option<SchemaDocs>,
}

#[derive(Debug, Serialize)]
pub struct SchemaDocs {
    /// 字段类型说明
    pub field_types: JsonValue,
    /// 编码类型说明
    pub encoding_types: JsonValue,
    /// 特殊概念说明
    pub special_concepts: JsonValue,
    /// YAML示例
    pub yaml_examples: JsonValue,
}

/// 查询DI定义信息
///
/// # 输入参数
/// - protocol: 协议类型
/// - di: DI码（十六进制字符串）
/// - region: 可选的省份/区域代码
/// - direction: 可选的方向（上行/下行）
///
/// # 返回
/// DI的定义信息（名称、结构等）
pub fn lookup_di(input: LookupDiInput) -> Result<LookupDiOutput> {
    let di_code = u32::from_str_radix(&input.di, 16)
        .context("DI码格式错误，应为十六进制字符串")?;

    let region = input.region.as_deref().unwrap_or(DEFAULT_REGION);
    let catalog = get_spec_catalog();

    // 尝试查找：优先精确匹配，然后回退到通用定义
    let key = (
        input.protocol.clone(),
        di_code,
        region.to_string(),
        input.direction.clone(),
    );

    let named_field = catalog.get(&key).or_else(|| {
        // 回退到DEFAULT_REGION
        catalog.get(&(
            input.protocol.clone(),
            di_code,
            DEFAULT_REGION.to_string(),
            input.direction.clone(),
        ))
    });

    match named_field {
        Some(field) => {
            // 直接使用 serde 序列化，无需手写转换函数
            let structure = serde_json::to_value(&field.spec)
                .context("序列化 FieldSpec 失败")?;
            let schema_docs = generate_schema_docs();
            
            Ok(LookupDiOutput {
                success: true,
                di_info: Some(DiInfo {
                    di: format!("{:08X}", di_code),
                    name: field.name.clone(),
                    protocol: input.protocol,
                    region: region.to_string(),
                    direction: input.direction,
                    structure,
                    schema_docs: Some(schema_docs),
                }),
                error: None,
            })
        }
        None => Ok(LookupDiOutput {
            success: false,
            di_info: None,
            error: Some(format!(
                "未找到DI: {} (protocol={}, region={})",
                input.di, input.protocol, region
            )),
        }),
    }
}

/// 生成Schema文档说明
fn generate_schema_docs() -> SchemaDocs {
    SchemaDocs {
        field_types: generate_field_types_docs(),
        encoding_types: generate_encoding_types_docs(),
        special_concepts: generate_special_concepts_docs(),
        yaml_examples: generate_yaml_examples_docs(),
    }
}

/// 生成字段类型文档
fn generate_field_types_docs() -> JsonValue {
    serde_json::json!({
        "fixed": {
            "description": "定长字段：固定字节数的基本数据类型",
            "usage": "绝大多数字段（BCD/BIN/ASCII/HEX/时间戳）都使用此类型",
            "parameters": {
                "length": "字节长度（整数）或 length_ref（引用其他字段）或 lengthrule（表达式）",
                "encoding": "编码方式：bcd/bin/ascii/hex/time/raw",
                "unit": "可选：单位，如 V、kWh、A 等",
                "decimals": "可选：小数位数（BCD编码）",
                "signed": "可选：是否有符号位（原码，非补码）",
                "endian": "可选：字节序（仅bin且length>1时有效），little或big",
                "enum_map": "可选：枚举映射，将数值映射为文本含义"
            },
            "example": "心跳周期: length=1, type=bcd, unit=分"
        },
        "container": {
            "description": "容器：包含多个子字段的复合结构",
            "usage": "将多个字段组合成一个逻辑单元",
            "parameters": {
                "fields": "子字段列表，按顺序解析"
            },
            "notes": "子字段可以带id，此时可独立寻址；不带id则仅作为容器内部字段"
        },
        "repeat": {
            "description": "重复结构：按计数或位图驱动的重复字段序列",
            "usage": "费率数据块、APP列表等重复出现的结构",
            "parameters": {
                "count": "固定重复次数（编译期已知）",
                "count_ref": "引用其他字段作为重复次数（运行时动态）",
                "count_expr": "表达式计算重复次数，如 '$remaining / 4'",
                "bits_ref": "引用bitfield字段，按置位情况驱动重复",
                "element": "重复的元素结构",
                "name_template": "元素命名模板，支持{index}、{index0}、{bit_name}等占位符",
                "id_expr": "可选：DI号计算表达式，如 '0x00010100 + index0*0x0100'"
            },
            "notes": "count/count_ref/count_expr三选一；总长度=count×单个元素长度"
        },
        "switch": {
            "description": "条件分支：根据某个字段的值选择不同的解析分支",
            "usage": "字段类型/含义依赖同容器内另一字段的值",
            "parameters": {
                "on": "引用字段的id或name，或特殊值 $remaining/$len/$length",
                "cases": "分支映射：值->字段规格",
                "case_names": "可选：case名称映射",
                "default": "可选：默认分支"
            },
            "example": "通信地址依赖通信通道类型：GPRS用IP+端口，其他用电话号码BCD"
        },
        "bitfield": {
            "description": "位域：将字节按位切分，每个位或位段有独立的含义",
            "usage": "运行状态字、配置标志位等",
            "parameters": {
                "length": "字节长度",
                "bits": "位定义列表，每项包含 range、name、可选enum_map和ref_id"
            },
            "notes": "bit编号约定需对照协议文档确认（LSB或MSB）"
        },
        "bitmask": {
            "description": "位掩码：按位图决定后续字段是否存在的可变结构",
            "usage": "失败表、告警项目位图、状态掩码",
            "parameters": {
                "length": "掩码字节长度",
                "bit_direction": "位遍历方向：lsb或msb",
                "iterate_order": "迭代顺序：asc或desc",
                "bit_specs": "位定义列表",
                "element": "每位对应的后续结构",
                "name_template": "元素命名模板"
            },
            "notes": "位为0时跳过不占字节；位为1时按element解析后续数据"
        },
        "external": {
            "description": "外部协议：内嵌其他协议的完整报文",
            "usage": "中继转发内嵌DL/T645报文等",
            "parameters": {
                "protocol": "外部协议名称，如 dlt645-2007",
                "length": "报文长度：remaining、固定值、length_ref或lengthrule"
            },
            "notes": "调用外部协议解析器，复用单位是整个协议"
        },
        "dict_ref": {
            "description": "字典引用：运行时根据字段值查找对应的DI定义",
            "usage": "主动上报报文中'数据标识+对应采集值'列表",
            "parameters": {
                "di_ref": "引用字段的ref_id（必须提前定义ref_id）"
            },
            "notes": "与di_sequence的区别：dict_ref引用目标运行时才知道，di_sequence编译期已知"
        },
        "skip": {
            "description": "跳过：消耗字节但不生成解析节点",
            "usage": "保留字段、填充字节、空分支",
            "parameters": {
                "length": "跳过的字节数，省略时默认为0"
            },
            "notes": "常用于switch/bitmask的空分支：{ length: 0, type: skip }"
        },
        "info_point": {
            "description": "信息点标识（DA）：固定2字节，测量点组号+位掩码",
            "usage": "选择测量点（6.1.3 信息点标识DA）",
            "structure": {
                "DA1": "位掩码（第1字节），D0-D7对应该组内8个测量点",
                "DA2": "测量点组号（第2字节），1-254"
            },
            "special_values": {
                "00 00": "终端测量点p0",
                "FF FF": "除终端测量点外的所有测量点"
            },
            "notes": "一个info_point可能命中多个测量点，结果为List"
        },
        "di_code": {
            "description": "数据标识编码（DI）：固定4字节，仅标识不解析内容",
            "usage": "任务定义中枚举包含哪些DI（6.1.4 数据标识编码DI）",
            "structure": "DI0 DI1 DI2 DI3（小端序），按小端读出后查字典获取名称",
            "output": "格式为 '{DI码8位十六进制}_{字典名称}'，未知DI输出 '{DI码}_未知数据标识'",
            "notes": "只解析标识，不解析该DI的数据内容；与dict_ref的区别见文档7.5节"
        },
        "custom": {
            "description": "自定义处理：特殊的解析逻辑，无法用标准规则描述",
            "usage": "语义无法从表格推断的极少数字段",
            "parameters": {
                "handler": "Rust函数名，如 parse_xxx_field"
            },
            "notes": "应尽量避免使用，使用频率应控制在个位数百分比"
        }
    })
}

/// 生成编码类型文档
fn generate_encoding_types_docs() -> JsonValue {
    serde_json::json!({
        "bcd": {
            "name": "BCD编码（Binary-Coded Decimal）",
            "description": "每个字节表示两位十进制数，一个字节的高4位和低4位各表示一位十进制数（0-9）",
            "parameters": {
                "decimals": {
                    "description": "小数位数",
                    "example": "decimals=2 表示 XXXXXX.XX 格式，如 123456 -> 1234.56"
                },
                "signed": {
                    "description": "是否有符号（原码，非补码）",
                    "detail": "符号位固定在首字节最高位；清除符号位后剩下的是数值绝对值",
                    "example": "0x8005 表示 -5（符号位=1，数值=0005）"
                },
                "endian": {
                    "description": "字节序（BCD默认不受影响）",
                    "note": "通常BCD不需要指定endian，按十进制位从左到右解析"
                }
            },
            "example": "字节 0x12 0x34 -> BCD解码 -> 1234"
        },
        "bin": {
            "name": "二进制编码",
            "description": "字节直接表示数值，需要考虑字节序",
            "parameters": {
                "endian": {
                    "description": "字节序（length>1时必须指定）",
                    "Little": "小端序（低字节在前）：链路层字段、报文长度等",
                    "Big": "大端序（高字节在前）：网络字节序，如IP端口号",
                    "example": "0x23 0x29 -> Little=0x2923=10531, Big=0x2329=9001"
                },
                "signed": {
                    "description": "是否有符号（原码，非补码）",
                    "detail": "符号位在最高有效字节的最高位；小端时在最后字节，大端时在首字节",
                    "example": "小端2字节 0x05 0x80 表示 -5（符号位在0x80，数值=0x0005）"
                }
            },
            "notes": "不写endian时默认little，但大端字段必须显式标注 endian: big"
        },
        "ascii": {
            "name": "ASCII字符串",
            "description": "字节按ASCII编码解释为可读文本",
            "example": "0x41 0x42 0x43 -> 'ABC'",
            "usage": "APP名称、版本号、厂家代码等文本字段"
        },
        "hex": {
            "name": "十六进制字符串",
            "description": "原始字节转为十六进制表示，不做数值计算或字节序转换",
            "difference": {
                "vs_bin": "bin表示数值（需要endian），hex表示标识符/代码（不需要endian）",
                "vs_ascii": "ascii解码为文本(0x41->'A')，hex输出十六进制(0x41->'41')"
            },
            "example": "0xFF 0x01 -> 'FF01'（按传输顺序拼接）",
            "usage": "MAC地址、序列号、密钥指纹、密码等级标识等"
        },
        "time": {
            "name": "时间编码",
            "description": "按特定格式解析时间字段，需配合bcd或bin类型使用",
            "parameters": {
                "format": {
                    "description": "时间格式字符串",
                    "examples": {
                        "ssmmhhDDMMYY": "秒分时日月年（6字节）",
                        "mmhhDDMMYY": "分时日月年（5字节）",
                        "DDHHMM": "日时分（3字节）"
                    }
                },
                "encoding": {
                    "description": "编码方式",
                    "Bcd": "每字节BCD编码",
                    "Bin": "二进制编码，需配合endian"
                }
            },
            "example": "type=bcd, time='mmhhDDMMYY' -> BCD编码的5字节时间"
        },
        "raw": {
            "name": "原始字节",
            "description": "不做任何解码，直接返回字节数组",
            "usage": "密钥状态、加密数据等不需要解释的原始二进制数据",
            "output": "字节数组的十六进制表示"
        }
    })
}

/// 生成特殊概念文档
fn generate_special_concepts_docs() -> JsonValue {
    serde_json::json!({
        "region": {
            "description": "区域/省份标识，支持按省份覆盖DI定义",
            "syntax": "YAML列表，每个元素一个省份，如 ['南网', '广东', '海南']",
            "error": "不要写成 ['南网,广东,海南']，这会被当成一个省份名",
            "note": "顶层省略时落在DEFAULT_REGION；嵌套字段省略时继承父级region"
        },
        "ref_id": {
            "description": "字段引用标识，供其他字段引用",
            "usage": "count_ref、length_ref、bits_ref、switch.on、dict_ref等都通过ref_id引用",
            "syntax": "在字段定义中添加 ref_id: xxx，引用时直接使用 xxx（不要加$前缀）",
            "example": "定义：ref_id: rate_count，引用：count_ref: rate_count"
        },
        "length_variants": {
            "fixed": "固定长度：length: 4",
            "ref": "引用其他字段：length_ref: frame_length（需提前定义ref_id）",
            "expr": "表达式计算：lengthrule: 'ref(char_count) * 2'",
            "remaining": "剩余全部：length: remaining"
        },
        "signed_encoding": {
            "description": "符号编码采用原码（sign-magnitude），非补码",
            "detail": "符号位单独表示正负，剩余位表示数值绝对值",
            "bcd": "符号位固定在首字节最高位",
            "bin": "符号位在最高有效字节的最高位（位置由endian决定）",
            "example": "BCD 0x80 0x05 -> -5, BIN小端 0x05 0x80 -> -5"
        },
        "$_prefix": {
            "description": "运行时合成变量前缀",
            "switch_on": {
                "$remaining": "当前容器剩余字节数",
                "$len": "同$remaining",
                "$length": "同$remaining",
                "$bit_value": "bitmask迭代时当前位的值"
            },
            "count_expr": {
                "$remaining": "剩余字节数，常用于 '$remaining / 4'"
            },
            "note": "引用普通字段时不要加$前缀，直接使用ref_id"
        },
        "enum_map": {
            "description": "枚举映射：将数值映射为文本含义",
            "key_format": "字符串形式的十六进制或二进制，如 '00', '01', 'FFFF'",
            "example": "enum_map: { '00': 无效, '01': 有效 }",
            "usage": "有效性标志、周期单位、运行状态等有固定含义的数值字段"
        }
    })
}

/// 生成YAML示例文档
fn generate_yaml_examples_docs() -> JsonValue {
    serde_json::json!({
        "simple_fixed": {
            "description": "简单定长字段",
            "yaml": "- name: 心跳周期\n  length: 1\n  type: bcd\n  unit: 分"
        },
        "time_field": {
            "description": "时间字段",
            "yaml": "- name: 上报基准时间\n  length: 5\n  type: bcd\n  time: mmhhDDMMYY"
        },
        "repeat_with_count_ref": {
            "description": "带计数引用的重复结构",
            "yaml": "- name: APP数量\n  ref_id: app_count\n  length: 1\n  type: bin\n\n- name: APP信息\n  type: repeat\n  count_ref: app_count\n  element:\n    length: 30\n    type: container\n    fields: [...]"
        },
        "switch_on_field": {
            "description": "基于字段值的条件分支",
            "yaml": "- name: 通信通道类型\n  ref_id: channel_type\n  length: 1\n  type: bin\n\n- name: 通信地址\n  length: 8\n  type: switch\n  on: channel_type\n  cases:\n    '02':\n      type: container\n      fields:\n        - name: IP地址\n          length: 4\n          type: bin\n        - name: 端口号\n          length: 2\n          type: bin\n          endian: big"
        },
        "bitfield": {
            "description": "位域字段",
            "yaml": "- name: 运行状态字1\n  length: 2\n  type: bitfield\n  bits:\n    - range: [0, 0]\n      name: 保留\n    - range: [1, 1]\n      name: 需量积算方式\n      enum: { '1': 区间, '0': 滑差 }"
        }
    })
}
