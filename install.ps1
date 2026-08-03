# Spec Engine MCP 安装脚本 (PowerShell)
# 用途：自动编译并安装到 Kiro IDE

param(
    [switch]$Global,
    [switch]$Workspace,
    [string]$WorkspacePath = "",
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# 显示帮助信息
if ($Help) {
    Write-Host @"
Spec Engine MCP 安装脚本

用法:
  .\install.ps1 [-Global] [-Workspace] [-WorkspacePath <path>] [-Help]

选项:
  -Global          安装到全局用户配置 (~/.kiro/settings/mcp.json)
  -Workspace       安装到当前工作区 (.kiro/settings/mcp.json)
  -WorkspacePath   指定工作区路径（与 -Workspace 一起使用）
  -Help            显示此帮助信息

示例:
  .\install.ps1 -Global                # 安装到全局配置
  .\install.ps1 -Workspace             # 安装到当前目录的 .kiro/settings/mcp.json
  .\install.ps1 -Workspace -WorkspacePath "D:\MyProject"  # 安装到指定工作区

默认行为:
  如果未指定选项，将提示用户选择安装位置
"@
    exit 0
}

Write-Host "================================" -ForegroundColor Cyan
Write-Host " Spec Engine MCP 安装向导" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# 步骤 1: 编译 Release 版本
Write-Host "步骤 1/3: 编译 Release 版本..." -ForegroundColor Yellow
Write-Host ""

try {
    $buildOutput = cargo build --release 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ 编译失败！" -ForegroundColor Red
        Write-Host $buildOutput
        exit 1
    }
    Write-Host "✅ 编译成功！" -ForegroundColor Green
} catch {
    Write-Host "❌ 编译失败：$_" -ForegroundColor Red
    exit 1
}

Write-Host ""

# 步骤 2: 复制可执行文件到标准路径
Write-Host "步骤 2/4: 复制可执行文件..." -ForegroundColor Yellow
Write-Host ""

# 源文件路径
$sourceExe = Join-Path $PSScriptRoot "target\release\spec-engine-mcp.exe"
if (-not (Test-Path $sourceExe)) {
    Write-Host "❌ 找不到编译后的可执行文件：$sourceExe" -ForegroundColor Red
    exit 1
}

# 目标安装路径（用户级标准路径）
$installBinDir = Join-Path $env:LOCALAPPDATA "Programs\spec-engine-mcp"
$targetExe = Join-Path $installBinDir "spec-engine-mcp.exe"

# 确保安装目录存在
if (-not (Test-Path $installBinDir)) {
    Write-Host "创建安装目录：$installBinDir" -ForegroundColor Gray
    New-Item -ItemType Directory -Path $installBinDir -Force | Out-Null
}

# 复制可执行文件
try {
    Write-Host "复制 $sourceExe" -ForegroundColor Gray
    Write-Host "  到 $targetExe" -ForegroundColor Gray
    Copy-Item -Path $sourceExe -Destination $targetExe -Force
    Write-Host "✅ 可执行文件已安装到：$targetExe" -ForegroundColor Green
} catch {
    Write-Host "❌ 复制文件失败：$_" -ForegroundColor Red
    Write-Host "提示：如果文件正在使用，请先关闭相关程序" -ForegroundColor Yellow
    exit 1
}

# 在 Windows 上使用正斜杠（JSON 兼容）
$exePathJson = $targetExe -replace '\\', '/'

Write-Host ""

# 步骤 3: 确定配置安装位置
Write-Host "步骤 3/4: 确定配置安装位置..." -ForegroundColor Yellow
Write-Host ""

$installLocation = ""

if (-not $Global -and -not $Workspace) {
    # 交互式选择
    Write-Host "请选择安装位置：" -ForegroundColor White
    Write-Host "  1. 全局用户配置 (~/.kiro/settings/mcp.json)" -ForegroundColor White
    Write-Host "  2. 当前工作区 (.kiro/settings/mcp.json)" -ForegroundColor White
    Write-Host ""
    
    do {
        $choice = Read-Host "请输入选项 (1 或 2)"
    } while ($choice -ne "1" -and $choice -ne "2")
    
    if ($choice -eq "1") {
        $Global = $true
    } else {
        $Workspace = $true
    }
}

