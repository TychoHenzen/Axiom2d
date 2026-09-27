# Build release binary and compress with UPX
param(
    [string]$Package = "card_game_bin",
    [switch]$SkipUpx
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path $PSScriptRoot).Path
$releaseDirectory = Join-Path $repoRoot "target\release"
$exe = Join-Path $releaseDirectory "$Package.exe"

Push-Location $repoRoot
try {
    Write-Host "Building $Package (profile=release, target=$exe)..." -ForegroundColor Cyan
    cargo build --release -p $Package
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $debugSidecars = @(Get-ChildItem -LiteralPath $releaseDirectory -Filter "$Package*.pdb" -File -ErrorAction SilentlyContinue)
    foreach ($sidecar in $debugSidecars) {
        Remove-Item -LiteralPath $sidecar.FullName -Force
    }
    $remainingDebugSidecars = @(Get-ChildItem -LiteralPath $releaseDirectory -Filter "$Package*.pdb" -File -ErrorAction SilentlyContinue)
    if ($remainingDebugSidecars.Count -gt 0) {
        throw "Size-focused release retained unexpected debug sidecar(s): $($remainingDebugSidecars.Name -join ', ')"
    }

    $size = (Get-Item $exe).Length
    Write-Host "Build: $([math]::Round($size / 1MB, 2)) MB; debug sidecars: none" -ForegroundColor Green

    if (-not $SkipUpx) {
        Write-Host "Packing with UPX..." -ForegroundColor Cyan
        & "$PSScriptRoot\upx.exe" --best --force $exe
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

        $packed = (Get-Item $exe).Length
        Write-Host "Packed: $([math]::Round($packed / 1MB, 2)) MB ($([math]::Round($packed / $size * 100, 1))%)" -ForegroundColor Green
    }
}
finally {
    Pop-Location
}
