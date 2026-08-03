# Spec Engine MCP Installation Script (PowerShell)
# Universal version - Works on Windows and cross-platform PowerShell

param(
    [switch]$Global,
    [switch]$Workspace,
    [string]$WorkspacePath = "",
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# Display help
if ($Help) {
    Write-Host @"
Spec Engine MCP Installation Script

Usage:
  .\install-universal.ps1 [-Global] [-Workspace] [-WorkspacePath <path>] [-Help]

Options:
  -Global          Install to global user config (~/.kiro/settings/mcp.json)
  -Workspace       Install to current workspace (.kiro/settings/mcp.json)
  -WorkspacePath   Specify workspace path (use with -Workspace)
  -Help            Display this help message

Examples:
  .\install-universal.ps1 -Global
  .\install-universal.ps1 -Workspace
  .\install-universal.ps1 -Workspace -WorkspacePath "D:\MyProject"

Default behavior:
  If no option is specified, you will be prompted to choose
"@
    exit 0
}

Write-Host "================================" -ForegroundColor Cyan
Write-Host " Spec Engine MCP Installer" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Build Release version
Write-Host "Step 1/3: Building Release version..." -ForegroundColor Yellow
Write-Host ""

try {
    $buildOutput = cargo build --release 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Build failed!" -ForegroundColor Red
        Write-Host $buildOutput
        exit 1
    }
    Write-Host "[OK] Build succeeded!" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Build failed: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Detect executable name based on platform
$exeName = if ($IsWindows -or $env:OS -eq "Windows_NT") { 
    "spec-engine-mcp.exe" 
} else { 
    "spec-engine-mcp" 
}

# Get executable path
$exePath = Join-Path $PSScriptRoot "target" "release" $exeName
if (-not (Test-Path $exePath)) {
    Write-Host "[ERROR] Executable not found: $exePath" -ForegroundColor Red
    exit 1
}

# Convert to absolute path and use forward slashes (JSON compatible)
$exePathAbsolute = (Resolve-Path $exePath).Path

# Convert to forward slashes for JSON (works on both Windows and Unix)
if ($IsWindows -or $env:OS -eq "Windows_NT") {
    $exePathJson = $exePathAbsolute -replace '\\', '/'
} else {
    $exePathJson = $exePathAbsolute
}

Write-Host "Executable path: $exePathJson" -ForegroundColor Gray
Write-Host ""

# Step 2: Determine installation location
Write-Host "Step 2/3: Determining installation location..." -ForegroundColor Yellow
Write-Host ""

$installLocation = ""

if (-not $Global -and -not $Workspace) {
    # Interactive mode
    Write-Host "Please select installation location:" -ForegroundColor White
    Write-Host "  1. Global user config (~/.kiro/settings/mcp.json)" -ForegroundColor White
    Write-Host "  2. Current workspace (.kiro/settings/mcp.json)" -ForegroundColor White
    Write-Host ""
    
    do {
        $choice = Read-Host "Enter option (1 or 2)"
    } while ($choice -ne "1" -and $choice -ne "2")
    
    if ($choice -eq "1") {
        $Global = $true
    } else {
        $Workspace = $true
    }
}

if ($Global) {
    $homeDir = if ($IsWindows -or $env:OS -eq "Windows_NT") { 
        $env:USERPROFILE 
    } else { 
        $env:HOME 
    }
    $installLocation = Join-Path $homeDir ".kiro" "settings" "mcp.json"
    Write-Host "[*] Installing to global config: $installLocation" -ForegroundColor Cyan
} elseif ($Workspace) {
    if ($WorkspacePath -ne "") {
        $workspaceRoot = $WorkspacePath
    } else {
        $workspaceRoot = Get-Location
    }
    $installLocation = Join-Path $workspaceRoot ".kiro" "settings" "mcp.json"
    Write-Host "[*] Installing to workspace config: $installLocation" -ForegroundColor Cyan
}

Write-Host ""

# Step 3: Write configuration
Write-Host "Step 3/3: Writing MCP configuration..." -ForegroundColor Yellow
Write-Host ""

# Ensure directory exists
$configDir = Split-Path $installLocation -Parent
if (-not (Test-Path $configDir)) {
    Write-Host "Creating directory: $configDir" -ForegroundColor Gray
    New-Item -ItemType Directory -Path $configDir -Force | Out-Null
}

# MCP server configuration
$mcpServerConfig = @{
    command = $exePathJson
    args = @()
    env = @{
        RUST_LOG = "info"
    }
    disabled = $false
}

# Read or create configuration file
$config = @{}
if (Test-Path $installLocation) {
    Write-Host "Reading existing config file..." -ForegroundColor Gray
    try {
        $existingContent = Get-Content $installLocation -Raw -Encoding UTF8
        $config = $existingContent | ConvertFrom-Json -AsHashtable
        
        if (-not $config.ContainsKey("mcpServers")) {
            $config["mcpServers"] = @{}
        }
    } catch {
        Write-Host "[WARN] Existing config file has invalid format, creating new file" -ForegroundColor Yellow
        $config = @{
            mcpServers = @{}
        }
    }
} else {
    Write-Host "Creating new config file..." -ForegroundColor Gray
    $config = @{
        mcpServers = @{}
    }
}

# Check if spec-engine already exists
if ($config.mcpServers.ContainsKey("spec-engine")) {
    Write-Host "[WARN] 'spec-engine' server already exists in config" -ForegroundColor Yellow
    $overwrite = Read-Host "Overwrite existing config? (y/N)"
    if ($overwrite -ne "y" -and $overwrite -ne "Y") {
        Write-Host "[CANCELLED] Installation cancelled" -ForegroundColor Red
        exit 0
    }
}

# Add or update configuration
$config.mcpServers["spec-engine"] = $mcpServerConfig

# Write to file
try {
    $jsonContent = $config | ConvertTo-Json -Depth 10
    # Ensure UTF-8 without BOM
    [System.IO.File]::WriteAllText($installLocation, $jsonContent, [System.Text.UTF8Encoding]::new($false))
    Write-Host "[OK] Configuration written: $installLocation" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Failed to write config file: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Success
Write-Host "================================" -ForegroundColor Green
Write-Host " Installation Successful!" -ForegroundColor Green
Write-Host "================================" -ForegroundColor Green
Write-Host ""

Write-Host "Configuration details:" -ForegroundColor White
Write-Host "  Server name: spec-engine" -ForegroundColor Gray
Write-Host "  Executable: $exePathJson" -ForegroundColor Gray
Write-Host "  Config file: $installLocation" -ForegroundColor Gray
Write-Host ""

Write-Host "Next steps:" -ForegroundColor White
Write-Host "  1. Restart Kiro IDE (or use 'MCP: Reconnect Server' from command palette)" -ForegroundColor Gray
Write-Host "  2. MCP server will be automatically loaded" -ForegroundColor Gray
Write-Host "  3. AI assistant can now use these tools:" -ForegroundColor Gray
Write-Host "     - parse_frame: Parse power protocol frames" -ForegroundColor Gray
Write-Host "     - lookup_di: Query DI definitions" -ForegroundColor Gray
Write-Host "     - search_di: Search DI" -ForegroundColor Gray
Write-Host "     - list_protocols: List supported protocols" -ForegroundColor Gray
Write-Host ""

Write-Host "Test command:" -ForegroundColor White
Write-Host "  echo '{`"jsonrpc`":`"2.0`",`"id`":1,`"method`":`"initialize`",`"params`":{}}' | & '$exePathJson'" -ForegroundColor Gray
Write-Host ""

Write-Host "To uninstall, run:" -ForegroundColor White
Write-Host "  .\uninstall.ps1" -ForegroundColor Gray
Write-Host ""
