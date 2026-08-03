#!/bin/bash
# Spec Engine MCP 卸载脚本 (Bash)
# 用途：从 Kiro IDE 配置中移除 MCP 服务器

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
Spec Engine MCP 卸载脚本

用法:
  ./uninstall.sh [选项]

选项:
  --global         从全局用户配置中卸载
  --workspace      从当前工作区配置中卸载
  --workspace-path <path>  指定工作区路径（与 --workspace 一起使用）
  --help           显示此帮助信息

示例:
  ./uninstall.sh --global                      # 从全局配置卸载
  ./uninstall.sh --workspace                   # 从当前工作区卸载
  ./uninstall.sh --workspace --workspace-path /path/to/workspace  # 从指定工作区卸载

默认行为:
  如果未指定选项，将提示用户选择卸载位置

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
echo -e "${CYAN} Spec Engine MCP 卸载向导${NC}"
echo -e "${CYAN}================================${NC}"
echo ""

# 步骤 1: 删除已安装的可执行文件
echo -e "${YELLOW}步骤 1/2: 删除已安装的可执行文件...${NC}"
echo ""

INSTALL_BIN_DIR="$HOME/.local/bin"
INSTALLED_EXE="$INSTALL_BIN_DIR/spec-engine-mcp"

if [ -f "$INSTALLED_EXE" ]; then
    echo -e "${GRAY}删除：$INSTALLED_EXE${NC}"
    if rm "$INSTALLED_EXE"; then
        echo -e "${GREEN}✅ 可执行文件已删除${NC}"
    else
        echo -e "${YELLOW}⚠️  删除可执行文件失败${NC}"
        echo -e "${YELLOW}提示：如果文件正在使用，请先关闭相关程序${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  未找到已安装的可执行文件${NC}"
fi

echo ""

# 步骤 2: 从配置文件中移除
echo -e "${YELLOW}步骤 2/2: 从配置文件中移除...${NC}"
echo ""

# 确定卸载位置
if [ "$GLOBAL" = false ] && [ "$WORKSPACE" = false ]; then
    # 交互式选择
    echo -e "${NC}请选择卸载位置：${NC}"
    echo "  1. 全局用户配置 (~/.kiro/settings/mcp.json)"
    echo "  2. 当前工作区 (.kiro/settings/mcp.json)"
    echo "  3. 两者都卸载"
    echo ""
    
    while true; do
        read -p "请输入选项 (1, 2 或 3): " choice
        case $choice in
            1)
                GLOBAL=true
                break
                ;;
            2)
                WORKSPACE=true
                break
                ;;
            3)
                GLOBAL=true
                WORKSPACE=true
                break
                ;;
            *)
                echo "无效选项，请输入 1, 2 或 3"
                ;;
        esac
    done
fi

LOCATIONS=()

if [ "$GLOBAL" = true ]; then
    LOCATIONS+=("$HOME/.kiro/settings/mcp.json|全局配置")
fi

if [ "$WORKSPACE" = true ]; then
    if [ -n "$WORKSPACE_PATH" ]; then
        WORKSPACE_ROOT="$WORKSPACE_PATH"
    else
        WORKSPACE_ROOT="$(pwd)"
    fi
    LOCATIONS+=("$WORKSPACE_ROOT/.kiro/settings/mcp.json|工作区配置")
fi

REMOVED_COUNT=0
NOT_FOUND_COUNT=0

for location_info in "${LOCATIONS[@]}"; do
    IFS='|' read -r config_path config_name <<< "$location_info"
    
    echo -e "${YELLOW}处理 $config_name...${NC}"
    echo -e "${GRAY}  路径：$config_path${NC}"
    
    if [ ! -f "$config_path" ]; then
        echo -e "${YELLOW}  ⚠️  配置文件不存在${NC}"
        ((NOT_FOUND_COUNT++))
        echo ""
        continue
    fi
    
    # 检查是否包含 spec-engine 配置
    if ! grep -q '"spec-engine"' "$config_path"; then
        echo -e "${YELLOW}  ⚠️  配置中没有 spec-engine 服务器${NC}"
        ((NOT_FOUND_COUNT++))
        echo ""
        continue
    fi
    
    # 移除配置
    if command -v jq &> /dev/null; then
        TMP_FILE=$(mktemp)
        jq 'del(.mcpServers["spec-engine"])' "$config_path" > "$TMP_FILE"
        mv "$TMP_FILE" "$config_path"
        echo -e "${GREEN}  ✅ 已从配置中移除 spec-engine${NC}"
        ((REMOVED_COUNT++))
    elif command -v python3 &> /dev/null; then
        python3 -c "
import json
with open('$config_path', 'r', encoding='utf-8') as f:
    config = json.load(f)
if 'mcpServers' in config and 'spec-engine' in config['mcpServers']:
    del config['mcpServers']['spec-engine']
    with open('$config_path', 'w', encoding='utf-8') as f:
        json.dump(config, f, indent=2, ensure_ascii=False)
    print('removed')
else:
    print('not_found')
" | {
            read result
            if [ "$result" = "removed" ]; then
                echo -e "${GREEN}  ✅ 已从配置中移除 spec-engine${NC}"
                ((REMOVED_COUNT++))
            else
                echo -e "${YELLOW}  ⚠️  配置中没有 spec-engine 服务器${NC}"
                ((NOT_FOUND_COUNT++))
            fi
        }
    else
        echo -e "${RED}  ❌ 需要 jq 或 python3 来处理 JSON 配置${NC}"
        echo "  请手动编辑 $config_path 并移除 spec-engine 配置"
    fi
    
    echo ""
done

# 总结
echo -e "${CYAN}================================${NC}"
echo -e "${CYAN} 卸载完成${NC}"
echo -e "${CYAN}================================${NC}"
echo ""

if [ $REMOVED_COUNT -gt 0 ]; then
    echo -e "${GREEN}✅ 成功移除 $REMOVED_COUNT 个配置${NC}"
fi

if [ $NOT_FOUND_COUNT -gt 0 ]; then
    echo -e "${YELLOW}⚠️  $NOT_FOUND_COUNT 个位置未找到配置${NC}"
fi

echo ""

if [ $REMOVED_COUNT -gt 0 ]; then
    echo -e "${NC}下一步：${NC}"
    echo -e "${GRAY}  1. 重启 Kiro IDE（或使用命令面板中的 'MCP: Reconnect Server'）${NC}"
    echo -e "${GRAY}  2. spec-engine 服务器将不再可用${NC}"
    echo ""
    
    echo -e "${NC}如需重新安装，请运行：${NC}"
    echo -e "${GRAY}  ./install.sh${NC}"
    echo ""
fi
