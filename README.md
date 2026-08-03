# Spec Engine MCP Server

基于 [spec-engine](../spec-engine) 的 Model Context Protocol (MCP) 服务器，提供电力自动化协议解析能力。

## 📚 完整文档

- **[INSTALL.md](INSTALL.md)** - 🚀 **快速安装指南（从这里开始）**
- **[README_MCP_GUIDE.md](README_MCP_GUIDE.md)** - 完整的 MCP 使用指南
- **[HOW_AGENT_WORKS.md](HOW_AGENT_WORKS.md)** - Agent 如何识别和调用工具
- **[TOOL_DISCOVERY_FLOW.md](TOOL_DISCOVERY_FLOW.md)** - 工具发现流程详解
- **[DESCRIPTION_BEST_PRACTICES.md](DESCRIPTION_BEST_PRACTICES.md)** - 工具描述最佳实践
- **[INSTALLATION_GUIDE.md](INSTALLATION_GUIDE.md)** - 详细安装指南
- **[SCRIPTS_GUIDE.md](SCRIPTS_GUIDE.md)** - 脚本使用指南
- **[DEPLOYMENT.md](DEPLOYMENT.md)** - 部署指南
- **[TEST_RESULTS.md](TEST_RESULTS.md)** - 测试结果

## 功能特性

- 🔌 **协议解析**：支持 DL/T645-2007、南网CSG13/CSG16 等协议
- 🔍 **DI查询**：快速查询数据标识定义和结构
- 📊 **协议统计**：列出所有支持的协议及统计信息
- 🔎 **智能搜索**：按关键词搜索DI定义

## 支持的协议

- **DL/T 645-2007**: 多功能电能表通信协议
- **Q/CSG 1209013-2019**: 南网CSG13（计量自动化终端上行通信规约）
- **Q/CSG 1209016-2023**: 南网CSG16（新一代计量自动化终端上行通信规约）

## 快速开始

### 方式一：自动安装（推荐）

使用提供的安装脚本自动编译、安装到标准系统路径并配置到 Kiro IDE：

**安装位置：**
- Windows: `%LOCALAPPDATA%\Programs\spec-engine-mcp\spec-engine-mcp.exe`
- Linux/macOS: `~/.local/bin/spec-engine-mcp`

#### Windows (PowerShell)

```powershell
# 安装到全局配置
.\install.ps1 -Global

# 或安装到当前工作区
.\install.ps1 -Workspace

# 测试安装
.\test_installation.ps1
```

#### Linux / macOS / Git Bash

```bash
# 赋予执行权限
chmod +x install.sh test_installation.sh

# 安装到全局配置
./install.sh --global

# 或安装到当前工作区
./install.sh --workspace

# 测试安装
./test_installation.sh
```

#### 使用 Makefile（推荐）

```bash
# 编译、安装到全局配置并测试
make install test

# 或安装到工作区
make install-workspace test

# 查看所有可用命令
make help
```

### 方式二：手动安装

#### 1. 编译

```bash
cargo build --release
```

#### 2. 配置 Kiro IDE

创建或编辑 `.kiro/settings/mcp.json`：

**全局配置** (`~/.kiro/settings/mcp.json`)：

Windows:
```json
{
  "mcpServers": {
    "spec-engine": {
      "command": "C:/Users/<YourUsername>/AppData/Local/Programs/spec-engine-mcp/spec-engine-mcp.exe",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      },
      "disabled": false
    }
  }
}
```

Linux/macOS:
```json
{
  "mcpServers": {
    "spec-engine": {
      "command": "/home/<username>/.local/bin/spec-engine-mcp",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      },
      "disabled": false
    }
  }
}
```

**工作区配置** (`<workspace>/.kiro/settings/mcp.json`)：同上

**注意**：
- Windows 路径使用正斜杠 `/` 或双反斜杠 `\\`
- 将 `<YourUsername>` 或 `<username>` 替换为实际的用户名
- 自动安装脚本会自动配置正确的路径

详细安装说明请参考 [INSTALLATION_GUIDE.md](INSTALLATION_GUIDE.md)

### 使用示例

