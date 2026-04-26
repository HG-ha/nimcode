$ErrorActionPreference = "Stop"
$repo = "HG-ha/nimcode"
$binary = "nimcode.exe"

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

$target = "x86_64-pc-windows-msvc"
$url = "https://github.com/$repo/releases/download/v$version/nimcode-$target.zip"
$installDir = "$env:USERPROFILE\.nimcode\bin"

Write-Host "  Version : v$version"
Write-Host "  Target  : $target"
Write-Host "  Install : $installDir\$binary"
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
Write-Host "nimcode v$version installed to $installDir\$binary" -ForegroundColor Green
Write-Host ""
Write-Host "Run 'nimcode' to get started. It will prompt for your NVIDIA NIM API Key on first launch."
