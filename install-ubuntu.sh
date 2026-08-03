#!/bin/bash
# spec-engine-mcp Ubuntu 安装脚本

set -e

echo "🚀 安装 spec-engine-mcp for Ubuntu 24.04"

# 检测架构
ARCH=$(uname -m)
if [ "$ARCH" != "x86_64" ]; then
    echo "❌ 错误: 仅支持 x86_64 架构"
    exit 1
fi

# 设置安装路径
INSTALL_DIR="$HOME/.local/bin"
BIN_NAME="spec-engine-mcp"

# 创建目录
mkdir -p "$INSTALL_DIR"

# 确定二进制文件路径
if [ -f "./target/x86_64-unknown-linux-musl/release/spec-engine-mcp" ]; then
    BINARY="./target/x86_64-unknown-linux-musl/release/spec-engine-mcp"
elif [ -f "./target/x86_64-unknown-linux-gnu/release/spec-engine-mcp" ]; then
    BINARY="./target/x86_64-unknown-linux-gnu/release/spec-engine-mcp"
elif [ -f "./target/release/spec-engine-mcp" ]; then
    BINARY="./target/release/spec-engine-mcp"
else
    echo "❌ 错误: 找不到编译好的二进制文件"
    echo "请先运行以下命令之一:"
    echo "  cargo build --release"
    echo "  cross build --release --target x86_64-unknown-linux-musl"
    exit 1
fi

# 复制二进制文件
echo "📦 复制 $BINARY 到 $INSTALL_DIR/$BIN_NAME"
cp "$BINARY" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME"

# 检查 PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "⚠️  $INSTALL_DIR 不在 PATH 中"
    echo "请将以下内容添加到 ~/.bashrc 或 ~/.zshrc:"
    echo ""
    echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "然后运行: source ~/.bashrc"
fi

# 验证安装
if [ -x "$INSTALL_DIR/$BIN_NAME" ]; then
    echo ""
    echo "✅ 安装成功！"
    echo ""
    echo "运行方式："
    echo "  $INSTALL_DIR/$BIN_NAME"
    echo ""
    echo "或者如果 $INSTALL_DIR 在 PATH 中："
    echo "  $BIN_NAME"
else
    echo "❌ 安装失败"
    exit 1
fi

# 创建 MCP 配置示例
CONFIG_DIR="$HOME/.kiro/settings"
CONFIG_FILE="$CONFIG_DIR/mcp.json"

if [ ! -f "$CONFIG_FILE" ]; then
    echo ""
    echo "📝 创建 MCP 配置文件: $CONFIG_FILE"
    mkdir -p "$CONFIG_DIR"
    cat > "$CONFIG_FILE" << 'EOF'
{
  "mcpServers": {
    "spec-engine": {
      "command": "$HOME/.local/bin/spec-engine-mcp",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      },
      "disabled": false
    }
  }
}
EOF
    echo "✅ MCP 配置已创建"
else
    echo ""
    echo "ℹ️  MCP 配置文件已存在: $CONFIG_FILE"
    echo "如需使用新安装的版本，请更新 command 路径为:"
    echo "  $INSTALL_DIR/$BIN_NAME"
fi

echo ""
echo "🎉 安装完成！"
