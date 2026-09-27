[CmdletBinding(SupportsShouldProcess)]
param(
    [Parameter(Mandatory)]
    [string]$Root,
    [ValidateRange(1, 100)]
    [int]$KeepSuccessful = 3,
    [string]$CurrentRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$rootPath = (Resolve-Path -LiteralPath $Root).Path
$rootName = Split-Path -Leaf $rootPath
$parentName = Split-Path -Leaf (Split-Path -Parent $rootPath)
if ($rootName -ne 'ui-smoke-ci' -or $parentName -ne 'target') {
    throw "Refusing retention outside target\\ui-smoke-ci: $rootPath"
}

$currentPath = $null
if (-not [string]::IsNullOrWhiteSpace($CurrentRun)) {
    $currentPath = [System.IO.Path]::GetFullPath($CurrentRun)
    if (-not $currentPath.StartsWith($rootPath + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Current run is outside retention root: $currentPath"
    }
}

function Test-SuccessfulRun {
    param([string]$Path)

    $markers = @(Get-ChildItem -LiteralPath $Path -Recurse -Filter 'runner.terminal.txt' -File -ErrorAction SilentlyContinue)
    if ($markers.Count -eq 0) {
        return $false
    }
    foreach ($marker in $markers) {
        if ([System.IO.File]::ReadAllText($marker.FullName) -notmatch '(?m)^outcome=passed\r?$') {
            return $false
        }
    }
    return $true
}

$runs = @(Get-ChildItem -LiteralPath $rootPath -Directory)
$successfulRuns = @($runs | Where-Object { Test-SuccessfulRun -Path $_.FullName })
$orderedSuccessful = @($successfulRuns | Sort-Object LastWriteTime -Descending)
$keepPaths = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
if ($null -ne $currentPath -and $orderedSuccessful.FullName -contains $currentPath) {
    $keepPaths.Add($currentPath) | Out-Null
}
foreach ($run in $orderedSuccessful) {
    if ($keepPaths.Count -ge $KeepSuccessful) {
        break
    }
    $keepPaths.Add($run.FullName) | Out-Null
}

$removed = 0
foreach ($run in $orderedSuccessful) {
    if ($keepPaths.Contains($run.FullName)) {
        continue
    }
    if ($PSCmdlet.ShouldProcess($run.FullName, 'remove successful generated artifact root')) {
        Remove-Item -LiteralPath $run.FullName -Recurse -Force
        $removed++
    }
}

Write-Output ("Retained {0} successful UI smoke run root(s); removed {1}; failure and unknown roots were untouched." -f $keepPaths.Count, $removed)
