# Spec Engine MCP Makefile
# 提供常用命令的快捷方式

.PHONY: help build release install install-global install-workspace uninstall test clean run

# 默认目标：显示帮助
help:
	@echo "Spec Engine MCP - 可用命令："
	@echo ""
	@echo "  make build            - 编译 Debug 版本"
	@echo "  make release          - 编译 Release 版本"
	@echo "  make install-global   - 安装到全局配置"
	@echo "  make install-workspace - 安装到当前工作区"
	@echo "  make uninstall        - 卸载"
	@echo "  make test             - 测试安装"
	@echo "  make run              - 运行服务器（stdin/stdout 模式）"
	@echo "  make clean            - 清理编译产物"
	@echo ""
	@echo "示例："
	@echo "  make release install-global test"
	@echo ""

# 编译 Debug 版本
build:
	@echo "编译 Debug 版本..."
	cargo build

# 编译 Release 版本
release:
	@echo "编译 Release 版本..."
	cargo build --release

# 安装到全局配置
install-global: release
	@echo "安装到全局配置..."
ifeq ($(OS),Windows_NT)
	powershell.exe -ExecutionPolicy Bypass -File install.ps1 -Global
else
	./install.sh --global
endif

# 安装到当前工作区
install-workspace: release
	@echo "安装到当前工作区..."
ifeq ($(OS),Windows_NT)
	powershell.exe -ExecutionPolicy Bypass -File install.ps1 -Workspace
else
	./install.sh --workspace
endif

# 快捷方式：install 默认为全局安装
install: install-global

# 卸载
uninstall:
	@echo "卸载..."
ifeq ($(OS),Windows_NT)
	powershell.exe -ExecutionPolicy Bypass -File uninstall.ps1
else
	./uninstall.sh
endif

# 测试安装
test:
	@echo "测试安装..."
ifeq ($(OS),Windows_NT)
	powershell.exe -ExecutionPolicy Bypass -File test_installation.ps1
else
	./test_installation.sh
endif

# 运行服务器（用于测试）
run: release
	@echo "运行 MCP 服务器..."
	@echo "输入 JSON-RPC 请求（Ctrl+C 退出）："
ifeq ($(OS),Windows_NT)
	target\release\spec-engine-mcp.exe
else
	target/release/spec-engine-mcp
endif

# 清理编译产物
clean:
	@echo "清理编译产物..."
	cargo clean

# 完整重新安装
reinstall: clean release install-global test
	@echo "重新安装完成！"

# 开发模式：编译并测试（不安装）
dev: release
	@echo "测试编译结果..."
ifeq ($(OS),Windows_NT)
	@echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | target\release\spec-engine-mcp.exe
else
	@echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | target/release/spec-engine-mcp
endif

# 快速测试：初始化 + 工具列表
quick-test: release
	@echo "快速测试..."
ifeq ($(OS),Windows_NT)
	@echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | target\release\spec-engine-mcp.exe 2>&1 | findstr "{"
	@echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | target\release\spec-engine-mcp.exe 2>&1 | findstr "{"
else
	@echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | target/release/spec-engine-mcp 2>&1 | grep '^{'
	@echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | target/release/spec-engine-mcp 2>&1 | grep '^{'
endif

# 格式化代码
fmt:
	@echo "格式化代码..."
	cargo fmt

# 代码检查
lint:
	@echo "运行代码检查..."
	cargo clippy -- -D warnings

# 运行单元测试
unit-test:
	@echo "运行单元测试..."
	cargo test

# CI 流程：格式化、检查、测试、编译
ci: fmt lint unit-test release
	@echo "CI 检查完成！"