if ($Global) {
    $installLocation = Join-Path $env:USERPROFILE ".kiro\settings\mcp.json"
    Write-Host "📍 安装到全局配置：$installLocation" -ForegroundColor Cyan
} elseif ($Workspace) {
    if ($WorkspacePath -ne "") {
        $workspaceRoot = $WorkspacePath
    } else {
        $workspaceRoot = Get-Location
    }
    $installLocation = Join-Path $workspaceRoot ".kiro\settings\mcp.json"
    Write-Host "📍 安装到工作区配置：$installLocation" -ForegroundColor Cyan
}

Write-Host ""

# 步骤 4: 写入配置
Write-Host "步骤 4/4: 写入 MCP 配置..." -ForegroundColor Yellow
Write-Host ""

# 确保目录存在
$configDir = Split-Path $installLocation -Parent
if (-not (Test-Path $configDir)) {
    Write-Host "创建目录：$configDir" -ForegroundColor Gray
    New-Item -ItemType Directory -Path $configDir -Force | Out-Null
}

# MCP 配置对象
$mcpServerConfig = @{
    command = $exePathJson
    args = @()
    env = @{
        RUST_LOG = "info"
    }
    disabled = $false
}

# 读取或创建配置文件
$config = @{}
if (Test-Path $installLocation) {
    Write-Host "读取现有配置文件..." -ForegroundColor Gray
    try {
        $existingContent = Get-Content $installLocation -Raw -Encoding UTF8
        $config = $existingContent | ConvertFrom-Json -AsHashtable
        
        if (-not $config.ContainsKey("mcpServers")) {
            $config["mcpServers"] = @{}
        }
    } catch {
        Write-Host "⚠️  现有配置文件格式错误，将创建新文件" -ForegroundColor Yellow
        $config = @{
            mcpServers = @{}
        }
    }
} else {
    Write-Host "创建新配置文件..." -ForegroundColor Gray
    $config = @{
        mcpServers = @{}
    }
}

# 检查是否已存在 spec-engine 配置
if ($config.mcpServers.ContainsKey("spec-engine")) {
    Write-Host "⚠️  配置中已存在 'spec-engine' 服务器" -ForegroundColor Yellow
    $overwrite = Read-Host "是否覆盖现有配置？(y/N)"
    if ($overwrite -ne "y" -and $overwrite -ne "Y") {
        Write-Host "❌ 安装已取消" -ForegroundColor Red
        exit 0
    }
}

# 添加或更新配置
$config.mcpServers["spec-engine"] = $mcpServerConfig

# 写入文件
try {
    $jsonContent = $config | ConvertTo-Json -Depth 10
    # 确保 UTF-8 无 BOM
    [System.IO.File]::WriteAllText($installLocation, $jsonContent, [System.Text.UTF8Encoding]::new($false))
    Write-Host "✅ 配置已写入：$installLocation" -ForegroundColor Green
} catch {
    Write-Host "❌ 写入配置文件失败：$_" -ForegroundColor Red
    exit 1
}

Write-Host ""

# 完成
Write-Host "================================" -ForegroundColor Green
Write-Host " ✅ 安装成功！" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Green
Write-Host ""

Write-Host "配置详情：" -ForegroundColor White
Write-Host "  服务器名称：spec-engine" -ForegroundColor Gray
Write-Host "  可执行文件：$targetExe" -ForegroundColor Gray
Write-Host "  配置文件：$installLocation" -ForegroundColor Gray
Write-Host ""

Write-Host "下一步：" -ForegroundColor White
Write-Host "  1. 重启 Kiro IDE（或使用命令面板中的 'MCP: Reconnect Server'）" -ForegroundColor Gray
Write-Host "  2. MCP 服务器将自动加载" -ForegroundColor Gray
Write-Host "  3. AI 助手可以开始使用以下工具：" -ForegroundColor Gray
Write-Host "     - parse_data_item: 解析数据项内容" -ForegroundColor Gray
Write-Host "     - lookup_di: 查询 DI 定义" -ForegroundColor Gray
Write-Host "     - search_di: 搜索 DI" -ForegroundColor Gray
Write-Host "     - list_protocols: 列出支持的协议" -ForegroundColor Gray
Write-Host ""

Write-Host "测试命令：" -ForegroundColor White
Write-Host "  echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}' | & '$targetExe'" -ForegroundColor Gray
Write-Host ""

Write-Host "如需卸载，请运行：" -ForegroundColor White
Write-Host "  .\uninstall.ps1" -ForegroundColor Gray
Write-Host ""
