[CmdletBinding()]
param(
    [string]$RepositoryRoot = (Split-Path -Parent $PSScriptRoot)
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$RepositoryRoot = (Resolve-Path -LiteralPath $RepositoryRoot).Path
$failures = [System.Collections.Generic.List[string]]::new()

function Get-Text {
    param([string]$RelativePath)

    $path = Join-Path $RepositoryRoot $RelativePath
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        $failures.Add("missing required file: $RelativePath")
        return $null
    }
    return [System.IO.File]::ReadAllText($path)
}

function Get-TomlSection {
    param(
        [string]$Text,
        [string]$Name
    )

    $escapedName = [regex]::Escape($Name)
    $match = [regex]::Match($Text, "(?ms)^\[$escapedName\]\r?\n(?:(?!^\[).)*")
    if (-not $match.Success) {
        return $null
    }
    return $match.Value
}

function Require-Pattern {
    param(
        [string]$Label,
        [string]$Text,
        [string]$Pattern
    )

    if ($null -eq $Text -or -not [regex]::IsMatch($Text, $Pattern)) {
        $failures.Add("$Label is missing: $Pattern")
    }
}

$cargo = Get-Text 'Cargo.toml'
$config = Get-Text '.cargo\config.toml'
$release = Get-Text 'release.ps1'
$profiling = Get-Text 'scripts\profile-card-game.ps1'
$uiSmoke = Get-Text 'crates\card_game_bin\tests\ui-smoke.ps1'
$uiWorkflow = Get-Text '.github\workflows\ui-smoke.yml'
$qualityWorkflow = Get-Text '.github\workflows\quality.yml'

if ($null -ne $cargo) {
    $profiles = @{
        'profile.dev' = @('incremental\s*=\s*false', 'debug\s*=\s*1')
        'profile.dev.package."*"' = @('incremental\s*=\s*false', 'debug\s*=\s*false')
        'profile.release' = @('incremental\s*=\s*false', 'debug\s*=\s*false', 'strip\s*=\s*"symbols"')
    }
    foreach ($profile in $profiles.GetEnumerator()) {
        $section = Get-TomlSection -Text $cargo -Name $profile.Key
        if ($null -eq $section) {
            $failures.Add("missing Cargo profile section: $($profile.Key)")
            continue
        }
        foreach ($pattern in $profile.Value) {
            Require-Pattern -Label "$($profile.Key) policy" -Text $section -Pattern "(?m)^\s*$pattern"
        }
    }
    foreach ($diagnosticProfile in @('profile.release-debuggable', 'profile.profiling', 'profile.bench')) {
        if ($null -eq (Get-TomlSection -Text $cargo -Name $diagnosticProfile)) {
            $failures.Add("missing opt-in Cargo profile: $diagnosticProfile")
        }
    }
}

Require-Pattern -Label 'Cargo target root' -Text $config -Pattern '(?m)^\s*target-dir\s*=\s*"target"'
Require-Pattern -Label 'ordinary release command' -Text $release -Pattern 'cargo\s+build\s+--release'
Require-Pattern -Label 'release sidecar guard' -Text $release -Pattern 'debugSidecars'
Require-Pattern -Label 'profiling command' -Text $profiling -Pattern 'cargo\.exe\s+build\s+--profile\s+profiling'
Require-Pattern -Label 'UI smoke CI root' -Text $uiWorkflow -Pattern 'target\\ui-smoke-ci'
Require-Pattern -Label 'UI smoke artifact retention' -Text $uiWorkflow -Pattern '(?m)^\s*retention-days:\s*14\s*$'
Require-Pattern -Label 'coverage artifact retention' -Text $qualityWorkflow -Pattern '(?m)^\s*retention-days:\s*14\s*$'

$artifactRoot = Join-Path $RepositoryRoot 'target\ui-smoke-ci'
$successfulRuns = 0
if (Test-Path -LiteralPath $artifactRoot -PathType Container) {
    foreach ($run in @(Get-ChildItem -LiteralPath $artifactRoot -Directory)) {
        $markers = @(Get-ChildItem -LiteralPath $run.FullName -Recurse -Filter 'runner.terminal.txt' -File -ErrorAction SilentlyContinue)
        $passed = $markers.Count -gt 0
        foreach ($marker in $markers) {
            if ([System.IO.File]::ReadAllText($marker.FullName) -notmatch '(?m)^outcome=passed\r?$') {
                $passed = $false
            }
        }
        if ($passed) {
            $successfulRuns++
        }
    }
}
if ($successfulRuns -gt 3) {
    $failures.Add("UI smoke retention exceeded: $successfulRuns successful run roots found under target\\ui-smoke-ci; maximum is 3")
}

if ($failures.Count -gt 0) {
    Write-Error ("Build artifact policy failed:`n - " + ($failures -join "`n - "))
    exit 1
}

Write-Output "Build artifact policy passed: profiles, target root, release/profile entry points, and artifact retention boundaries are consistent."
Write-Output "Successful UI smoke run roots: $successfulRuns/3"
