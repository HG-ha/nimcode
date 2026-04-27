$ErrorActionPreference = "Stop"
$repo = "HG-ha/nimcode"
$binary = "nimcode.exe"
$installDir = "$env:USERPROFILE\.nimcode\bin"

Write-Host "NimCode installer" -ForegroundColor Cyan
Write-Host ""

# Detect version
if ($args.Count -gt 0) {
    $version = $args[0]
} else {
    $release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
    $version = $release.tag_name -replace '^v', ''
}

if (-not $version) {
    Write-Error "Could not detect latest version. Pass a version as argument."
    exit 1
}

# Check installed version
$installedVersion = ""
$existingBinary = Join-Path $installDir $binary
if (Test-Path $existingBinary) {
    try {
        $versionOutput = & $existingBinary --version 2>&1
        if ($versionOutput -match '(\d+\.\d+\.\d+)') {
            $installedVersion = $Matches[1]
        }
    } catch {}
}

if ($installedVersion -and ($installedVersion -eq $version)) {
    Write-Host "  nimcode v$version is already installed and up to date." -ForegroundColor Green
    Write-Host "  To force reinstall, run: .\install.ps1 $version"
    Write-Host ""
    exit 0
}

if ($installedVersion) {
    $action = "Upgrading"
    Write-Host "  Installed : v$installedVersion"
} else {
    $action = "Installing"
}

$target = "x86_64-pc-windows-msvc"
$url = "https://github.com/$repo/releases/download/v$version/nimcode-$target.zip"

Write-Host "  $action  : v$version"
Write-Host "  Target    : $target"
Write-Host "  Location  : $installDir\$binary"
Write-Host ""

# Download
$tmpDir = Join-Path $env:TEMP "nimcode-install-$(Get-Random)"
New-Item -ItemType Directory -Force $tmpDir | Out-Null
$zipPath = Join-Path $tmpDir "nimcode.zip"

Write-Host "Downloading $url..."
Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing

Write-Host "Extracting..."
Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

# Install
New-Item -ItemType Directory -Force $installDir | Out-Null
Copy-Item (Join-Path $tmpDir $binary) (Join-Path $installDir $binary) -Force

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    Write-Host "  Added $installDir to user PATH (restart terminal to take effect)"
}

# Cleanup
Remove-Item -Recurse -Force $tmpDir

Write-Host ""
if ($installedVersion) {
    Write-Host "nimcode upgraded from v$installedVersion to v$version" -ForegroundColor Green
} else {
    Write-Host "nimcode v$version installed to $installDir\$binary" -ForegroundColor Green
    Write-Host ""
    Write-Host "Run 'nimcode' to get started. It will prompt for your NVIDIA NIM API Key on first launch."
}
