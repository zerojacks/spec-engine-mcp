# Spec Engine MCP 卸载脚本 (PowerShell)
# 用途：从 Kiro IDE 配置中移除 MCP 服务器

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
Spec Engine MCP 卸载脚本

用法:
  .\uninstall.ps1 [-Global] [-Workspace] [-WorkspacePath <path>] [-Help]

选项:
  -Global          从全局用户配置中卸载
  -Workspace       从当前工作区配置中卸载
  -WorkspacePath   指定工作区路径（与 -Workspace 一起使用）
  -Help            显示此帮助信息

示例:
  .\uninstall.ps1 -Global                # 从全局配置卸载
  .\uninstall.ps1 -Workspace             # 从当前工作区卸载
  .\uninstall.ps1 -Workspace -WorkspacePath "D:\MyProject"  # 从指定工作区卸载

默认行为:
  如果未指定选项，将提示用户选择卸载位置
"@
    exit 0
}

Write-Host "================================" -ForegroundColor Cyan
Write-Host " Spec Engine MCP 卸载向导" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# 步骤 1: 删除已安装的可执行文件
Write-Host "步骤 1/2: 删除已安装的可执行文件..." -ForegroundColor Yellow
Write-Host ""

$installBinDir = Join-Path $env:LOCALAPPDATA "Programs\spec-engine-mcp"
$installedExe = Join-Path $installBinDir "spec-engine-mcp.exe"

if (Test-Path $installedExe) {
    try {
        Write-Host "删除：$installedExe" -ForegroundColor Gray
        Remove-Item -Path $installedExe -Force
        Write-Host "✅ 可执行文件已删除" -ForegroundColor Green
        
        # 如果目录为空，也删除目录
        if ((Get-ChildItem -Path $installBinDir -Force | Measure-Object).Count -eq 0) {
            Remove-Item -Path $installBinDir -Force
            Write-Host "✅ 安装目录已删除" -ForegroundColor Green
        }
    } catch {
        Write-Host "⚠️  删除可执行文件失败：$_" -ForegroundColor Yellow
        Write-Host "提示：如果文件正在使用，请先关闭相关程序" -ForegroundColor Yellow
    }
} else {
    Write-Host "⚠️  未找到已安装的可执行文件" -ForegroundColor Yellow
}

Write-Host ""

# 步骤 2: 从配置文件中移除
Write-Host "步骤 2/2: 从配置文件中移除..." -ForegroundColor Yellow
Write-Host ""

# 确定卸载位置
$uninstallLocation = ""

if (-not $Global -and -not $Workspace) {
    # 交互式选择
    Write-Host "请选择卸载位置：" -ForegroundColor White
    Write-Host "  1. 全局用户配置 (~/.kiro/settings/mcp.json)" -ForegroundColor White
    Write-Host "  2. 当前工作区 (.kiro/settings/mcp.json)" -ForegroundColor White
    Write-Host "  3. 两者都卸载" -ForegroundColor White
    Write-Host ""
    
    do {
        $choice = Read-Host "请输入选项 (1, 2 或 3)"
    } while ($choice -ne "1" -and $choice -ne "2" -and $choice -ne "3")
    
    if ($choice -eq "1") {
        $Global = $true
    } elseif ($choice -eq "2") {
        $Workspace = $true
    } else {
        $Global = $true
        $Workspace = $true
    }
}

$locations = @()

if ($Global) {
    $globalPath = Join-Path $env:USERPROFILE ".kiro\settings\mcp.json"
    $locations += @{ Path = $globalPath; Name = "全局配置" }
}

if ($Workspace) {
    if ($WorkspacePath -ne "") {
        $workspaceRoot = $WorkspacePath
    } else {
        $workspaceRoot = Get-Location
    }
    $workspacePath = Join-Path $workspaceRoot ".kiro\settings\mcp.json"
    $locations += @{ Path = $workspacePath; Name = "工作区配置" }
}

$removedCount = 0
$notFoundCount = 0

foreach ($location in $locations) {
    $configPath = $location.Path
    $configName = $location.Name
    
    Write-Host "处理 $configName..." -ForegroundColor Yellow
    Write-Host "  路径：$configPath" -ForegroundColor Gray
    
    if (-not (Test-Path $configPath)) {
        Write-Host "  ⚠️  配置文件不存在" -ForegroundColor Yellow
        $notFoundCount++
        Write-Host ""
        continue
    }
    
    # 读取配置
    try {
        $existingContent = Get-Content $configPath -Raw -Encoding UTF8
        $config = $existingContent | ConvertFrom-Json -AsHashtable
        
        if (-not $config.ContainsKey("mcpServers")) {
            Write-Host "  ⚠️  配置中没有 mcpServers 节点" -ForegroundColor Yellow
            $notFoundCount++
            Write-Host ""
            continue
        }
        
        if (-not $config.mcpServers.ContainsKey("spec-engine")) {
            Write-Host "  ⚠️  配置中没有 spec-engine 服务器" -ForegroundColor Yellow
            $notFoundCount++
            Write-Host ""
            continue
        }
        
        # 移除配置
        $config.mcpServers.Remove("spec-engine")
        
        # 写回文件
        $jsonContent = $config | ConvertTo-Json -Depth 10
        [System.IO.File]::WriteAllText($configPath, $jsonContent, [System.Text.UTF8Encoding]::new($false))
        
        Write-Host "  ✅ 已从配置中移除 spec-engine" -ForegroundColor Green
        $removedCount++
        
    } catch {
        Write-Host "  ❌ 处理失败：$_" -ForegroundColor Red
    }
    
    Write-Host ""
}

# 总结
Write-Host "================================" -ForegroundColor Cyan
Write-Host " 卸载完成" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

if ($removedCount -gt 0) {
    Write-Host "✅ 成功移除 $removedCount 个配置" -ForegroundColor Green
}

if ($notFoundCount -gt 0) {
    Write-Host "⚠️  $notFoundCount 个位置未找到配置" -ForegroundColor Yellow
}

Write-Host ""

if ($removedCount -gt 0) {
    Write-Host "下一步：" -ForegroundColor White
    Write-Host "  1. 重启 Kiro IDE（或使用命令面板中的 'MCP: Reconnect Server'）" -ForegroundColor Gray
    Write-Host "  2. spec-engine 服务器将不再可用" -ForegroundColor Gray
    Write-Host ""
    
    Write-Host "如需重新安装，请运行：" -ForegroundColor White
    Write-Host "  .\install.ps1" -ForegroundColor Gray
    Write-Host ""
}
