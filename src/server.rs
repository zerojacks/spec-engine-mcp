use anyhow::Result;
use serde_json::{json, Value as JsonValue};
use std::io::{BufRead, Write};
use tracing::{debug, error, info};

use crate::tools;

/// MCP协议消息
#[derive(Debug, serde::Deserialize)]
struct McpRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<JsonValue>,
    method: String,
    #[serde(default)]
    params: Option<JsonValue>,
}

/// MCP协议响应
#[derive(Debug, serde::Serialize)]
struct McpResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Debug, serde::Serialize)]
struct McpError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<JsonValue>,
}

/// MCP服务器
pub struct McpServer {
    stdin: std::io::Stdin,
    stdout: std::io::Stdout,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            stdin: std::io::stdin(),
            stdout: std::io::stdout(),
        }
    }

    /// 运行MCP服务器（stdio模式）
    pub fn run(&mut self) -> Result<()> {
        info!("Spec Engine MCP服务器启动");

        let reader = std::io::BufReader::new(&self.stdin);
        
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            debug!("收到请求: {}", line);

            let response = match serde_json::from_str::<McpRequest>(&line) {
                Ok(request) => self.handle_request(request),
                Err(e) => {
                    error!("解析请求失败: {}", e);
                    McpResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        result: None,
                        error: Some(McpError {
                            code: -32700,
                            message: "Parse error".to_string(),
                            data: Some(json!({"detail": e.to_string()})),
                        }),
                    }
                }
            };

            let response_json = serde_json::to_string(&response)?;
            debug!("发送响应: {}", response_json);
            
            writeln!(&mut self.stdout, "{}", response_json)?;
            self.stdout.flush()?;
        }

        Ok(())
    }

    fn handle_request(&self, request: McpRequest) -> McpResponse {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(request.params),
            _ => Err(anyhow::anyhow!("未知方法: {}", request.method)),
        };

        match result {
            Ok(result) => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(result),
                error: None,
            },
            Err(e) => {
                error!("处理请求失败: {}", e);
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(McpError {
                        code: -32603,
                        message: e.to_string(),
                        data: None,
                    }),
                }
            }
        }
    }

    fn handle_initialize(&self, _params: Option<JsonValue>) -> Result<JsonValue> {
        info!("初始化MCP服务器");
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": {
                "name": "spec-engine-mcp",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "capabilities": {
                "tools": {}
            }
        }))
    }

    fn handle_tools_list(&self) -> Result<JsonValue> {
        Ok(json!({
            "tools": [
                {
                    "name": "parse_data_item",
                    "description": "解析电力协议数据项内容。根据DI码定义，解析对应的数据项内容（hex_data）。支持DL/T645-2007、南网CSG13/CSG16等协议",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "protocol": {
                                "type": "string",
                                "description": "协议类型，如 'dlt645-2007', 'csg13', 'csg16'"
                            },
                            "di": {
                                "type": "string",
                                "description": "DI码（数据标识，十六进制字符串），如 '00010000'"
                            },
                            "hex_data": {
                                "type": "string",
                                "description": "数据项内容（十六进制字符串），如 '01020304'"
                            },
                            "region": {
                                "type": "string",
                                "description": "可选的省份/区域代码，如 '南网', '广东', '云南', '海南', '广西', '贵州', '深圳'"
                            },
                            "dir": {
                                "type": "string",
                                "description": "可选的方向：'0'表示下行，'1'表示上行，或省略"
                            }
                        },
                        "required": ["protocol", "di", "hex_data"]
                    }
                },
                {
                    "name": "lookup_di",
                    "description": "查询DI定义信息，包括名称、结构等",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "protocol": {
                                "type": "string",
                                "description": "协议类型"
                            },
                            "di": {
                                "type": "string",
                                "description": "DI码（十六进制字符串）"
                            },
                            "region": {
                                "type": "string",
                                "description": "可选的省份/区域代码"
                            },
                            "direction": {
                                "type": "string",
                                "description": "可选的方向：'0'表示下行，'1'表示上行，或省略"
                            }
                        },
                        "required": ["protocol", "di"]
                    }
                },
                {
                    "name": "list_protocols",
                    "description": "列出所有支持的协议及其统计信息",
                    "inputSchema": {
                        "type": "object",
                        "properties": {}
                    }
                },
                {
                    "name": "search_di",
                    "description": "搜索DI定义，支持按关键词、协议、区域过滤",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "keyword": {
                                "type": "string",
                                "description": "搜索关键词"
                            },
                            "protocol": {
                                "type": "string",
                                "description": "可选的协议过滤"
                            },
                            "region": {
                                "type": "string",
                                "description": "可选的区域过滤"
                            },
                            "limit": {
                                "type": "number",
                                "description": "最大返回结果数量，默认50"
                            }
                        },
                        "required": ["keyword"]
                    }
                },
                {
                    "name": "add_custom_di",
                    "description": "添加自定义DI定义。通过YAML格式提交DI定义数组，系统将验证、持久化并自动加载新定义",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "yaml_content": {
                                "type": "string",
                                "description": "YAML格式的DI定义数组，每个元素包含id、name、protocol等字段"
                            },
                            "force": {
                                "type": "boolean",
                                "description": "是否强制覆盖已存在的定义（默认false）"
                            },
                            "dry_run": {
                                "type": "boolean",
                                "description": "仅验证不保存（默认false）"
                            }
                        },
                        "required": ["yaml_content"]
                    }
                }
            ]
        }))
    }

    fn handle_tools_call(&self, params: Option<JsonValue>) -> Result<JsonValue> {
        let params = params.ok_or_else(|| anyhow::anyhow!("缺少参数"))?;
        
        let tool_name = params["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("缺少工具名称"))?;
        
        let arguments = params["arguments"].clone();

        let result = match tool_name {
            "parse_data_item" => {
                let input = serde_json::from_value(arguments)?;
                let output = tools::parse_data_item(input)?;
                serde_json::to_value(output)?
            }
            "lookup_di" => {
                let input = serde_json::from_value(arguments)?;
                let output = tools::lookup_di(input)?;
                serde_json::to_value(output)?
            }
            "list_protocols" => {
                let output = tools::list_protocols()?;
                serde_json::to_value(output)?
            }
            "search_di" => {
                let input = serde_json::from_value(arguments)?;
                let output = tools::search_di(input)?;
                serde_json::to_value(output)?
            }
            "add_custom_di" => {
                let input = serde_json::from_value(arguments)?;
                let output = tools::add_custom_di(input)?;
                serde_json::to_value(output)?
            }
            _ => {
                return Err(anyhow::anyhow!("未知工具: {}", tool_name));
            }
        };

        Ok(json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result)?
                }
            ]
        }))
    }
}
