#!/bin/bash
# Spec Engine MCP 安装脚本 (Bash)
# 用途：自动编译并安装到 Kiro IDE

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
NC='\033[0m' # No Color

# 显示帮助信息
show_help() {
    cat << EOF
Spec Engine MCP 安装脚本

用法:
  ./install.sh [选项]

选项:
  --global         安装到全局用户配置 (~/.kiro/settings/mcp.json)
  --workspace      安装到当前工作区 (.kiro/settings/mcp.json)
  --workspace-path <path>  指定工作区路径（与 --workspace 一起使用）
  --help           显示此帮助信息

示例:
  ./install.sh --global                      # 安装到全局配置
  ./install.sh --workspace                   # 安装到当前目录的工作区配置
  ./install.sh --workspace --workspace-path /path/to/workspace  # 安装到指定工作区

默认行为:
  如果未指定选项，将提示用户选择安装位置

EOF
    exit 0
}

# 解析参数
GLOBAL=false
WORKSPACE=false
WORKSPACE_PATH=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --global)
            GLOBAL=true
            shift
            ;;
        --workspace)
            WORKSPACE=true
            shift
            ;;
        --workspace-path)
            WORKSPACE_PATH="$2"
            shift 2
            ;;
        --help)
            show_help
            ;;
        *)
            echo -e "${RED}未知选项: $1${NC}"
            echo "使用 --help 查看帮助"
            exit 1
            ;;
    esac
done

echo -e "${CYAN}================================${NC}"
echo -e "${CYAN} Spec Engine MCP 安装向导${NC}"
echo -e "${CYAN}================================${NC}"
echo ""

# 步骤 1: 编译 Release 版本
echo -e "${YELLOW}步骤 1/3: 编译 Release 版本...${NC}"
echo ""

if cargo build --release; then
    echo -e "${GREEN}✅ 编译成功！${NC}"
else
    echo -e "${RED}❌ 编译失败！${NC}"
    exit 1
fi

echo ""

# 步骤 2: 复制可执行文件到标准路径
echo -e "${YELLOW}步骤 2/4: 复制可执行文件...${NC}"
echo ""

# 源文件路径
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXE_NAME="spec-engine-mcp"
SOURCE_EXE="$SCRIPT_DIR/target/release/$EXE_NAME"

if [ ! -f "$SOURCE_EXE" ]; then
    echo -e "${RED}❌ 找不到编译后的可执行文件：$SOURCE_EXE${NC}"
    exit 1
fi

# 目标安装路径（用户级标准路径）
INSTALL_BIN_DIR="$HOME/.local/bin"
TARGET_EXE="$INSTALL_BIN_DIR/$EXE_NAME"

# 确保安装目录存在
if [ ! -d "$INSTALL_BIN_DIR" ]; then
    echo -e "${GRAY}创建安装目录：$INSTALL_BIN_DIR${NC}"
    mkdir -p "$INSTALL_BIN_DIR"
fi

# 复制可执行文件
echo -e "${GRAY}复制 $SOURCE_EXE${NC}"
echo -e "${GRAY}  到 $TARGET_EXE${NC}"

if cp "$SOURCE_EXE" "$TARGET_EXE"; then
    chmod +x "$TARGET_EXE"
    echo -e "${GREEN}✅ 可执行文件已安装到：$TARGET_EXE${NC}"
else
    echo -e "${RED}❌ 复制文件失败${NC}"
    echo -e "${YELLOW}提示：如果文件正在使用，请先关闭相关程序${NC}"
    exit 1
fi

# 检查 ~/.local/bin 是否在 PATH 中
if [[ ":$PATH:" != *":$INSTALL_BIN_DIR:"* ]]; then
    echo -e "${YELLOW}⚠️  $INSTALL_BIN_DIR 不在 PATH 中${NC}"
    echo -e "${YELLOW}建议添加以下行到您的 shell 配置文件（~/.bashrc 或 ~/.zshrc）：${NC}"
    echo -e "${GRAY}  export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
fi

# 在 Git Bash/MSYS 环境下，转换路径为 Windows 格式
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    # 将 /c/Users/... 转换为 C:/Users/... (不加 .exe)
    EXE_PATH_JSON=$(cygpath -w "$TARGET_EXE" 2>/dev/null || echo "$TARGET_EXE")
    EXE_PATH_JSON="${EXE_PATH_JSON//\\//}"  # 将反斜杠替换为正斜杠
    # 移除可能被 cygpath 添加的 .exe 扩展名（如果原本没有）
    if [[ "$TARGET_EXE" != *.exe ]] && [[ "$EXE_PATH_JSON" == *.exe ]]; then
        EXE_PATH_JSON="${EXE_PATH_JSON%.exe}"
    fi
    echo -e "${GRAY}转换为 Windows 路径：$EXE_PATH_JSON${NC}"
else
    EXE_PATH_JSON="$TARGET_EXE"
fi

echo ""

