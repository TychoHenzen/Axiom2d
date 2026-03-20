# Build release binary and compress with UPX
param(
    [string]$Package = "card_game_bin",
    [switch]$SkipUpx
)

$ErrorActionPreference = "Stop"
$exe = "target\release\$Package.exe"

Write-Host "Building $Package (release)..." -ForegroundColor Cyan
cargo build --release -p $Package
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$size = (Get-Item $exe).Length
Write-Host "Build: $([math]::Round($size / 1MB, 2)) MB" -ForegroundColor Green

if (-not $SkipUpx) {
    Write-Host "Packing with UPX..." -ForegroundColor Cyan
    & "$PSScriptRoot\upx.exe" --best --force $exe
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $packed = (Get-Item $exe).Length
    Write-Host "Packed: $([math]::Round($packed / 1MB, 2)) MB ($([math]::Round($packed / $size * 100, 1))%)" -ForegroundColor Green
}