连接MCP服务器后，可以使用以下工具：

#### 1. 解析数据项内容

```
使用 parse_data_item 工具解析这个数据项内容：
- 协议：dlt645-2007
- DI：00010000（数据标识）
- 数据：01020304（数据项内容的十六进制）
```

#### 2. 查询DI定义

```
查询 CSG13 协议中 DI 为 00010001 的定义
```

#### 3. 列出协议

```
列出所有支持的电力协议
```

#### 4. 搜索DI

```
搜索所有包含"正向有功"的DI
```

## 提供的工具

### parse_data_item

解析电力协议数据项内容。根据DI码定义，解析对应的数据项内容（hex_data）。

**输入参数：**
- `protocol` (string, 必需): 协议类型，如 "dlt645-2007"
- `di` (string, 必需): DI码（数据标识，十六进制），如 "00010000"
- `hex_data` (string, 必需): 数据项内容（十六进制），如 "01020304"
- `region` (string, 可选): 省份/区域代码，如 "GD"
- `dir` (string, 可选): 方向（上行/下行）

**输出：**
- `success` (boolean): 解析是否成功
- `data` (object): 解析结果（JSON格式）
- `consumed_bytes` (number): 消耗的字节数
- `error` (string): 错误信息（如果失败）

### lookup_di

查询DI定义信息。

**输入参数：**
- `protocol` (string, 必需): 协议类型
- `di` (string, 必需): DI码（十六进制）
- `region` (string, 可选): 省份/区域代码
- `direction` (string, 可选): 方向（上行/下行）

**输出：**
- `success` (boolean): 查询是否成功
- `di_info` (object): DI信息（名称、结构等）
- `error` (string): 错误信息（如果失败）

### list_protocols

列出所有支持的协议。

**输入参数：** 无

**输出：**
- `protocols` (array): 协议列表
  - `name` (string): 协议名称
  - `di_count` (number): DI条目数量
  - `regions` (array): 支持的区域
- `total` (number): 协议总数

### search_di

搜索DI定义。

**输入参数：**
- `keyword` (string, 必需): 搜索关键词
- `protocol` (string, 可选): 协议过滤
- `region` (string, 可选): 区域过滤
- `limit` (number, 可选): 最大结果数量（默认50）

**输出：**
- `results` (array): 搜索结果
  - `di` (string): DI码
  - `name` (string): DI名称
  - `protocol` (string): 协议
  - `region` (string): 区域
- `count` (number): 结果数量
- `has_more` (boolean): 是否有更多结果

## 卸载

### 自动卸载

```bash
# Windows
.\uninstall.ps1 -Global
# 或
.\uninstall.ps1 -Workspace

# Linux/macOS
./uninstall.sh --global
# 或
./uninstall.sh --workspace

# 使用 Makefile
make uninstall
```

### 手动卸载

编辑配置文件并移除 `spec-engine` 节点，然后重启 Kiro IDE。

## 开发

### 项目结构

```
spec-engine-mcp/
├── src/
│   ├── main.rs              # 程序入口
│   ├── server.rs            # MCP服务器实现
│   └── tools/               # 工具实现
│       ├── mod.rs
│       ├── parse_data_item.rs # 数据项解析
│       ├── lookup_di.rs     # DI查询
│       ├── list_protocols.rs # 协议列表
│       └── search_di.rs     # DI搜索
├── Cargo.toml
└── README.md
```

### 添加新工具

1. 在 `src/tools/` 下创建新文件
2. 实现工具函数
3. 在 `src/tools/mod.rs` 中导出
4. 在 `src/server.rs` 中注册工具

### 测试

```bash
# 运行测试
cargo test

# 手动测试MCP协议
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run
```

## 依赖

- [spec-engine](../spec-engine): 核心协议解析引擎
- tokio: 异步运行时
- serde/serde_json: JSON序列化
- tracing: 日志记录

## 许可证

同 spec-engine 项目

## 相关资源

- [MCP 协议规范](https://modelcontextprotocol.io/)
- [Spec Engine 文档](../spec-engine/README.md)
- [DL/T 645-2007 标准](https://www.doc88.com/p-9974447699724.html)