# 步骤 3: 确定配置安装位置
echo -e "${YELLOW}步骤 3/4: 确定配置安装位置...${NC}"
echo ""

if [ "$GLOBAL" = false ] && [ "$WORKSPACE" = false ]; then
    # 交互式选择
    echo -e "${NC}请选择安装位置：${NC}"
    echo "  1. 全局用户配置 (~/.kiro/settings/mcp.json)"
    echo "  2. 当前工作区 (.kiro/settings/mcp.json)"
    echo ""
    
    while true; do
        read -p "请输入选项 (1 或 2): " choice
        case $choice in
            1)
                GLOBAL=true
                break
                ;;
            2)
                WORKSPACE=true
                break
                ;;
            *)
                echo "无效选项，请输入 1 或 2"
                ;;
        esac
    done
fi

if [ "$GLOBAL" = true ]; then
    INSTALL_LOCATION="$HOME/.kiro/settings/mcp.json"
    echo -e "${CYAN}📍 安装到全局配置：$INSTALL_LOCATION${NC}"
elif [ "$WORKSPACE" = true ]; then
    if [ -n "$WORKSPACE_PATH" ]; then
        WORKSPACE_ROOT="$WORKSPACE_PATH"
    else
        WORKSPACE_ROOT="$(pwd)"
    fi
    INSTALL_LOCATION="$WORKSPACE_ROOT/.kiro/settings/mcp.json"
    echo -e "${CYAN}📍 安装到工作区配置：$INSTALL_LOCATION${NC}"
fi

echo ""

# 步骤 4: 写入配置
echo -e "${YELLOW}步骤 4/4: 写入 MCP 配置...${NC}"
echo ""

# 确保目录存在
CONFIG_DIR=$(dirname "$INSTALL_LOCATION")
if [ ! -d "$CONFIG_DIR" ]; then
    echo -e "${GRAY}创建目录：$CONFIG_DIR${NC}"
    mkdir -p "$CONFIG_DIR"
fi

# 在 Windows 环境下，转换配置文件路径为 Windows 格式供 Python 使用
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    CONFIG_FILE_FOR_PYTHON=$(cygpath -w "$INSTALL_LOCATION" 2>/dev/null || echo "$INSTALL_LOCATION")
    CONFIG_FILE_FOR_PYTHON="${CONFIG_FILE_FOR_PYTHON//\\//}"  # 将反斜杠替换为正斜杠
else
    CONFIG_FILE_FOR_PYTHON="$INSTALL_LOCATION"
fi

# 读取或创建配置文件
if [ -f "$INSTALL_LOCATION" ]; then
    echo -e "${GRAY}读取现有配置文件...${NC}"
    
    # 检查是否已存在 spec-engine 配置
    if grep -q '"spec-engine"' "$INSTALL_LOCATION"; then
        echo -e "${YELLOW}⚠️  配置中已存在 'spec-engine' 服务器${NC}"
        read -p "是否覆盖现有配置？(y/N): " overwrite
        if [[ ! "$overwrite" =~ ^[Yy]$ ]]; then
            echo -e "${RED}❌ 安装已取消${NC}"
            exit 0
        fi
        
        # 使用 jq 或 python 移除旧配置
        if command -v jq &> /dev/null; then
            TMP_FILE=$(mktemp)
            jq 'del(.mcpServers["spec-engine"])' "$INSTALL_LOCATION" > "$TMP_FILE"
            mv "$TMP_FILE" "$INSTALL_LOCATION"
        elif command -v python &> /dev/null; then
            python << PYEOF
import json

config_file = r'$CONFIG_FILE_FOR_PYTHON'

with open(config_file, 'r', encoding='utf-8') as f:
    config = json.load(f)

if 'mcpServers' in config and 'spec-engine' in config['mcpServers']:
    del config['mcpServers']['spec-engine']

with open(config_file, 'w', encoding='utf-8') as f:
    json.dump(config, f, indent=2, ensure_ascii=False)
PYEOF
        elif command -v python3 &> /dev/null; then
            python3 << PYEOF
import json

config_file = r'$CONFIG_FILE_FOR_PYTHON'

with open(config_file, 'r', encoding='utf-8') as f:
    config = json.load(f)

if 'mcpServers' in config and 'spec-engine' in config['mcpServers']:
    del config['mcpServers']['spec-engine']

with open(config_file, 'w', encoding='utf-8') as f:
    json.dump(config, f, indent=2, ensure_ascii=False)
PYEOF
        fi
    fi
    
    # 添加新配置
    if command -v jq &> /dev/null; then
        TMP_FILE=$(mktemp)
        jq --arg cmd "$EXE_PATH_JSON" \
           '.mcpServers["spec-engine"] = {
               "command": $cmd,
               "args": [],
               "env": {"RUST_LOG": "info"},
               "disabled": false
           }' "$INSTALL_LOCATION" > "$TMP_FILE"
        mv "$TMP_FILE" "$INSTALL_LOCATION"
    elif command -v python &> /dev/null; then
        python << PYEOF
