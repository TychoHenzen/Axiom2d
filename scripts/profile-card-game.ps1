param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$GameArgs
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$exePath = Join-Path $repoRoot "target\\profiling\\card_game_bin.exe"
$samply = Get-Command "samply.exe" -ErrorAction SilentlyContinue
if ($null -eq $samply) {
    $samply = Get-Command "samply" -ErrorAction SilentlyContinue
}

if ($null -eq $samply) {
    throw "samply was not found in PATH. Install it on Windows first."
}

if ($null -eq (Get-Command "cargo.exe" -ErrorAction SilentlyContinue)) {
    throw "cargo.exe was not found in PATH."
}

Push-Location $repoRoot
try {
    & cargo.exe build --profile profiling -p card_game_bin
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    if (-not (Test-Path $exePath)) {
        throw "Expected profiled binary at '$exePath' after build."
    }

    & $samply.Source record `
        --windows-symbol-server https://msdl.microsoft.com/download/symbols `
        $exePath `
        @GameArgs
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