import json
import sys

config_file = r'$CONFIG_FILE_FOR_PYTHON'
exe_path = r'$EXE_PATH_JSON'

try:
    with open(config_file, 'r', encoding='utf-8') as f:
        config = json.load(f)
except:
    config = {}

if 'mcpServers' not in config:
    config['mcpServers'] = {}

config['mcpServers']['spec-engine'] = {
    'command': exe_path,
    'args': [],
    'env': {'RUST_LOG': 'info'},
    'disabled': False
}

with open(config_file, 'w', encoding='utf-8') as f:
    json.dump(config, f, indent=2, ensure_ascii=False)

print('配置已更新')
PYEOF
    elif command -v python3 &> /dev/null; then
        python3 << PYEOF
import json
import sys

config_file = r'$CONFIG_FILE_FOR_PYTHON'
exe_path = r'$EXE_PATH_JSON'

try:
    with open(config_file, 'r', encoding='utf-8') as f:
        config = json.load(f)
except:
    config = {}

if 'mcpServers' not in config:
    config['mcpServers'] = {}

config['mcpServers']['spec-engine'] = {
    'command': exe_path,
    'args': [],
    'env': {'RUST_LOG': 'info'},
    'disabled': False
}

with open(config_file, 'w', encoding='utf-8') as f:
    json.dump(config, f, indent=2, ensure_ascii=False)

print('配置已更新')
PYEOF
    else
        echo -e "${RED}❌ 需要 jq 或 python3 来处理 JSON 配置${NC}"
        echo "请手动添加以下配置到 $INSTALL_LOCATION:"
        echo ""
        cat << EOF
{
  "mcpServers": {
    "spec-engine": {
      "command": "$EXE_PATH_JSON",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      },
      "disabled": false
    }
  }
}
EOF
        exit 1
    fi
else
    echo -e "${GRAY}创建新配置文件...${NC}"
    
    # 使用 Python 创建新配置（优先 python，再 python3）
    if command -v python &> /dev/null; then
        python << PYEOF
import json

config_file = r'$CONFIG_FILE_FOR_PYTHON'
exe_path = r'$EXE_PATH_JSON'

config = {
    'mcpServers': {
        'spec-engine': {
            'command': exe_path,
            'args': [],
            'env': {'RUST_LOG': 'info'},
            'disabled': False
        }
    }
}

with open(config_file, 'w', encoding='utf-8') as f:
    json.dump(config, f, indent=2, ensure_ascii=False)

print('新配置已创建')
PYEOF
    elif command -v python3 &> /dev/null; then
        python3 << PYEOF
import json

config_file = r'$CONFIG_FILE_FOR_PYTHON'
exe_path = r'$EXE_PATH_JSON'

config = {
    'mcpServers': {
        'spec-engine': {
            'command': exe_path,
            'args': [],
            'env': {'RUST_LOG': 'info'},
            'disabled': False
        }
    }
}

with open(config_file, 'w', encoding='utf-8') as f:
    json.dump(config, f, indent=2, ensure_ascii=False)

print('新配置已创建')
PYEOF
    else
        # 如果没有 Python，使用 cat（可能有变量展开问题）
        cat > "$INSTALL_LOCATION" << EOF
{
  "mcpServers": {
    "spec-engine": {
      "command": "$EXE_PATH_JSON",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      },
      "disabled": false
    }
  }
}
EOF
    fi
fi

echo -e "${GREEN}✅ 配置已写入：$INSTALL_LOCATION${NC}"
echo ""

# 完成
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN} ✅ 安装成功！${NC}"
echo -e "${GREEN}================================${NC}"
echo ""

echo -e "${NC}配置详情：${NC}"
echo -e "${GRAY}  服务器名称：spec-engine${NC}"
echo -e "${GRAY}  可执行文件：$TARGET_EXE${NC}"
echo -e "${GRAY}  配置文件：$INSTALL_LOCATION${NC}"
echo ""

echo -e "${NC}下一步：${NC}"
echo -e "${GRAY}  1. 重启 Kiro IDE（或使用命令面板中的 'MCP: Reconnect Server'）${NC}"
echo -e "${GRAY}  2. MCP 服务器将自动加载${NC}"
echo -e "${GRAY}  3. AI 助手可以开始使用以下工具：${NC}"
echo -e "${GRAY}     - parse_data_item: 解析数据项内容${NC}"
echo -e "${GRAY}     - lookup_di: 查询 DI 定义${NC}"
echo -e "${GRAY}     - search_di: 搜索 DI${NC}"
echo -e "${GRAY}     - list_protocols: 列出支持的协议${NC}"
echo ""

echo -e "${NC}测试命令：${NC}"
echo -e "${GRAY}  echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}' | '$TARGET_EXE'${NC}"
echo ""

echo -e "${NC}如需卸载，请运行：${NC}"
echo -e "${GRAY}  ./uninstall.sh${NC}"
echo ""
