param(
    [switch]$RunnerChild,
    [string]$RunnerArtifactDirectory,
    [switch]$Interaction,
    [switch]$HandRoundTrip,
    [switch]$ZoneTransition,
    [switch]$ReaderRoundTrip,
    [switch]$CombinerProcessing,
    [switch]$CableWrapping,
    [switch]$StashRoundTrip,
    [switch]$BoosterOpening,
    [switch]$IdentitySignature,
    [switch]$ArtFace
)

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$artifactDir = if ($RunnerChild) {
    $RunnerArtifactDirectory
}
else {
    Join-Path $repoRoot (Join-Path 'target' ("ui-smoke-{0}" -f [guid]::NewGuid().ToString('N')))
}

if (-not $RunnerChild) {
    $scenarioFlags = @(
        $Interaction,
        $HandRoundTrip,
        $ZoneTransition,
        $ReaderRoundTrip,
        $CombinerProcessing,
        $CableWrapping,
        $StashRoundTrip,
        $BoosterOpening,
        $IdentitySignature,
        $ArtFace
    )
    if (@($scenarioFlags | Where-Object { $_ }).Count -gt 1) {
        throw 'Interaction, HandRoundTrip, ZoneTransition, ReaderRoundTrip, CombinerProcessing, CableWrapping, StashRoundTrip, BoosterOpening, IdentitySignature, and ArtFace are mutually exclusive'
    }
    New-Item -ItemType Directory -Path $artifactDir -Force | Out-Null
    $runnerStdoutPath = Join-Path $artifactDir 'runner.stdout.log'
    $runnerStderrPath = Join-Path $artifactDir 'runner.stderr.log'
    $runnerExitCodePath = Join-Path $artifactDir 'runner.exitcode.txt'
    $utf8WithoutBom = [System.Text.UTF8Encoding]::new($false)
    [System.IO.File]::WriteAllText($runnerStdoutPath, '', $utf8WithoutBom)
    [System.IO.File]::WriteAllText($runnerStderrPath, '', $utf8WithoutBom)
    [System.IO.File]::WriteAllText($runnerExitCodePath, 'running', $utf8WithoutBom)

    $runnerExitCode = 1
    try {
        $runnerArguments = @(
            '-NoProfile',
            '-File',
            "`"$PSCommandPath`"",
            '-RunnerChild',
            '-RunnerArtifactDirectory',
            "`"$artifactDir`""
        )
        if ($Interaction) {
            $runnerArguments += '-Interaction'
        }
        if ($HandRoundTrip) {
            $runnerArguments += '-HandRoundTrip'
        }
        if ($ZoneTransition) {
            $runnerArguments += '-ZoneTransition'
        }
        if ($ReaderRoundTrip) {
            $runnerArguments += '-ReaderRoundTrip'
        }
        if ($CombinerProcessing) {
            $runnerArguments += '-CombinerProcessing'
        }
        if ($CableWrapping) {
            $runnerArguments += '-CableWrapping'
        }
        if ($StashRoundTrip) {
            $runnerArguments += '-StashRoundTrip'
        }
        if ($BoosterOpening) {
            $runnerArguments += '-BoosterOpening'
        }
        if ($IdentitySignature) {
            $runnerArguments += '-IdentitySignature'
        }
        if ($ArtFace) {
            $runnerArguments += '-ArtFace'
        }
        $runnerProcess = Start-Process -FilePath (Get-Process -Id $PID).Path `
            -ArgumentList $runnerArguments `
            -WorkingDirectory $repoRoot `
            -NoNewWindow `
            -PassThru `
            -Wait `
            -RedirectStandardOutput $runnerStdoutPath `
            -RedirectStandardError $runnerStderrPath
        $runnerProcess.Refresh()
        $runnerExitCode = $runnerProcess.ExitCode
    }
    catch {
        $message = "runner launch failed: $($_.Exception.Message)$([Environment]::NewLine)"
        [System.IO.File]::AppendAllText($runnerStderrPath, $message, $utf8WithoutBom)
    }

    [System.IO.File]::WriteAllText($runnerExitCodePath, "$runnerExitCode`r`n", $utf8WithoutBom)
    [Console]::Out.Write([System.IO.File]::ReadAllText($runnerStdoutPath))
    [Console]::Error.Write([System.IO.File]::ReadAllText($runnerStderrPath))
    exit $runnerExitCode
}

if ([string]::IsNullOrWhiteSpace($artifactDir)) {
    throw 'RunnerArtifactDirectory is required for an inner run'
}

$stateFile = Join-Path $artifactDir 'state.txt'
$inputFile = Join-Path $artifactDir 'input.txt'
$stdoutPath = Join-Path $artifactDir 'app.stdout.log'
$stderrPath = Join-Path $artifactDir 'app.stderr.log'
$baselineFramePath = Join-Path $artifactDir 'baseline.bmp'
$framePath = Join-Path $artifactDir 'frame.bmp'
$releasedFramePath = Join-Path $artifactDir 'released.bmp'
$flippedFramePath = Join-Path $artifactDir 'flipped.bmp'
$handFramePath = Join-Path $artifactDir 'hand.bmp'
$returnedFramePath = Join-Path $artifactDir 'returned.bmp'
$zoneHolderFramePath = Join-Path $artifactDir 'zone-holder.bmp'
$zoneReturnedFramePath = Join-Path $artifactDir 'zone-returned.bmp'
$readerInsertedFramePath = Join-Path $artifactDir 'reader-inserted.bmp'
$readerEjectedFramePath = Join-Path $artifactDir 'reader-ejected.bmp'
$readerReturnedFramePath = Join-Path $artifactDir 'reader-returned.bmp'
$combinerFramePath = Join-Path $artifactDir 'combiner.bmp'
$cableWrappingFramePath = Join-Path $artifactDir 'cable-wrapping.bmp'
$holderStatePath = if ($ZoneTransition) {
    Join-Path $artifactDir 'zone-holder-state.txt'
}
else {
    Join-Path $artifactDir 'hand-state.txt'
}
$returnedStatePath = if ($ZoneTransition) {
    Join-Path $artifactDir 'zone-returned-state.txt'
}
elseif ($ReaderRoundTrip) {
    Join-Path $artifactDir 'reader-returned-state.txt'
}
else {
    Join-Path $artifactDir 'returned-state.txt'
}
$holderFramePath = if ($ZoneTransition) { $zoneHolderFramePath } else { $handFramePath }
$returnedHolderFramePath = if ($ZoneTransition) { $zoneReturnedFramePath } else { $returnedFramePath }
$stashOpenFramePath = Join-Path $artifactDir 'stash-open.bmp'
$stashStoredFramePath = Join-Path $artifactDir 'stash-stored.bmp'
$stashPageTwoFramePath = Join-Path $artifactDir 'stash-page-2.bmp'
$stashRetrievedFramePath = Join-Path $artifactDir 'stash-retrieved.bmp'
$boosterOpeningFramePath = Join-Path $artifactDir 'booster-opening.bmp'
$boosterOpenedFramePath = Join-Path $artifactDir 'booster-opened.bmp'
$identityFramePath = Join-Path $artifactDir 'identity.bmp'
$artFaceFramePath = Join-Path $artifactDir 'art-face.bmp'
$failureFramePath = Join-Path $artifactDir 'failure.bmp'
$frameCaptureRequestFile = Join-Path $artifactDir 'frame-capture.request'
$scenarioName = if ($Interaction) {
    'seeded-card-interaction'
}
elseif ($ZoneTransition) {
    'seeded-card-zone-transition'
}
elseif ($ReaderRoundTrip) {
    'seeded-card-reader-roundtrip'
}
elseif ($CombinerProcessing) {
    'seeded-card-combiner'
}
elseif ($CableWrapping) {
    'seeded-card-cable-wrapping'
}
elseif ($HandRoundTrip) {
    'seeded-card-hand-roundtrip'
}
elseif ($StashRoundTrip) {
    'seeded-card-stash-roundtrip'
}
elseif ($BoosterOpening) {
    'seeded-booster-opening'
}
elseif ($IdentitySignature) {
    'seeded-card-identity'
}
elseif ($ArtFace) {
    'seeded-card-art-face'
}
else {
    'seeded-card-drag'
}
$process = $null
$windowHandle = [IntPtr]::Zero
$foregroundBeforeLaunch = $null
$previousCursor = $null
$previousDpiContext = [IntPtr]::Zero
$mouseDown = $false
$rightMouseDown = $false
$leftPressObserved = $false
$succeeded = $false
$cleanupFailure = $null
$failureCaptureError = $null
$stage = 'setup'
$identityNoInput = 'not_applicable_identity_no_input'
$foregroundBeforePostMessage = [pscustomobject]@{
    Handle = $identityNoInput
    ProcessId = $identityNoInput
}
$cursorBeforePostMessage = [pscustomobject]@{
    X = $identityNoInput
    Y = $identityNoInput
}
$lastInputTickBeforePostMessage = $identityNoInput
$lastInputTickAfterReleaseAck = $identityNoInput

New-Item -ItemType Directory -Path $artifactDir -Force | Out-Null

function Read-UiState {
    param([string]$Path)

    $file = $null
    try {
        $file = [System.IO.File]::Open(
            $Path,
            [System.IO.FileMode]::Open,
            [System.IO.FileAccess]::Read,
            [System.IO.FileShare]::ReadWrite)
        $contents = [System.IO.StreamReader]::new($file).ReadToEnd()
        $state = @{}
        foreach ($line in $contents -split '\r?\n') {
            $separator = $line.IndexOf('=')
            if ($separator -gt 0) {
                $state[$line.Substring(0, $separator)] = $line.Substring($separator + 1)
            }
        }
        if ($state.Count -lt 8) { return $null }
        if ($script:mouseDown -and $state['left_pressed'] -eq 'true') {
            $script:leftPressObserved = $true
        }
        return ,$state
    }
    catch {
        return $null
    }
    finally {
        if ($null -ne $file) { $file.Dispose() }
    }
}

function Test-Position {
    param(
        [hashtable]$State,
        [string]$XKey = 'rendered_x',
        [string]$YKey = 'rendered_y',
        [double]$ExpectedX,
        [double]$ExpectedY,
        [double]$Tolerance
    )

    try {
        $actualX = [double]::Parse($State[$XKey], [Globalization.CultureInfo]::InvariantCulture)
        $actualY = [double]::Parse($State[$YKey], [Globalization.CultureInfo]::InvariantCulture)
        return [Math]::Abs($actualX - $ExpectedX) -le $Tolerance -and
            [Math]::Abs($actualY - $ExpectedY) -le $Tolerance
    }
    catch {
        return $false
    }
}

function Test-ExpectedRotation {
    param(
        [hashtable]$State,
        [string]$Key,
        [double]$Expected,
        [double]$Tolerance
    )

    try {
        $actual = [double]::Parse($State[$Key], [Globalization.CultureInfo]::InvariantCulture)
        return [Math]::Sign($actual) -eq [Math]::Sign($Expected) -and
            [Math]::Abs($actual - $Expected) -le $Tolerance
    }
    catch {
        return $false
    }
}

function Format-UiStateDiagnostic {
    param([hashtable]$State)

    if ($null -eq $State) {
        return 'none (no complete state snapshot)'
    }
    return ("scenario={0}, dragging={1}, booster_dragging={2}, zone={3}, hand_contains={4}, hand_count={5}, stash_visible={6}, stash_page={7}, stash_slot={8}, stash_present={9}, stash_origin={10}, stash_follow={11}, booster_present={12}, booster_phase={13}, booster_cards={14}, opened_card={15}, opened_zone={16}, opened_seed={17}, rendered=({18},{19}), rotation={20}, face_up={21}, mouse=({22},{23}), left_pressed={24}, right_pressed={25}, holder={26}, holder_occupied={27}, zone_config=({28},{29},{30})" -f `
        $State['scenario'], $State['dragging'], $State['booster_dragging'], $State['zone'], $State['hand_contains'], `
        $State['hand_count'], $State['stash_visible'], $State['stash_page'], $State['stash_slot'], `
        $State['stash_slot_present'], $State['stash_origin'], $State['stash_cursor_follow'], `
        $State['booster_pack_present'], $State['booster_phase'], $State['booster_card_count'], `
        $State['opened_card_present'], $State['opened_card_zone'], $State['opened_card_seed'], `
        $State['rendered_x'], $State['rendered_y'], $State['rotation'], $State['face_up'], `
        $State['mouse_x'], $State['mouse_y'], $State['left_pressed'], $State['right_pressed'],
        $State['holder'], $State['holder_occupied'], $State['zone_has_physics'],
        $State['zone_render_layer'], $State['zone_has_item_form'])
}

function Wait-UiState {
    param(
        [System.Diagnostics.Process]$Process,
        [string]$StateFile,
        [string]$Scenario,
        [string]$Stage,
        [int]$TimeoutSeconds,
        [string]$ExpectedState,
        [scriptblock]$Predicate
    )

    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    $lastState = $null
    while ($watch.Elapsed.TotalSeconds -lt $TimeoutSeconds) {
        $Process.Refresh()
        if ($Process.HasExited) {
            $observedState = Read-UiState -Path $StateFile
            if ($null -ne $observedState) { $lastState = $observedState }
            $lastObserved = Format-UiStateDiagnostic -State $lastState
            throw ("scenario={0} stage={1}: game process exited with code {2}; expected=[{3}]; last_observed=[{4}]" -f `
                $Scenario, $Stage, $Process.ExitCode, $ExpectedState, $lastObserved)
        }
        $observedState = Read-UiState -Path $StateFile
        if ($null -ne $observedState) {
            $lastState = $observedState
            if (& $Predicate $observedState) {
                return $observedState
            }
        }
        Start-Sleep -Milliseconds 200
    }

    $lastObserved = Format-UiStateDiagnostic -State $lastState
    throw ("scenario={0} stage={1}: timed out after {2}s; expected=[{3}]; last_observed=[{4}]" -f `
        $Scenario, $Stage, $TimeoutSeconds, $ExpectedState, $lastObserved)
}

function Request-GameFrameCapture {
    param(
        [System.Diagnostics.Process]$Process,
        [string]$RequestFile,
        [string]$CapturePath,
        [string]$Stage,
        [int]$TimeoutSeconds = 10
    )

    $resultFile = [System.IO.Path]::ChangeExtension($RequestFile, 'result')
    foreach ($stalePath in @($RequestFile, $resultFile)) {
        if (Test-Path -LiteralPath $stalePath) {
            Remove-Item -LiteralPath $stalePath -Force
        }
    }
    $temporaryRequestFile = "$RequestFile.$PID.tmp"
    [System.IO.File]::WriteAllText(
        $temporaryRequestFile,
        $CapturePath,
        [System.Text.UTF8Encoding]::new($false))
    Move-Item -LiteralPath $temporaryRequestFile -Destination $RequestFile -Force

    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    while ($watch.Elapsed.TotalSeconds -lt $TimeoutSeconds) {
        if (Test-Path -LiteralPath $resultFile) {
            $result = [System.IO.File]::ReadAllText($resultFile).Trim()
            if ($result.StartsWith('error=', [StringComparison]::OrdinalIgnoreCase)) {
                throw "stage=${Stage}: frame capture failed: $($result.Substring(6))"
            }
            if ($result -ne 'ok') {
                throw "stage=${Stage}: invalid frame capture response '$result'"
            }
            if (-not (Test-Path -LiteralPath $CapturePath) -or
                (Get-Item -LiteralPath $CapturePath).Length -eq 0) {
                throw "stage=${Stage}: frame capture response was ok but the BMP is missing or empty"
            }
            return
        }

        $Process.Refresh()
        if ($Process.HasExited) {
            throw "stage=${Stage}: game process exited before returning frame capture status (exit $($Process.ExitCode))"
        }
        Start-Sleep -Milliseconds 100
    }
    throw "stage=${Stage}: timed out after ${TimeoutSeconds}s waiting for game-frame capture"
}

function Read-UiSmokeBitmap {
    param([string]$Path)

    $bytes = [System.IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -lt 54 -or [System.Text.Encoding]::ASCII.GetString($bytes, 0, 2) -ne 'BM') {
        throw "invalid BMP header: $Path"
    }
    $pixelOffset = [int][BitConverter]::ToUInt32($bytes, 10)
    $dibSize = [BitConverter]::ToInt32($bytes, 14)
    $width = [BitConverter]::ToInt32($bytes, 18)
    $signedHeight = [BitConverter]::ToInt32($bytes, 22)
    $planes = [BitConverter]::ToInt16($bytes, 26)
    $bitsPerPixel = [BitConverter]::ToInt16($bytes, 28)
    $compression = [BitConverter]::ToInt32($bytes, 30)
    if ($dibSize -lt 40 -or $width -le 0 -or $signedHeight -eq 0 -or
        $planes -ne 1 -or $bitsPerPixel -ne 32 -or $compression -ne 0 -or $pixelOffset -lt 54) {
        throw "unsupported BMP layout: $Path"
    }

    $height = [int][Math]::Abs([long]$signedHeight)
    $rowStride = [long]$width * 4
    $requiredLength = [long]$pixelOffset + $rowStride * $height
    if ($requiredLength -gt $bytes.LongLength) {
        throw "BMP pixel data is truncated: $Path"
    }
    return @{
        Bytes = $bytes
        Width = $width
        Height = $height
        RowStride = [int]$rowStride
        PixelOffset = $pixelOffset
        TopDown = $signedHeight -lt 0
    }
}

function Get-UiSmokeChangedPixels {
    param(
        [hashtable]$Before,
        [hashtable]$After,
        [int]$CenterX,
        [int]$CenterY,
        [int]$HalfWidth = 55,
        [int]$HalfHeight = 70,
        [int]$RgbDeltaThreshold = 24
    )

    if ($Before.Width -ne $After.Width -or $Before.Height -ne $After.Height) {
        throw 'frame dimensions differ'
    }
    $left = [Math]::Max(0, $CenterX - $HalfWidth)
    $right = [Math]::Min($Before.Width - 1, $CenterX + $HalfWidth)
    $top = [Math]::Max(0, $CenterY - $HalfHeight)
    $bottom = [Math]::Min($Before.Height - 1, $CenterY + $HalfHeight)
    $changedPixels = 0
    for ($y = $top; $y -le $bottom; $y++) {
        $beforeY = if ($Before.TopDown) { $y } else { $Before.Height - 1 - $y }
        $afterY = if ($After.TopDown) { $y } else { $After.Height - 1 - $y }
        $beforeRow = $Before.PixelOffset + $beforeY * $Before.RowStride
        $afterRow = $After.PixelOffset + $afterY * $After.RowStride
        for ($x = $left; $x -le $right; $x++) {
            $beforePixel = $beforeRow + $x * 4
            $afterPixel = $afterRow + $x * 4
            $rgbDelta = [Math]::Abs([int]$Before.Bytes[$beforePixel] - [int]$After.Bytes[$afterPixel]) +
                [Math]::Abs([int]$Before.Bytes[$beforePixel + 1] - [int]$After.Bytes[$afterPixel + 1]) +
                [Math]::Abs([int]$Before.Bytes[$beforePixel + 2] - [int]$After.Bytes[$afterPixel + 2])
            if ($rgbDelta -ge $RgbDeltaThreshold) {
                $changedPixels++
            }
        }
    }
    return @{
        Left = $left
        Top = $top
        Width = $right - $left + 1
        Height = $bottom - $top + 1
        ChangedPixels = $changedPixels
    }
}

function Get-UiSmokeForegroundSnapshot {
    $window = [AxiomUiSmokeNative]::GetForegroundWindow()
    return [pscustomobject]@{
        Handle = $window
        Title = [AxiomUiSmokeNative]::GetWindowTitle($window)
        ProcessId = [AxiomUiSmokeNative]::GetWindowProcessId($window)
    }
}

function New-UiSmokeCardTemplate {
    param(
        [hashtable]$Frame,
        [int]$CenterX,
        [int]$CenterY,
        [int]$Width = 20,
        [int]$Height = 24
    )

    $left = $CenterX - [int][Math]::Floor($Width / 2.0)
    $top = $CenterY - [int][Math]::Floor($Height / 2.0)
    if ($left -lt 0 -or $top -lt 0 -or $left + $Width -gt $Frame.Width -or $top + $Height -gt $Frame.Height) {
        throw 'baseline card template would exceed the captured frame'
    }
    $pixels = [byte[]]::new($Width * $Height * 3)
    $colors = [System.Collections.Generic.HashSet[int]]::new()
    for ($y = 0; $y -lt $Height; $y++) {
        $frameY = if ($Frame.TopDown) { $top + $y } else { $Frame.Height - 1 - ($top + $y) }
        $frameRow = $Frame.PixelOffset + $frameY * $Frame.RowStride
        for ($x = 0; $x -lt $Width; $x++) {
            $framePixel = $frameRow + ($left + $x) * 4
            $templatePixel = ($y * $Width + $x) * 3
            $red = $Frame.Bytes[$framePixel + 2]
            $green = $Frame.Bytes[$framePixel + 1]
            $blue = $Frame.Bytes[$framePixel]
            $pixels[$templatePixel] = $red
            $pixels[$templatePixel + 1] = $green
            $pixels[$templatePixel + 2] = $blue
            [void]$colors.Add((([int]$red -shl 16) -bor ([int]$green -shl 8) -bor [int]$blue))
        }
    }
    if ($colors.Count -lt 16) {
        throw "baseline card template is not distinctive enough ($($colors.Count) RGB colors)"
    }
    return @{
        Pixels = $pixels
        Width = $Width
        Height = $Height
        PixelCount = $Width * $Height
        Left = $left
        Top = $top
        UniqueColors = $colors.Count
    }
}

function Get-UiSmokeTemplateMatchCount {
    param(
        [hashtable]$Frame,
        [hashtable]$Template,
        [int]$Left,
        [int]$Top,
        [int]$RgbDeltaMaximum
    )

    $matches = 0
    for ($y = 0; $y -lt $Template.Height; $y++) {
        $frameY = if ($Frame.TopDown) { $Top + $y } else { $Frame.Height - 1 - ($Top + $y) }
        $frameRow = $Frame.PixelOffset + $frameY * $Frame.RowStride
        for ($x = 0; $x -lt $Template.Width; $x++) {
            $framePixel = $frameRow + ($Left + $x) * 4
            $templatePixel = ($y * $Template.Width + $x) * 3
            $rgbDelta = [Math]::Abs([int]$Frame.Bytes[$framePixel + 2] - [int]$Template.Pixels[$templatePixel]) +
                [Math]::Abs([int]$Frame.Bytes[$framePixel + 1] - [int]$Template.Pixels[$templatePixel + 1]) +
                [Math]::Abs([int]$Frame.Bytes[$framePixel] - [int]$Template.Pixels[$templatePixel + 2])
            if ($rgbDelta -le $RgbDeltaMaximum) {
                $matches++
            }
        }
    }
    return $matches
}

function Find-UiSmokeCardTemplate {
    param(
        [hashtable]$Frame,
        [hashtable]$Template,
        [int]$ExpectedCenterX,
        [int]$ExpectedCenterY,
        [int]$SearchRadius = 28,
        [int]$RgbDeltaMaximum = 48
    )

    $expectedLeft = $ExpectedCenterX - [int][Math]::Floor($Template.Width / 2.0)
    $expectedTop = $ExpectedCenterY - [int][Math]::Floor($Template.Height / 2.0)
    $bestMatches = -1
    $bestOffsetX = 0
    $bestOffsetY = 0
    for ($offsetY = -$SearchRadius; $offsetY -le $SearchRadius; $offsetY += 2) {
        for ($offsetX = -$SearchRadius; $offsetX -le $SearchRadius; $offsetX += 2) {
            $left = $expectedLeft + $offsetX
            $top = $expectedTop + $offsetY
            if ($left -lt 0 -or $top -lt 0 -or
                $left + $Template.Width -gt $Frame.Width -or $top + $Template.Height -gt $Frame.Height) {
                continue
            }
            $matches = Get-UiSmokeTemplateMatchCount -Frame $Frame -Template $Template `
                -Left $left -Top $top -RgbDeltaMaximum $RgbDeltaMaximum
            if ($matches -gt $bestMatches) {
                $bestMatches = $matches
                $bestOffsetX = $offsetX
                $bestOffsetY = $offsetY
            }
        }
    }
    if ($bestMatches -lt 0) {
        throw 'card template search region did not overlap the captured frame'
    }

    $coarseOffsetX = $bestOffsetX
    $coarseOffsetY = $bestOffsetY
    for ($offsetY = [Math]::Max(-$SearchRadius, $coarseOffsetY - 1); $offsetY -le [Math]::Min($SearchRadius, $coarseOffsetY + 1); $offsetY++) {
        for ($offsetX = [Math]::Max(-$SearchRadius, $coarseOffsetX - 1); $offsetX -le [Math]::Min($SearchRadius, $coarseOffsetX + 1); $offsetX++) {
            $left = $expectedLeft + $offsetX
            $top = $expectedTop + $offsetY
            $matches = Get-UiSmokeTemplateMatchCount -Frame $Frame -Template $Template `
                -Left $left -Top $top -RgbDeltaMaximum $RgbDeltaMaximum
            if ($matches -gt $bestMatches) {
                $bestMatches = $matches
                $bestOffsetX = $offsetX
                $bestOffsetY = $offsetY
            }
        }
    }
    return @{
        MatchedPixels = $bestMatches
        PixelCount = $Template.PixelCount
        OffsetX = $bestOffsetX
        OffsetY = $bestOffsetY
        Ratio = [double]$bestMatches / $Template.PixelCount
    }
}

function Get-UiSmokeArtRegionEvidence {
    param(
        [hashtable]$Frame,
        [int]$CenterX,
        [int]$CenterY,
        [int]$HalfWidth = 24,
        [int]$HalfHeight = 17,
        [int]$ExpectedRed = 180,
        [int]$ExpectedGreen = 200,
        [int]$ExpectedBlue = 230,
        [int]$RgbDeltaThreshold = 24
    )

    $left = $CenterX - $HalfWidth
    $right = $CenterX + $HalfWidth
    $top = $CenterY - $HalfHeight
    $bottom = $CenterY + $HalfHeight
    if ($left -lt 0 -or $top -lt 0 -or $right -ge $Frame.Width -or $bottom -ge $Frame.Height) {
        throw 'art region exceeds the captured frame'
    }
    $colors = [System.Collections.Generic.HashSet[int]]::new()
    $nonBackgroundPixels = 0
    for ($y = $top; $y -le $bottom; $y++) {
        $frameY = if ($Frame.TopDown) { $y } else { $Frame.Height - 1 - $y }
        $frameRow = $Frame.PixelOffset + $frameY * $Frame.RowStride
        for ($x = $left; $x -le $right; $x++) {
            $framePixel = $frameRow + $x * 4
            $red = [int]$Frame.Bytes[$framePixel + 2]
            $green = [int]$Frame.Bytes[$framePixel + 1]
            $blue = [int]$Frame.Bytes[$framePixel]
            [void]$colors.Add(($red -shl 16) -bor ($green -shl 8) -bor $blue)
            $rgbDelta = [Math]::Abs($red - $ExpectedRed) +
                [Math]::Abs($green - $ExpectedGreen) +
                [Math]::Abs($blue - $ExpectedBlue)
            if ($rgbDelta -ge $RgbDeltaThreshold) {
                $nonBackgroundPixels++
            }
        }
    }
    return @{
        Left = $left
        Top = $top
        Width = $right - $left + 1
        Height = $bottom - $top + 1
        PixelCount = ($right - $left + 1) * ($bottom - $top + 1)
        UniqueColors = $colors.Count
        NonBackgroundPixels = $nonBackgroundPixels
    }
}

try {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class AxiomUiSmokeNative
{
    [StructLayout(LayoutKind.Sequential)]
    public struct Point
    {
        public int X;
        public int Y;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct Rect
    {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
        public int Width { get { return Right - Left; } }
        public int Height { get { return Bottom - Top; } }
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct LastInputInfo
    {
        public uint Size;
        public uint Time;
    }

    private delegate bool EnumWindowsCallback(IntPtr window, IntPtr parameter);

    private const uint WmMouseMove = 0x0200;
    private const uint WmLeftButtonDown = 0x0201;
    private const uint WmLeftButtonUp = 0x0202;
    private const uint WmRightButtonDown = 0x0204;
    private const uint WmRightButtonUp = 0x0205;
    private const uint WmKeyDown = 0x0100;
    private const uint WmKeyUp = 0x0101;
    private const uint MkLeftButton = 0x0001;
    private const uint MkRightButton = 0x0002;

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool GetClientRect(IntPtr window, out Rect rect);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool ClientToScreen(IntPtr window, ref Point point);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool GetCursorPos(out Point point);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool GetLastInputInfo(ref LastInputInfo info);

    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();

    [DllImport("user32.dll", EntryPoint = "PostMessageW", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool PostWindowMessage(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

    public static IntPtr UsePerMonitorDpiContext()
    {
        IntPtr previous = SetThreadDpiAwarenessContext(new IntPtr(-4));
        if (previous == IntPtr.Zero)
            throw new InvalidOperationException("Win32 DPI context change failed: " + Marshal.GetLastWin32Error());
        return previous;
    }

    public static void RestoreDpiContext(IntPtr context)
    {
        if (context != IntPtr.Zero && SetThreadDpiAwarenessContext(context) == IntPtr.Zero)
            throw new InvalidOperationException("Win32 DPI context restore failed: " + Marshal.GetLastWin32Error());
    }

    [DllImport("user32.dll")]
    private static extern bool EnumWindows(EnumWindowsCallback callback, IntPtr parameter);

    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);

    [DllImport("user32.dll")]
    private static extern bool IsWindowVisible(IntPtr window);

    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern int GetWindowText(IntPtr window, StringBuilder text, int count);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool SetWindowPos(IntPtr window, IntPtr insertAfter,
        int x, int y, int width, int height, uint flags);

    public static void PlaceBehindWithoutActivation(IntPtr window)
    {
        const uint SwpNoSize = 0x0001;
        const uint SwpNoMove = 0x0002;
        const uint SwpNoActivate = 0x0010;
        if (!SetWindowPos(window, new IntPtr(1), 0, 0, 0, 0,
            SwpNoSize | SwpNoMove | SwpNoActivate))
            throw new InvalidOperationException("Win32 background Z-order change failed: " + Marshal.GetLastWin32Error());
    }

    public static uint GetWindowProcessId(IntPtr window)
    {
        uint processId;
        GetWindowThreadProcessId(window, out processId);
        return processId;
    }

    public static IntPtr FindGameWindow(int processId)
    {
        IntPtr found = IntPtr.Zero;
        EnumWindows(delegate(IntPtr window, IntPtr parameter) {
            uint ownerProcessId;
            GetWindowThreadProcessId(window, out ownerProcessId);
            if (ownerProcessId != (uint)processId || !IsWindowVisible(window))
                return true;
            StringBuilder title = new StringBuilder(256);
            GetWindowText(window, title, title.Capacity);
            if (!title.ToString().StartsWith("Card Game", StringComparison.Ordinal))
                return true;
            Rect client;
            if (GetClientRect(window, out client) && client.Right > 0 && client.Bottom > 0)
            {
                found = window;
                return false;
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }

    public static Rect GetClientScreenRect(IntPtr window)
    {
        Rect rect;
        Point origin = new Point();
        if (!GetClientRect(window, out rect) || !ClientToScreen(window, ref origin))
            throw new InvalidOperationException("Win32 client bounds failed: " + Marshal.GetLastWin32Error());
        return new Rect {
            Left = origin.X,
            Top = origin.Y,
            Right = origin.X + rect.Right,
            Bottom = origin.Y + rect.Bottom
        };
    }

    public static string GetWindowTitle(IntPtr window)
    {
        StringBuilder title = new StringBuilder(256);
        GetWindowText(window, title, title.Capacity);
        return title.ToString();
    }

    public static Point GetCursorPosition()
    {
        Point point;
        if (!GetCursorPos(out point))
            throw new InvalidOperationException("Win32 cursor query failed: " + Marshal.GetLastWin32Error());
        return point;
    }

    public static uint GetLastInputTick()
    {
        LastInputInfo info = new LastInputInfo();
        info.Size = (uint)Marshal.SizeOf(typeof(LastInputInfo));
        if (!GetLastInputInfo(ref info))
            throw new InvalidOperationException("Win32 last-input query failed: " + Marshal.GetLastWin32Error());
        return info.Time;
    }

    public static void PostMouseMove(IntPtr window, int clientX, int clientY, bool leftButtonDown)
    {
        PostMouseMessage(window, WmMouseMove,
            leftButtonDown ? new UIntPtr(MkLeftButton) : UIntPtr.Zero, clientX, clientY);
    }

    public static void PostLeftButtonDown(IntPtr window, int clientX, int clientY)
    {
        PostMouseMessage(window, WmLeftButtonDown, new UIntPtr(MkLeftButton), clientX, clientY);
    }

    public static void PostLeftButtonUp(IntPtr window, int clientX, int clientY)
    {
        PostMouseMessage(window, WmLeftButtonUp, UIntPtr.Zero, clientX, clientY);
    }

    public static void PostRightButtonDown(IntPtr window, int clientX, int clientY)
    {
        PostMouseMessage(window, WmRightButtonDown, new UIntPtr(MkRightButton), clientX, clientY);
    }

    public static void PostRightButtonUp(IntPtr window, int clientX, int clientY)
    {
        PostMouseMessage(window, WmRightButtonUp, UIntPtr.Zero, clientX, clientY);
    }

    public static void PostVirtualKey(IntPtr window, uint virtualKey, bool pressed)
    {
        uint message = pressed ? WmKeyDown : WmKeyUp;
        int lParam = pressed ? 1 : unchecked((int)0xC0000001);
        if (!PostWindowMessage(window, message, new UIntPtr(virtualKey), new IntPtr(lParam)))
            throw new InvalidOperationException("Win32 key message post failed: " + Marshal.GetLastWin32Error());
    }

    private static void PostMouseMessage(IntPtr window, uint message, UIntPtr buttonState,
        int clientX, int clientY)
    {
        if (clientX < short.MinValue || clientX > short.MaxValue ||
            clientY < short.MinValue || clientY > short.MaxValue)
            throw new ArgumentOutOfRangeException("client position exceeds WM_MOUSE coordinate range");

        uint packedPosition = unchecked((ushort)clientX) | ((uint)unchecked((ushort)clientY) << 16);
        IntPtr position = new IntPtr(unchecked((int)packedPosition));
        if (!PostWindowMessage(window, message, buttonState, position))
            throw new InvalidOperationException("Win32 mouse message post failed: " + Marshal.GetLastWin32Error());
    }

}
'@

    $stage = 'build'
    Push-Location $repoRoot
    try {
        & cargo build -p card_game_bin --features ui-test --locked
        if ($LASTEXITCODE -ne 0) {
            throw "cargo build failed with exit code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }

    $appPath = Join-Path $repoRoot 'target\debug\card_game_bin.exe'
    if (-not (Test-Path -LiteralPath $appPath)) {
        throw "Built application not found at $appPath"
    }

    $stage = 'launch'
    $previousDpiContext = [AxiomUiSmokeNative]::UsePerMonitorDpiContext()
    $foregroundBeforeLaunch = Get-UiSmokeForegroundSnapshot
    $previousCursor = [AxiomUiSmokeNative]::GetCursorPosition()
    $previousStateFile = $env:AXIOM_UI_TEST_STATE_FILE
    $previousScenario = $env:AXIOM_UI_TEST_SCENARIO
    $previousFrameCaptureRequestFile = $env:AXIOM_UI_TEST_FRAME_CAPTURE_REQUEST_FILE
    $previousBackend = $env:WGPU_BACKEND
    try {
        $env:AXIOM_UI_TEST_STATE_FILE = $stateFile
        $env:AXIOM_UI_TEST_SCENARIO = $scenarioName
        $env:AXIOM_UI_TEST_FRAME_CAPTURE_REQUEST_FILE = $frameCaptureRequestFile
        $env:WGPU_BACKEND = 'dx12'
        $process = Start-Process -FilePath $appPath `
            -WorkingDirectory $repoRoot `
            -NoNewWindow `
            -PassThru `
            -RedirectStandardOutput $stdoutPath `
            -RedirectStandardError $stderrPath
    }
    finally {
        $env:AXIOM_UI_TEST_STATE_FILE = $previousStateFile
        $env:AXIOM_UI_TEST_SCENARIO = $previousScenario
        $env:AXIOM_UI_TEST_FRAME_CAPTURE_REQUEST_FILE = $previousFrameCaptureRequestFile
        $env:WGPU_BACKEND = $previousBackend
    }

    $startupWatch = [System.Diagnostics.Stopwatch]::StartNew()
    while ($startupWatch.Elapsed.TotalSeconds -lt 30) {
        $process.Refresh()
        if ($process.HasExited) {
            throw "game process exited before creating its window (exit $($process.ExitCode))"
        }
        $windowHandle = [AxiomUiSmokeNative]::FindGameWindow($process.Id)
        if ($windowHandle -ne [IntPtr]::Zero) {
            break
        }
        Start-Sleep -Milliseconds 250
    }
    if ($windowHandle -eq [IntPtr]::Zero) {
        throw 'game window did not appear within 30 seconds'
    }
    $stage = 'place-window-behind'
    [AxiomUiSmokeNative]::PlaceBehindWithoutActivation($windowHandle)

    $stage = 'seeded-state'
    $cardWorldX = if ($IdentitySignature -or $ArtFace) { -80.0 } else { -160.0 }
    $cardWorldY = 130.0
    $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
        -Stage $stage -TimeoutSeconds 30 -ExpectedState "dragging=false, zone=Table, rendered=($cardWorldX,$cardWorldY) tolerance=1" -Predicate {
        param($state)
        $state['scenario'] -eq $scenarioName -and
        $state['dragging'] -eq 'false' -and
        $state['zone'] -eq 'Table' -and
        (Test-Position -State $state -ExpectedX $cardWorldX -ExpectedY $cardWorldY -Tolerance 1)
    }
    $foregroundAtPreparation = Get-UiSmokeForegroundSnapshot
    $cursorAtPreparation = [AxiomUiSmokeNative]::GetCursorPosition()

    $client = [AxiomUiSmokeNative]::GetClientScreenRect($windowHandle)
    if ($client.Width -lt 640 -or $client.Height -lt 480) {
        throw "game client area is too small for the smoke scenario ($($client.Width)x$($client.Height))"
    }
    $stage = 'capture-baseline-frame'
    Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
        -CapturePath $baselineFramePath -Stage $stage -TimeoutSeconds 10

    $startScreenX = $client.Left + [int][Math]::Round($client.Width / 2.0 + $cardWorldX)
    $startScreenY = $client.Top + [int][Math]::Round($client.Height / 2.0 + $cardWorldY)
    $targetScreenX = $client.Left + [int][Math]::Round($client.Width / 2.0 - 300)
    $targetScreenY = $client.Top + [int][Math]::Round($client.Height / 2.0 - 150)
    $startClientX = [int][Math]::Round($client.Width / 2.0 + $cardWorldX)
    $startClientY = [int][Math]::Round($client.Height / 2.0 + $cardWorldY)
    $targetClientX = [int][Math]::Round($client.Width / 2.0 - 300)
    $targetClientY = [int][Math]::Round($client.Height / 2.0 - 150)
    $handClientX = [int][Math]::Round($client.Width / 2.0)
    $handClientY = [int][Math]::Round($client.Height - 80.0)
    $handWorldX = 0.0
    $handWorldY = $handClientY - $client.Height / 2.0
    $stashClientX = 45
    $stashClientY = 58
    $stashWorldX = $stashClientX - $client.Width / 2.0
    $stashWorldY = $stashClientY - $client.Height / 2.0
    $stashTabTopY = 20 + 10 * 79 - 4
    $stashTabCenterY = [int][Math]::Round($stashTabTopY + 8)
    $stashTabStartX = 20 + (10 * 54 - 4) / 2.0 - (5 * 34 - 4) / 2.0
    $stashPageTwoTabX = [int][Math]::Round($stashTabStartX + 2 * 34 + 15)
    $boosterClientX = [int][Math]::Round($client.Width / 2.0 - 300)
$boosterClientY = [int][Math]::Round($client.Height / 2.0 - 150)
$boosterWorldX = -300.0
$boosterWorldY = -150.0
$readerClientX = [int][Math]::Round($client.Width / 2.0 + 300)
$readerClientY = [int][Math]::Round($client.Height / 2.0)
$readerWorldX = 300.0
$readerWorldY = 0.0
$secondCardWorldX = -80.0
$secondCardWorldY = 130.0
$secondCardClientX = [int][Math]::Round($client.Width / 2.0 + $secondCardWorldX)
$secondCardClientY = [int][Math]::Round($client.Height / 2.0 + $secondCardWorldY)
$secondReaderClientX = [int][Math]::Round($client.Width / 2.0 + 100.0)
$secondReaderClientY = [int][Math]::Round($client.Height / 2.0 - 150.0)
$secondReaderWorldX = 100.0
$secondReaderWorldY = -150.0
$readerJackClientX = [int][Math]::Round($client.Width / 2.0 + 352.0)
$readerJackClientY = [int][Math]::Round($client.Height / 2.0)
$screenJackClientX = [int][Math]::Round($client.Width / 2.0 + 173.0)
$screenJackClientY = [int][Math]::Round($client.Height / 2.0 + 150.0)
$secondReaderJackClientX = [int][Math]::Round($client.Width / 2.0 + 152.0)
$secondReaderJackClientY = [int][Math]::Round($client.Height / 2.0 - 150.0)
$combinerInputAClientX = [int][Math]::Round($client.Width / 2.0 + 248.0)
$combinerInputAClientY = [int][Math]::Round($client.Height / 2.0 - 140.0)
$combinerInputBClientX = [int][Math]::Round($client.Width / 2.0 + 248.0)
$combinerInputBClientY = [int][Math]::Round($client.Height / 2.0 - 160.0)
    $stage = 'prepare-background-input'
    $foregroundBeforeInput = Get-UiSmokeForegroundSnapshot
    $cursorBeforeInputSetup = [AxiomUiSmokeNative]::GetCursorPosition()
    $cursorSetupDrift = $cursorAtPreparation.X -ne $previousCursor.X -or
        $cursorAtPreparation.Y -ne $previousCursor.Y -or
        $cursorBeforeInputSetup.X -ne $previousCursor.X -or
        $cursorBeforeInputSetup.Y -ne $previousCursor.Y
    $mouseClientX = $startClientX
    $mouseClientY = $startClientY
    @(
        "process_id=$($process.Id)"
        "window_handle=$windowHandle"
        "window_z_order=HWND_BOTTOM;SWP_NOACTIVATE|SWP_NOMOVE|SWP_NOSIZE"
        "initial_foreground_handle=$($foregroundBeforeLaunch.Handle)"
        "initial_foreground_title=$($foregroundBeforeLaunch.Title)"
        "initial_foreground_process_id=$($foregroundBeforeLaunch.ProcessId)"
        "foreground_at_preparation=$($foregroundAtPreparation.Handle)"
        "foreground_title_at_preparation=$($foregroundAtPreparation.Title)"
        "foreground_process_id_at_preparation=$($foregroundAtPreparation.ProcessId)"
        "cursor_at_preparation=($($cursorAtPreparation.X),$($cursorAtPreparation.Y))"
        "foreground_before_input=$($foregroundBeforeInput.Handle)"
        "foreground_title_before_input=$($foregroundBeforeInput.Title)"
        "foreground_process_id_before_input=$($foregroundBeforeInput.ProcessId)"
        "cursor_before_launch=($($previousCursor.X),$($previousCursor.Y))"
        "cursor_before_input_setup=($($cursorBeforeInputSetup.X),$($cursorBeforeInputSetup.Y))"
        "cursor_setup_drift_from_launch=$cursorSetupDrift"
        "client_screen=($($client.Left),$($client.Top),$($client.Width),$($client.Height))"
        "start_screen=($startScreenX,$startScreenY)"
        "target_screen=($targetScreenX,$targetScreenY)"
        "start_client=($startClientX,$startClientY)"
        "target_client=($targetClientX,$targetClientY)"
        "hand_client=($handClientX,$handClientY)"
        "hand_world=($handWorldX,$handWorldY)"
        "stash_client=($stashClientX,$stashClientY)"
        "stash_world=($stashWorldX,$stashWorldY)"
        "stash_page_two_tab=($stashPageTwoTabX,$stashTabCenterY)"
        "booster_client=($boosterClientX,$boosterClientY)"
        "booster_world=($boosterWorldX,$boosterWorldY)"
        "reader_client=($readerClientX,$readerClientY)"
        "reader_world=($readerWorldX,$readerWorldY)"
        "second_card_client=($secondCardClientX,$secondCardClientY)"
        "second_card_world=($secondCardWorldX,$secondCardWorldY)"
        "second_reader_client=($secondReaderClientX,$secondReaderClientY)"
        "second_reader_world=($secondReaderWorldX,$secondReaderWorldY)"
        "reader_jack_client=($readerJackClientX,$readerJackClientY)"
        "screen_jack_client=($screenJackClientX,$screenJackClientY)"
        "second_reader_jack_client=($secondReaderJackClientX,$secondReaderJackClientY)"
        "combiner_input_a_client=($combinerInputAClientX,$combinerInputAClientY)"
        "combiner_input_b_client=($combinerInputBClientX,$combinerInputBClientY)"
    ) | Set-Content -LiteralPath $inputFile
    if ($ArtFace) {
        $expectedArtSignature = '0.330000,-0.310000,-0.350000,0.630000,-0.950000,0.650000,-0.290000,0.740000'
        $expectedArtElement = 'Solidum'
        $expectedArtAspect = 'Solid'
        $stage = 'verify-art-face-state'
        $artFaceState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'art_signature=<expected>, art_shape_count>0, face_up=true, zone=Table' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'false' -and
            $state['zone'] -eq 'Table' -and
            $state['face_up'] -eq 'true' -and
            $state['art_signature'] -eq $expectedArtSignature -and
            $state['art_element'] -eq $expectedArtElement -and
            $state['art_aspect'] -eq $expectedArtAspect -and
            [int]::Parse($state['art_shape_count']) -gt 0 -and
            (Test-Position -State $state -ExpectedX $cardWorldX -ExpectedY $cardWorldY -Tolerance 1)
        }
        $artFaceState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'art-face-state.txt')

        $stage = 'capture-art-face-frame'
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $artFaceFramePath -Stage $stage -TimeoutSeconds 10
        $artFaceFrame = Read-UiSmokeBitmap -Path $artFaceFramePath
        $artRegionCenterY = $startClientY - 7
        $artEvidence = Get-UiSmokeArtRegionEvidence -Frame $artFaceFrame `
            -CenterX $startClientX -CenterY $artRegionCenterY
        @(
            "frame_size=$($artFaceFrame.Width)x$($artFaceFrame.Height)"
            "art_region=($($artEvidence.Left),$($artEvidence.Top),$($artEvidence.Width),$($artEvidence.Height))"
            'expected_art_region_background_rgb=180,200,230'
            'art_region_rgb_delta_threshold=24'
            "art_region_unique_rgb_colors=$($artEvidence.UniqueColors)"
            "art_region_non_background_pixels=$($artEvidence.NonBackgroundPixels)/$($artEvidence.PixelCount)"
            'art_region_minimum_unique_rgb_colors=8'
            'art_region_minimum_non_background_pixels=10'
        ) | Set-Content -LiteralPath (Join-Path $artifactDir 'art-face-visual.txt')
        if ($artEvidence.UniqueColors -lt 8 -or $artEvidence.NonBackgroundPixels -lt 10) {
            throw "rendered art region verification failed: unique_colors=$($artEvidence.UniqueColors) required>=8, non_background=$($artEvidence.NonBackgroundPixels)/$($artEvidence.PixelCount) required>=10"
        }
        $foregroundAfterInput = Get-UiSmokeForegroundSnapshot
        $cursorAfterInput = [AxiomUiSmokeNative]::GetCursorPosition()
        $lastInputTickAfterReleaseAck = [AxiomUiSmokeNative]::GetLastInputTick()
        $cursorStability = 'not_applicable_no_input'
        $succeeded = $true
    }
    elseif ($IdentitySignature) {
        @(
            'input_mode=identity_no_postmessagew'
            "foreground_before_postmessagew=$identityNoInput"
            "foreground_process_id_before_postmessagew=$identityNoInput"
            "cursor_before_postmessagew=$identityNoInput"
            "last_input_tick_before_postmessagew=$identityNoInput"
        ) | Add-Content -LiteralPath $inputFile
    }
    if ($foregroundAtPreparation.Handle -eq $windowHandle) {
        throw "game window became foreground during preparation (hwnd=$windowHandle pid=$($process.Id))"
    }
    if ($foregroundBeforeInput.Handle -eq $windowHandle) {
        throw "game window was foreground before input (hwnd=$windowHandle pid=$($process.Id))"
    }

    if ($IdentitySignature) {
        $expectedIdentitySignature = '-0.609306,0.758221,0.160116,0.099062,-0.170454,0.657969,0.647077,0.870853'
        $expectedIdentitySeed = '13799725383080882384'
        $expectedIdentityRarity = 'Uncommon'
        $expectedIdentityTier = 'Dormant'
        $expectedIdentityName = 'Cresting Fadevanish'
        $stage = 'verify-identity-state'
        $identityState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState "identity_signature=$expectedIdentitySignature, identity_seed=$expectedIdentitySeed, identity_rarity=$expectedIdentityRarity, identity_tier=$expectedIdentityTier, identity_name=$expectedIdentityName, face_up=true, zone=Table, rendered=($cardWorldX,$cardWorldY)" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'false' -and
            $state['zone'] -eq 'Table' -and
            $state['face_up'] -eq 'true' -and
            $state['identity_signature'] -eq $expectedIdentitySignature -and
            $state['identity_seed'] -eq $expectedIdentitySeed -and
            $state['identity_rarity'] -eq $expectedIdentityRarity -and
            $state['identity_tier'] -eq $expectedIdentityTier -and
            $state['identity_name'] -eq $expectedIdentityName -and
            (Test-Position -State $state -ExpectedX $cardWorldX -ExpectedY $cardWorldY -Tolerance 1)
        }
        $identityState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'identity-state.txt')

        $stage = 'capture-identity-frame'
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $identityFramePath -Stage $stage -TimeoutSeconds 10
        $identityFrame = Read-UiSmokeBitmap -Path $identityFramePath
        $identityBaselineFrame = Read-UiSmokeBitmap -Path $baselineFramePath
        if ($identityBaselineFrame.Width -ne $identityFrame.Width -or
            $identityBaselineFrame.Height -ne $identityFrame.Height) {
            throw "identity baseline frame size $($identityBaselineFrame.Width)x$($identityBaselineFrame.Height) does not match identity frame $($identityFrame.Width)x$($identityFrame.Height)"
        }
        $identityExpectedTemplate = New-UiSmokeCardTemplate -Frame $identityBaselineFrame `
            -CenterX $startClientX -CenterY $startClientY
        $identityTemplateMatch = Find-UiSmokeCardTemplate -Frame $identityFrame -Template $identityExpectedTemplate `
            -ExpectedCenterX $startClientX -ExpectedCenterY $startClientY `
            -SearchRadius 8 -RgbDeltaMaximum 32
        $minimumIdentityMatchPixels = [int][Math]::Ceiling($identityExpectedTemplate.PixelCount * 90 / 100.0)
        @(
            "frame_size=$($identityFrame.Width)x$($identityFrame.Height)"
            'identity_expected_template_source=baseline.bmp'
            "identity_template_size=$($identityExpectedTemplate.Width)x$($identityExpectedTemplate.Height)"
            "identity_template_unique_rgb_colors=$($identityExpectedTemplate.UniqueColors)"
            "identity_template_match=$($identityTemplateMatch.MatchedPixels)/$($identityTemplateMatch.PixelCount)"
            "identity_template_minimum_match_percent=90"
            "identity_template_offset=$($identityTemplateMatch.OffsetX),$($identityTemplateMatch.OffsetY)"
        ) | Set-Content -LiteralPath (Join-Path $artifactDir 'identity-visual.txt')
        if ($identityTemplateMatch.MatchedPixels -lt $minimumIdentityMatchPixels) {
            throw "identity frame did not retain the visible card: match=$($identityTemplateMatch.MatchedPixels)/$($identityTemplateMatch.PixelCount), required>=$minimumIdentityMatchPixels"
        }

        $foregroundAfterInput = Get-UiSmokeForegroundSnapshot
        $cursorAfterInput = [AxiomUiSmokeNative]::GetCursorPosition()
        $lastInputTickAfterReleaseAck = [AxiomUiSmokeNative]::GetLastInputTick()
        $cursorStability = 'not_applicable_no_input'
        $succeeded = $true
    }
    elseif ($StashRoundTrip) {
        $stage = 'open-stash'
        $foregroundBeforePostMessage = Get-UiSmokeForegroundSnapshot
        $cursorBeforePostMessage = [AxiomUiSmokeNative]::GetCursorPosition()
        $lastInputTickBeforePostMessage = [AxiomUiSmokeNative]::GetLastInputTick()
        if ($foregroundBeforePostMessage.Handle -eq $windowHandle) {
            throw "game window was foreground before stash toggle (hwnd=$windowHandle pid=$($process.Id))"
        }
        [AxiomUiSmokeNative]::PostVirtualKey($windowHandle, 0x09, $true)
        [AxiomUiSmokeNative]::PostVirtualKey($windowHandle, 0x09, $false)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'stash_visible=true, stash_page=1, zone=Table' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['stash_visible'] -eq 'true' -and
            $state['stash_page'] -eq '1' -and
            $state['zone'] -eq 'Table'
        }
        $stage = 'capture-stash-open-frame'
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $stashOpenFramePath -Stage $stage -TimeoutSeconds 10
    }

    if (-not $IdentitySignature -and $BoosterOpening) {
        $stage = 'hover-booster-pack'
        $foregroundBeforePostMessage = Get-UiSmokeForegroundSnapshot
        $cursorBeforePostMessage = [AxiomUiSmokeNative]::GetCursorPosition()
        $lastInputTickBeforePostMessage = [AxiomUiSmokeNative]::GetLastInputTick()
        if ($foregroundBeforePostMessage.Handle -eq $windowHandle) {
            throw "game window was foreground before booster input (hwnd=$windowHandle pid=$($process.Id))"
        }
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $boosterClientX, $boosterClientY, $false)
        $sealedState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState 'booster_phase=sealed, booster_pack_present=true, booster_card_count=1' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['booster_phase'] -eq 'sealed' -and
            $state['booster_pack_present'] -eq 'true' -and
            $state['booster_card_count'] -eq '1' -and
            (Test-Position -State $state -XKey 'booster_rendered_x' -YKey 'booster_rendered_y' `
                -ExpectedX $boosterWorldX -ExpectedY $boosterWorldY -Tolerance 2) -and
            (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
                -ExpectedX $boosterClientX -ExpectedY $boosterClientY -Tolerance 3)
        }
        $sealedState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'booster-sealed-state.txt')

        $stage = 'first-booster-click'
        [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $boosterClientX, $boosterClientY)
        $mouseClientX = $boosterClientX
        $mouseClientY = $boosterClientY
        $mouseDown = $true
        $firstClickState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState 'booster_dragging=true, booster_phase=sealed, booster_pack_present=true' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['booster_dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['booster_phase'] -eq 'sealed' -and
            $state['booster_pack_present'] -eq 'true'
        }
        $firstClickState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'booster-first-click-state.txt')
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $boosterClientX, $boosterClientY)
        $mouseDown = $false
        Start-Sleep -Milliseconds 50
        $stage = 'open-booster-pack'
        [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $boosterClientX, $boosterClientY)
        $mouseDown = $true
        $openingState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'booster opening phase, booster_pack_present=true, booster_card_count=1' -Predicate {
            param($state)
            $openingPhases = @('moving_to_center', 'ripping', 'lowering_pack', 'revealing_cards', 'completing')
            $state['scenario'] -eq $scenarioName -and
            $openingPhases -contains $state['booster_phase'] -and
            $state['booster_pack_present'] -eq 'true' -and
            $state['booster_card_count'] -eq '1' -and
            $state['booster_dragging'] -eq 'false'
        }
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $boosterClientX, $boosterClientY)
        $mouseDown = $false
        $openingState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'booster-opening-state.txt')
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $boosterOpeningFramePath -Stage 'capture-booster-opening-frame' -TimeoutSeconds 10

        $stage = 'complete-booster-opening'
        $openedState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 15 -ExpectedState 'booster_phase=done, booster_pack_present=false, opened_card_present=true, opened_card_zone=Table, matching identity seed' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['booster_phase'] -eq 'done' -and
            $state['booster_pack_present'] -eq 'false' -and
            $state['booster_card_count'] -eq '1' -and
            $state['opened_card_present'] -eq 'true' -and
            $state['opened_card_zone'] -eq 'Table' -and
            $state['opened_card_face_up'] -eq 'true' -and
            $state['opened_card_seed'] -eq $state['expected_card_seed'] -and
            (Test-Position -State $state -XKey 'opened_card_x' -YKey 'opened_card_y' `
                -ExpectedX -300 -ExpectedY -230 -Tolerance 35)
        }
        $openedState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'booster-opened-state.txt')
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $boosterOpenedFramePath -Stage 'capture-booster-opened-frame' -TimeoutSeconds 10

        $baselineFrame = Read-UiSmokeBitmap -Path $baselineFramePath
        $openingFrame = Read-UiSmokeBitmap -Path $boosterOpeningFramePath
        $openedFrame = Read-UiSmokeBitmap -Path $boosterOpenedFramePath
        $rgbDeltaThreshold = 24
        $minimumBoosterChangedPixels = 250
        $sealedOriginChanges = Get-UiSmokeChangedPixels -Before $baselineFrame -After $openingFrame `
            -CenterX $boosterClientX -CenterY $boosterClientY -HalfWidth 90 -HalfHeight 125 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        $openingCenterChanges = Get-UiSmokeChangedPixels -Before $baselineFrame -After $openingFrame `
            -CenterX ([int][Math]::Round($client.Width / 2.0)) -CenterY ([int][Math]::Round($client.Height / 2.0)) `
            -HalfWidth 150 -HalfHeight 160 -RgbDeltaThreshold $rgbDeltaThreshold
        $openedCenterChanges = Get-UiSmokeChangedPixels -Before $openingFrame -After $openedFrame `
            -CenterX ([int][Math]::Round($client.Width / 2.0)) -CenterY ([int][Math]::Round($client.Height / 2.0)) `
            -HalfWidth 150 -HalfHeight 160 -RgbDeltaThreshold $rgbDeltaThreshold
        @(
            "frame_size=$($openedFrame.Width)x$($openedFrame.Height)"
            "rgb_delta_threshold=$rgbDeltaThreshold"
            "minimum_changed_pixels=$minimumBoosterChangedPixels"
            "sealed_origin_changed_pixels=$($sealedOriginChanges.ChangedPixels)"
            "opening_center_changed_pixels=$($openingCenterChanges.ChangedPixels)"
            "opened_center_changed_pixels=$($openedCenterChanges.ChangedPixels)"
            "expected_card_seed=$($openedState['expected_card_seed'])"
            "opened_card_seed=$($openedState['opened_card_seed'])"
        ) | Set-Content -LiteralPath (Join-Path $artifactDir 'booster-visual-diff.txt')
        if ($sealedOriginChanges.ChangedPixels -lt $minimumBoosterChangedPixels -or
            $openingCenterChanges.ChangedPixels -lt $minimumBoosterChangedPixels -or
            $openedCenterChanges.ChangedPixels -lt $minimumBoosterChangedPixels) {
            throw "booster opening frame changed too few pixels: sealed_origin=$($sealedOriginChanges.ChangedPixels), opening_center=$($openingCenterChanges.ChangedPixels), opened_center=$($openedCenterChanges.ChangedPixels), minimum=$minimumBoosterChangedPixels"
        }

        $stage = 'verify-background-input'
        $lastInputTickAfterReleaseAck = [AxiomUiSmokeNative]::GetLastInputTick()
        $foregroundAfterInput = Get-UiSmokeForegroundSnapshot
        $cursorAfterInput = [AxiomUiSmokeNative]::GetCursorPosition()
        $cursorMovedDuringInput = $cursorAfterInput.X -ne $cursorBeforePostMessage.X -or
            $cursorAfterInput.Y -ne $cursorBeforePostMessage.Y
        $lastInputChangedDuringInput = $lastInputTickAfterReleaseAck -ne $lastInputTickBeforePostMessage
        $cursorStability = if (-not $cursorMovedDuringInput) {
            'unchanged'
        }
        elseif ($lastInputChangedDuringInput) {
            'inconclusive_external_input'
        }
        else {
            'moved_without_external_input'
        }
        @(
            "foreground_after_input=$($foregroundAfterInput.Handle)"
            "foreground_title_after_input=$($foregroundAfterInput.Title)"
            "foreground_process_id_after_input=$($foregroundAfterInput.ProcessId)"
            "cursor_after_input=($($cursorAfterInput.X),$($cursorAfterInput.Y))"
            "last_input_tick_after_release_ack=$lastInputTickAfterReleaseAck"
            "cursor_stability=$cursorStability"
        ) | Add-Content -LiteralPath $inputFile
        if ($foregroundAfterInput.Handle -eq $windowHandle) {
            throw "game window became foreground during booster input (hwnd=$windowHandle pid=$($process.Id))"
        }
        if ($cursorMovedDuringInput -and -not $lastInputChangedDuringInput) {
            throw "booster opening PostMessageW input interval cursor drift without external input: before=($($cursorBeforePostMessage.X),$($cursorBeforePostMessage.Y)) after=($($cursorAfterInput.X),$($cursorAfterInput.Y)) last_input_tick_before=$lastInputTickBeforePostMessage last_input_tick_after=$lastInputTickAfterReleaseAck"
        }
        $succeeded = $true
    }
    elseif ($Interaction) {
        $interactionStartClientX = $startClientX + 15
        $interactionStartClientY = $startClientY
        $interactionTargetClientX = $targetClientX
        $interactionTargetClientY = $targetClientY
        $spinClientX = $interactionStartClientX + 15
        $spinClientY = $interactionStartClientY - 60
        # Repeated runs observed spin -1.0366..-0.9728 and final -2.2265..-2.2258 radians.
        $interactionRotationTolerance = 0.1
        # The repeated target-position error stayed below 23.2 units; share the existing 35-unit margin.
        $interactionPositionTolerance = 35
        @(
            "interaction_start_client=($interactionStartClientX,$interactionStartClientY)"
            "interaction_target_client=($interactionTargetClientX,$interactionTargetClientY)"
            "spin_client=($spinClientX,$spinClientY)"
            "expected_spin_rotation=-1.0000"
            "expected_final_rotation=-2.2250"
            "rotation_tolerance=$interactionRotationTolerance"
            "position_tolerance=$interactionPositionTolerance"
        ) | Add-Content -LiteralPath $inputFile

        $stage = 'hover-interaction-card'
        $foregroundBeforePostMessage = Get-UiSmokeForegroundSnapshot
        $cursorBeforePostMessage = [AxiomUiSmokeNative]::GetCursorPosition()
        $lastInputTickBeforePostMessage = [AxiomUiSmokeNative]::GetLastInputTick()
        if ($foregroundBeforePostMessage.Handle -eq $windowHandle) {
            throw "game window was foreground before interaction input (hwnd=$windowHandle pid=$($process.Id))"
        }
        [AxiomUiSmokeNative]::PostMouseMove(
            $windowHandle, $interactionStartClientX, $interactionStartClientY, $false)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState "left_pressed=false, mouse=($interactionStartClientX,$interactionStartClientY) tolerance=3" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['left_pressed'] -eq 'false' -and
            $state['right_pressed'] -eq 'false' -and
            (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
                -ExpectedX $interactionStartClientX -ExpectedY $interactionStartClientY -Tolerance 3)
        }

        $stage = 'pick-card'
        [AxiomUiSmokeNative]::PostLeftButtonDown(
            $windowHandle, $interactionStartClientX, $interactionStartClientY)
        $mouseClientX = $interactionStartClientX
        $mouseClientY = $interactionStartClientY
        $mouseDown = $true
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState 'dragging=true, left_pressed=true, face_up=false' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['face_up'] -eq 'false' -and
            $state['zone'] -eq 'Table'
        }

        $stage = 'spin-card'
        [AxiomUiSmokeNative]::PostMouseMove(
            $windowHandle, $spinClientX, $spinClientY, $true)
        $spinState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState 'dragging=true, rotation=-1.0000 +/- 0.1000 radians, negative' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            (Test-ExpectedRotation -State $state -Key 'rotation' -Expected -1.0 -Tolerance $interactionRotationTolerance)
        }
        $spinState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'spin-state.txt')

        $stage = 'glide-card'
        for ($step = 1; $step -le 10; $step++) {
            $mouseClientX = [int][Math]::Round(
                $spinClientX + ($interactionTargetClientX - $spinClientX) * $step / 10.0)
            $mouseClientY = [int][Math]::Round(
                $spinClientY + ($interactionTargetClientY - $spinClientY) * $step / 10.0)
            [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $mouseClientX, $mouseClientY, $true)
            Start-Sleep -Milliseconds 35
        }
        $interactionState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'dragging=true, target position tolerance=35, rotation=-2.2250 +/- 0.1000 radians, negative' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['zone'] -eq 'Table' -and
            (Test-Position -State $state -ExpectedX -300 -ExpectedY -150 -Tolerance $interactionPositionTolerance) -and
            (Test-ExpectedRotation -State $state -Key 'rotation' -Expected -2.225 -Tolerance $interactionRotationTolerance)
        }
        $interactionState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'interaction-state.txt')

        $stage = 'capture-interaction-frame'
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $framePath -Stage $stage -TimeoutSeconds 10
        $stage = 'verify-interaction-frame'
        $baselineFrame = Read-UiSmokeBitmap -Path $baselineFramePath
        $interactionFrame = Read-UiSmokeBitmap -Path $framePath
        $rgbDeltaThreshold = 24
        $minimumChangedPixels = 500
        $sourceChanges = Get-UiSmokeChangedPixels -Before $baselineFrame -After $interactionFrame `
            -CenterX $startClientX -CenterY $startClientY -HalfWidth 80 -HalfHeight 85 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        $targetChanges = Get-UiSmokeChangedPixels -Before $baselineFrame -After $interactionFrame `
            -CenterX $interactionTargetClientX -CenterY $interactionTargetClientY -HalfWidth 80 -HalfHeight 85 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        @(
            "frame_size=$($interactionFrame.Width)x$($interactionFrame.Height)"
            "rgb_delta_threshold=$rgbDeltaThreshold"
            "minimum_changed_pixels_per_roi=$minimumChangedPixels"
            "source_changed_pixels=$($sourceChanges.ChangedPixels)"
            "target_changed_pixels=$($targetChanges.ChangedPixels)"
            "rotation=$($interactionState['rotation'])"
        ) | Set-Content -LiteralPath (Join-Path $artifactDir 'interaction-visual-diff.txt')
        if ($sourceChanges.ChangedPixels -lt $minimumChangedPixels -or
            $targetChanges.ChangedPixels -lt $minimumChangedPixels) {
            throw "interaction frame changed too few pixels: source=$($sourceChanges.ChangedPixels), target=$($targetChanges.ChangedPixels), minimum=$minimumChangedPixels"
        }

        $stage = 'release-interaction-card'
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $mouseClientX, $mouseClientY)
        $releasedState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'dragging=false, left_pressed=false, zone=Table' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'false' -and
            $state['left_pressed'] -eq 'false' -and
            $state['zone'] -eq 'Table' -and
            (Test-Position -State $state -ExpectedX -300 -ExpectedY -150 -Tolerance $interactionPositionTolerance)
        }
        $mouseDown = $false
        Start-Sleep -Milliseconds 250
        $releasedState = Read-UiState -Path $stateFile
        if ($null -eq $releasedState -or
            -not (Test-Position -State $releasedState -ExpectedX -300 -ExpectedY -150 -Tolerance $interactionPositionTolerance)) {
            throw "released card left the expected target: observed=$(Format-UiStateDiagnostic -State $releasedState)"
        }
        $releasedState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'released-state.txt')
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $releasedFramePath -Stage 'capture-released-frame' -TimeoutSeconds 10

        $stage = 'flip-card'
        [AxiomUiSmokeNative]::PostMouseMove(
            $windowHandle, $interactionTargetClientX, $interactionTargetClientY, $false)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage 'hover-flip-card' -TimeoutSeconds 5 -ExpectedState 'right_pressed=false, target mouse position' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['right_pressed'] -eq 'false' -and
            (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
                -ExpectedX $interactionTargetClientX -ExpectedY $interactionTargetClientY -Tolerance 3)
        }
        [AxiomUiSmokeNative]::PostRightButtonDown(
            $windowHandle, $interactionTargetClientX, $interactionTargetClientY)
        $rightMouseDown = $true
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage 'press-flip-button' -TimeoutSeconds 5 -ExpectedState 'right_pressed=true' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['right_pressed'] -eq 'true'
        }
        [AxiomUiSmokeNative]::PostRightButtonUp(
            $windowHandle, $interactionTargetClientX, $interactionTargetClientY)
        $rightMouseDown = $false
        $flippedState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage 'complete-flip' -TimeoutSeconds 10 -ExpectedState 'face_up=true, right_pressed=false, target position tolerance=35, rotation=-2.2250 +/- 0.1000 radians, negative' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['right_pressed'] -eq 'false' -and
            $state['face_up'] -eq 'true' -and
            $state['dragging'] -eq 'false' -and
            (Test-Position -State $state -ExpectedX -300 -ExpectedY -150 -Tolerance $interactionPositionTolerance) -and
            (Test-ExpectedRotation -State $state -Key 'rotation' -Expected -2.225 -Tolerance $interactionRotationTolerance)
        }
        $flippedState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'flipped-state.txt')
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $flippedFramePath -Stage 'capture-flipped-frame' -TimeoutSeconds 10

        $releasedFrame = Read-UiSmokeBitmap -Path $releasedFramePath
        $flippedFrame = Read-UiSmokeBitmap -Path $flippedFramePath
        $flipChanges = Get-UiSmokeChangedPixels -Before $releasedFrame -After $flippedFrame `
            -CenterX $interactionTargetClientX -CenterY $interactionTargetClientY -HalfWidth 80 -HalfHeight 85 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        "flip_changed_pixels=$($flipChanges.ChangedPixels)" | Set-Content -LiteralPath (Join-Path $artifactDir 'flip-visual-diff.txt')
        if ($flipChanges.ChangedPixels -lt $minimumChangedPixels) {
            throw "flip frame changed too few pixels: changed=$($flipChanges.ChangedPixels), minimum=$minimumChangedPixels"
        }
        $stage = 'verify-background-input'
        $lastInputTickAfterReleaseAck = [AxiomUiSmokeNative]::GetLastInputTick()
        $foregroundAfterInput = Get-UiSmokeForegroundSnapshot
        $cursorAfterInput = [AxiomUiSmokeNative]::GetCursorPosition()
        $cursorMovedDuringInput = $cursorAfterInput.X -ne $cursorBeforePostMessage.X -or
            $cursorAfterInput.Y -ne $cursorBeforePostMessage.Y
        $lastInputChangedDuringInput = $lastInputTickAfterReleaseAck -ne $lastInputTickBeforePostMessage
        $cursorStability = if (-not $cursorMovedDuringInput) {
            'unchanged'
        }
        elseif ($lastInputChangedDuringInput) {
            'inconclusive_external_input'
        }
        else {
            'moved_without_external_input'
        }
        @(
            "foreground_after_input=$($foregroundAfterInput.Handle)"
            "foreground_title_after_input=$($foregroundAfterInput.Title)"
            "foreground_process_id_after_input=$($foregroundAfterInput.ProcessId)"
            "cursor_after_input=($($cursorAfterInput.X),$($cursorAfterInput.Y))"
            "last_input_tick_after_release_ack=$lastInputTickAfterReleaseAck"
            "cursor_stability=$cursorStability"
        ) | Add-Content -LiteralPath $inputFile
        if ($foregroundAfterInput.Handle -eq $windowHandle) {
            throw "game window became foreground during interaction input (hwnd=$windowHandle pid=$($process.Id))"
        }
        if ($cursorMovedDuringInput -and -not $lastInputChangedDuringInput) {
            throw "interaction PostMessageW input interval cursor drift without external input: before=($($cursorBeforePostMessage.X),$($cursorBeforePostMessage.Y)) after=($($cursorAfterInput.X),$($cursorAfterInput.Y)) last_input_tick_before=$lastInputTickBeforePostMessage last_input_tick_after=$lastInputTickAfterReleaseAck"
        }
        $succeeded = $true
    }
    elseif ($ReaderRoundTrip) {
        $seededState = Read-UiState -Path $stateFile
        $expectedReaderSignature = $seededState['identity_signature']
        if ($null -eq $seededState -or [string]::IsNullOrWhiteSpace($expectedReaderSignature)) {
            throw "reader scenario could not establish the seeded card signature: observed=$(Format-UiStateDiagnostic -State $seededState)"
        }
        $readerPositionTolerance = 25
        $rgbDeltaThreshold = 24
        $minimumReaderChangedPixels = 250
        @(
            "reader_expected_signature=$expectedReaderSignature"
            "reader_position_tolerance=$readerPositionTolerance"
            "rgb_delta_threshold=$rgbDeltaThreshold"
            "minimum_reader_changed_pixels=$minimumReaderChangedPixels"
        ) | Add-Content -LiteralPath $inputFile

        $stage = 'hover-reader-insert'
        $foregroundBeforePostMessage = Get-UiSmokeForegroundSnapshot
        $cursorBeforePostMessage = [AxiomUiSmokeNative]::GetCursorPosition()
        $lastInputTickBeforePostMessage = [AxiomUiSmokeNative]::GetLastInputTick()
        if ($foregroundBeforePostMessage.Handle -eq $windowHandle) {
            throw "game window was foreground before reader input (hwnd=$windowHandle pid=$($process.Id))"
        }
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $startClientX, $startClientY, $false)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState "left_pressed=false, mouse=($startClientX,$startClientY) tolerance=3" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['left_pressed'] -eq 'false' -and
            (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
                -ExpectedX $startClientX -ExpectedY $startClientY -Tolerance 3)
        }

        $stage = 'press-reader-card'
        [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $startClientX, $startClientY)
        $mouseClientX = $startClientX
        $mouseClientY = $startClientY
        $mouseDown = $true
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState 'dragging=true, left_pressed=true, zone=Table' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['zone'] -eq 'Table' -and
            $state['reader_loaded'] -eq 'false'
        }

        $stage = 'drag-card-to-reader'
        for ($step = 1; $step -le 10; $step++) {
            $mouseClientX = [int][Math]::Round($startClientX + ($readerClientX - $startClientX) * $step / 10.0)
            $mouseClientY = [int][Math]::Round($startClientY + ($readerClientY - $startClientY) * $step / 10.0)
            [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $mouseClientX, $mouseClientY, $true)
            Start-Sleep -Milliseconds 35
        }
        $dragToReaderState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState "dragging=true, zone=Table, rendered=($readerWorldX,$readerWorldY) tolerance=$readerPositionTolerance" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['zone'] -eq 'Table' -and
            $state['reader_loaded'] -eq 'false' -and
            (Test-Position -State $state -ExpectedX $readerWorldX -ExpectedY $readerWorldY -Tolerance $readerPositionTolerance)
        }
        $dragToReaderState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'reader-drag-state.txt')

        $stage = 'insert-card-into-reader'
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $readerClientX, $readerClientY)
        $insertedState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'dragging=false, zone=Reader, reader_loaded=true, matching signature, signature space populated, feedback=lit' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'false' -and
            $state['left_pressed'] -eq 'false' -and
            $state['zone'] -like 'Reader(*)' -and
            $state['reader_loaded'] -eq 'true' -and
            $state['reader_signature'] -eq $expectedReaderSignature -and
            $state['reader_space_contains'] -eq 'true' -and
            $state['reader_space_source_count'] -eq '1' -and
            [double]::Parse($state['reader_space_radius'], [Globalization.CultureInfo]::InvariantCulture) -gt 0 -and
            $state['reader_feedback'] -eq 'lit' -and
            (Test-Position -State $state -ExpectedX $readerWorldX -ExpectedY $readerWorldY -Tolerance 1)
        }
        $mouseDown = $false
        $insertedState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'reader-inserted-state.txt')
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $readerInsertedFramePath -Stage 'capture-reader-inserted-frame' -TimeoutSeconds 10

        $baselineFrame = Read-UiSmokeBitmap -Path $baselineFramePath
        $readerInsertedFrame = Read-UiSmokeBitmap -Path $readerInsertedFramePath
        $readerInsertedChanges = Get-UiSmokeChangedPixels -Before $baselineFrame -After $readerInsertedFrame `
            -CenterX $readerClientX -CenterY $readerClientY -HalfWidth 65 -HalfHeight 85 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        @(
            "frame_size=$($readerInsertedFrame.Width)x$($readerInsertedFrame.Height)"
            "rgb_delta_threshold=$rgbDeltaThreshold"
            "minimum_changed_pixels=$minimumReaderChangedPixels"
            "reader_changed_pixels=$($readerInsertedChanges.ChangedPixels)"
        ) | Set-Content -LiteralPath (Join-Path $artifactDir 'reader-inserted-visual-diff.txt')
        if ($readerInsertedChanges.ChangedPixels -lt $minimumReaderChangedPixels) {
            throw "reader insertion frame changed too few pixels: reader=$($readerInsertedChanges.ChangedPixels), minimum=$minimumReaderChangedPixels"
        }

        $stage = 'hover-inserted-reader-card'
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $readerClientX, $readerClientY, $false)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState "reader card mouse=($readerClientX,$readerClientY), reader_loaded=true" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['reader_loaded'] -eq 'true' -and
            (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
                -ExpectedX $readerClientX -ExpectedY $readerClientY -Tolerance 3)
        }

        $stage = 'eject-card-from-reader'
        [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $readerClientX, $readerClientY)
        $mouseClientX = $readerClientX
        $mouseClientY = $readerClientY
        $mouseDown = $true
        $ejectedState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'dragging=true, zone=Table, reader_loaded=false, signature space cleared, feedback=dim' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['zone'] -eq 'Table' -and
            $state['reader_loaded'] -eq 'false' -and
            [string]::IsNullOrEmpty($state['reader_signature']) -and
            $state['reader_space_contains'] -eq 'false' -and
            $state['reader_space_source_count'] -eq '0' -and
            $state['reader_feedback'] -eq 'dim'
        }
        $ejectedState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'reader-ejected-state.txt')
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $readerEjectedFramePath -Stage 'capture-reader-ejected-frame' -TimeoutSeconds 10

        $stage = 'drag-ejected-card-to-table'
        for ($step = 1; $step -le 10; $step++) {
            $mouseClientX = [int][Math]::Round($readerClientX + ($startClientX - $readerClientX) * $step / 10.0)
            $mouseClientY = [int][Math]::Round($readerClientY + ($startClientY - $readerClientY) * $step / 10.0)
            [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $mouseClientX, $mouseClientY, $true)
            Start-Sleep -Milliseconds 35
        }
        $dragFromReaderState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState "dragging=true, zone=Table, reader_loaded=false, rendered=($cardWorldX,$cardWorldY) tolerance=$readerPositionTolerance" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['zone'] -eq 'Table' -and
            $state['reader_loaded'] -eq 'false' -and
            (Test-Position -State $state -ExpectedX $cardWorldX -ExpectedY $cardWorldY -Tolerance $readerPositionTolerance)
        }
        $dragFromReaderState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'reader-return-drag-state.txt')

        $stage = 'release-ejected-card-to-table'
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $mouseClientX, $mouseClientY)
        $returnedState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState "dragging=false, zone=Table, reader_loaded=false, rendered=($cardWorldX,$cardWorldY) tolerance=35" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'false' -and
            $state['left_pressed'] -eq 'false' -and
            $state['zone'] -eq 'Table' -and
            $state['reader_loaded'] -eq 'false' -and
            [string]::IsNullOrEmpty($state['reader_signature']) -and
            $state['reader_space_source_count'] -eq '0' -and
            $state['reader_feedback'] -eq 'dim' -and
            (Test-Position -State $state -ExpectedX $cardWorldX -ExpectedY $cardWorldY -Tolerance 35)
        }
        $mouseDown = $false
        Start-Sleep -Milliseconds 250
        $returnedState = Read-UiState -Path $stateFile
        if ($null -eq $returnedState -or
            $returnedState['zone'] -ne 'Table' -or
            $returnedState['reader_loaded'] -ne 'false' -or
            -not [string]::IsNullOrEmpty($returnedState['reader_signature']) -or
            $returnedState['reader_space_source_count'] -ne '0' -or
            -not (Test-Position -State $returnedState -ExpectedX $cardWorldX -ExpectedY $cardWorldY -Tolerance 35)) {
            throw "ejected card left the expected table state: observed=$(Format-UiStateDiagnostic -State $returnedState)"
        }
        $returnedState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath $returnedStatePath
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $readerReturnedFramePath -Stage 'capture-reader-returned-frame' -TimeoutSeconds 10

        $readerEjectedFrame = Read-UiSmokeBitmap -Path $readerEjectedFramePath
        $readerReturnedFrame = Read-UiSmokeBitmap -Path $readerReturnedFramePath
        $ejectedReaderChanges = Get-UiSmokeChangedPixels -Before $readerInsertedFrame -After $readerEjectedFrame `
            -CenterX $readerClientX -CenterY $readerClientY -HalfWidth 65 -HalfHeight 85 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        $returnedReaderChanges = Get-UiSmokeChangedPixels -Before $readerInsertedFrame -After $readerReturnedFrame `
            -CenterX $readerClientX -CenterY $readerClientY -HalfWidth 65 -HalfHeight 85 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        $returnedTableChanges = Get-UiSmokeChangedPixels -Before $readerEjectedFrame -After $readerReturnedFrame `
            -CenterX $startClientX -CenterY $startClientY -HalfWidth 80 -HalfHeight 85 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        @(
            "frame_size=$($readerReturnedFrame.Width)x$($readerReturnedFrame.Height)"
            "rgb_delta_threshold=$rgbDeltaThreshold"
            "minimum_changed_pixels=$minimumReaderChangedPixels"
            "inserted_to_ejected_reader_changed_pixels=$($ejectedReaderChanges.ChangedPixels)"
            "inserted_to_returned_reader_changed_pixels=$($returnedReaderChanges.ChangedPixels)"
            "ejected_to_returned_table_changed_pixels=$($returnedTableChanges.ChangedPixels)"
        ) | Set-Content -LiteralPath (Join-Path $artifactDir 'reader-visual-diff.txt')
        if ($ejectedReaderChanges.ChangedPixels -lt $minimumReaderChangedPixels -or
            $returnedReaderChanges.ChangedPixels -lt $minimumReaderChangedPixels -or
            $returnedTableChanges.ChangedPixels -lt $minimumReaderChangedPixels) {
            throw "reader ejection frame changed too few pixels: inserted_to_ejected_reader=$($ejectedReaderChanges.ChangedPixels), inserted_to_returned_reader=$($returnedReaderChanges.ChangedPixels), ejected_to_returned_table=$($returnedTableChanges.ChangedPixels), minimum=$minimumReaderChangedPixels"
        }

        $stage = 'verify-background-input'
        $lastInputTickAfterReleaseAck = [AxiomUiSmokeNative]::GetLastInputTick()
        $foregroundAfterInput = Get-UiSmokeForegroundSnapshot
        $cursorAfterInput = [AxiomUiSmokeNative]::GetCursorPosition()
        $cursorMovedDuringInput = $cursorAfterInput.X -ne $cursorBeforePostMessage.X -or
            $cursorAfterInput.Y -ne $cursorBeforePostMessage.Y
        $lastInputChangedDuringInput = $lastInputTickAfterReleaseAck -ne $lastInputTickBeforePostMessage
        $cursorStability = if (-not $cursorMovedDuringInput) {
            'unchanged'
        }
        elseif ($lastInputChangedDuringInput) {
            'inconclusive_external_input'
        }
        else {
            'moved_without_external_input'
        }
        @(
            "foreground_after_input=$($foregroundAfterInput.Handle)"
            "foreground_title_after_input=$($foregroundAfterInput.Title)"
            "foreground_process_id_after_input=$($foregroundAfterInput.ProcessId)"
            "cursor_after_input=($($cursorAfterInput.X),$($cursorAfterInput.Y))"
            "last_input_tick_after_release_ack=$lastInputTickAfterReleaseAck"
            "cursor_stability=$cursorStability"
        ) | Add-Content -LiteralPath $inputFile
        if ($foregroundAfterInput.Handle -eq $windowHandle) {
            throw "game window became foreground during reader round-trip (hwnd=$windowHandle pid=$($process.Id))"
        }
        if ($cursorMovedDuringInput -and -not $lastInputChangedDuringInput) {
            throw "reader round-trip PostMessageW input interval cursor drift without external input: before=($($cursorBeforePostMessage.X),$($cursorBeforePostMessage.Y)) after=($($cursorAfterInput.X),$($cursorAfterInput.Y)) last_input_tick_before=$lastInputTickBeforePostMessage last_input_tick_after=$lastInputTickAfterReleaseAck"
        }
        $succeeded = $true
    }
    elseif ($CableWrapping) {
        $cablePositionTolerance = 2
        $expectedCableSourceX = 352.0
        $expectedCableSourceY = 0.0
        $expectedCableAnchorX = 340.0
        $expectedCableAnchorY = 55.0
        $expectedCableDestX = 173.0
        $expectedCableDestY = 150.0
        $rgbDeltaThreshold = 24
        $minimumCableChangedPixels = 100
        @(
            "cable_expected_source=($expectedCableSourceX,$expectedCableSourceY)"
            "cable_expected_anchor=($expectedCableAnchorX,$expectedCableAnchorY)"
            "cable_expected_dest=($expectedCableDestX,$expectedCableDestY)"
            "cable_position_tolerance=$cablePositionTolerance"
            "rgb_delta_threshold=$rgbDeltaThreshold"
            "minimum_cable_changed_pixels=$minimumCableChangedPixels"
        ) | Add-Content -LiteralPath $inputFile

        $stage = 'hover-cable-source'
        $foregroundBeforePostMessage = Get-UiSmokeForegroundSnapshot
        $cursorBeforePostMessage = [AxiomUiSmokeNative]::GetCursorPosition()
        $lastInputTickBeforePostMessage = [AxiomUiSmokeNative]::GetLastInputTick()
        if ($foregroundBeforePostMessage.Handle -eq $windowHandle) {
            throw "game window was foreground before cable input (hwnd=$windowHandle pid=$($process.Id))"
        }
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $readerJackClientX, $readerJackClientY, $false)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState "left_pressed=false, mouse=($readerJackClientX,$readerJackClientY) tolerance=3" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['left_pressed'] -eq 'false' -and
            (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
                -ExpectedX $readerJackClientX -ExpectedY $readerJackClientY -Tolerance 3)
        }

        $stage = 'start-cable-drag'
        [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $readerJackClientX, $readerJackClientY)
        $mouseClientX = $readerJackClientX
        $mouseClientY = $readerJackClientY
        $mouseDown = $true
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState 'cable_dragging=true, cable_present=true' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['cable_present'] -eq 'true' -and
            $state['pending_cable_dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true'
        }

        $stage = 'drag-cable-around-reader'
        for ($step = 1; $step -le 10; $step++) {
            $mouseClientX = [int][Math]::Round($readerJackClientX + ($screenJackClientX - $readerJackClientX) * $step / 10.0)
            $mouseClientY = [int][Math]::Round($readerJackClientY + ($screenJackClientY - $readerJackClientY) * $step / 10.0)
            [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $mouseClientX, $mouseClientY, $true)
            Start-Sleep -Milliseconds 35
        }
        $dragCableState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'cable_dragging=true, one wrap anchor, rendered geometry retained' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['pending_cable_dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            [int]::Parse($state['cable_anchor_count']) -eq 1 -and
            (Test-Position -State $state -XKey 'cable_anchor_0_x' -YKey 'cable_anchor_0_y' `
                -ExpectedX $expectedCableAnchorX -ExpectedY $expectedCableAnchorY -Tolerance $cablePositionTolerance)
        }
        $dragCableState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'cable-wrapping-drag-state.txt')

        $stage = 'connect-cable-to-screen-jack'
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $screenJackClientX, $screenJackClientY)
        $cableState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'cable_connected=true, one wrap anchor, rendered geometry matches expected path' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['pending_cable_dragging'] -eq 'false' -and
            $state['left_pressed'] -eq 'false' -and
            $state['cable_connected'] -eq 'true' -and
            [int]::Parse($state['cable_anchor_count']) -eq 1 -and
            (Test-Position -State $state -XKey 'cable_source_x' -YKey 'cable_source_y' `
                -ExpectedX $expectedCableSourceX -ExpectedY $expectedCableSourceY -Tolerance $cablePositionTolerance) -and
            (Test-Position -State $state -XKey 'cable_anchor_0_x' -YKey 'cable_anchor_0_y' `
                -ExpectedX $expectedCableAnchorX -ExpectedY $expectedCableAnchorY -Tolerance $cablePositionTolerance) -and
            (Test-Position -State $state -XKey 'cable_dest_x' -YKey 'cable_dest_y' `
                -ExpectedX $expectedCableDestX -ExpectedY $expectedCableDestY -Tolerance $cablePositionTolerance)
        }
        $mouseDown = $false
        $cableState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'cable-wrapping-state.txt')
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $cableWrappingFramePath -Stage 'capture-cable-wrapping-frame' -TimeoutSeconds 10

        $baselineFrame = Read-UiSmokeBitmap -Path $baselineFramePath
        $cableFrame = Read-UiSmokeBitmap -Path $cableWrappingFramePath
        if ($baselineFrame.Width -ne $cableFrame.Width -or $baselineFrame.Height -ne $cableFrame.Height) {
            throw "cable frame size $($cableFrame.Width)x$($cableFrame.Height) does not match baseline $($baselineFrame.Width)x$($baselineFrame.Height)"
        }
        $cableChanges = Get-UiSmokeChangedPixels -Before $baselineFrame -After $cableFrame `
            -CenterX ([int][Math]::Round($client.Width / 2.0 + 295.0)) `
            -CenterY ([int][Math]::Round($client.Height / 2.0 + 75.0)) `
            -HalfWidth 140 -HalfHeight 90 -RgbDeltaThreshold $rgbDeltaThreshold
        $cableRenderedVertexCount = [int]::Parse($cableState['cable_rendered_vertex_count'])
        $cableRenderedSourceX = [double]::Parse($cableState['cable_rendered_source_x'], [Globalization.CultureInfo]::InvariantCulture)
        $cableRenderedSourceY = [double]::Parse($cableState['cable_rendered_source_y'], [Globalization.CultureInfo]::InvariantCulture)
        $cableRenderedAnchorX = [double]::Parse($cableState['cable_rendered_anchor_x'], [Globalization.CultureInfo]::InvariantCulture)
        $cableRenderedAnchorY = [double]::Parse($cableState['cable_rendered_anchor_y'], [Globalization.CultureInfo]::InvariantCulture)
        $cableRenderedDestX = [double]::Parse($cableState['cable_rendered_dest_x'], [Globalization.CultureInfo]::InvariantCulture)
        $cableRenderedDestY = [double]::Parse($cableState['cable_rendered_dest_y'], [Globalization.CultureInfo]::InvariantCulture)
        $cableRenderedMaxDeviation = [double]::Parse($cableState['cable_rendered_max_deviation'], [Globalization.CultureInfo]::InvariantCulture)
        $cableRenderedMinX = [double]::Parse($cableState['cable_rendered_min_x'], [Globalization.CultureInfo]::InvariantCulture)
        $cableRenderedMaxX = [double]::Parse($cableState['cable_rendered_max_x'], [Globalization.CultureInfo]::InvariantCulture)
        $cableRenderedMinY = [double]::Parse($cableState['cable_rendered_min_y'], [Globalization.CultureInfo]::InvariantCulture)
        $cableRenderedMaxY = [double]::Parse($cableState['cable_rendered_max_y'], [Globalization.CultureInfo]::InvariantCulture)
        @(
            "frame_size=$($cableFrame.Width)x$($cableFrame.Height)"
            "rgb_delta_threshold=$rgbDeltaThreshold"
            "minimum_cable_changed_pixels=$minimumCableChangedPixels"
            "cable_roi=($($cableChanges.Left),$($cableChanges.Top),$($cableChanges.Width),$($cableChanges.Height))"
            "cable_changed_pixels=$($cableChanges.ChangedPixels)"
            "expected_waypoints=($expectedCableSourceX,$expectedCableSourceY)->($expectedCableAnchorX,$expectedCableAnchorY)->($expectedCableDestX,$expectedCableDestY)"
            "observed_waypoints=($($cableState['cable_source_x']),$($cableState['cable_source_y']))->($($cableState['cable_anchor_0_x']),$($cableState['cable_anchor_0_y']))->($($cableState['cable_dest_x']),$($cableState['cable_dest_y']))"
            "rendered_waypoints=($cableRenderedSourceX,$cableRenderedSourceY)->($cableRenderedAnchorX,$cableRenderedAnchorY)->($cableRenderedDestX,$cableRenderedDestY)"
            "rendered_max_deviation=$cableRenderedMaxDeviation"
            "rendered_bounds=($cableRenderedMinX,$cableRenderedMinY)-($cableRenderedMaxX,$cableRenderedMaxY)"
            "rendered_vertex_count=$cableRenderedVertexCount"
        ) | Set-Content -LiteralPath (Join-Path $artifactDir 'cable-wrapping-visual-diff.txt')
        if ($cableChanges.ChangedPixels -lt $minimumCableChangedPixels) {
            throw "cable frame changed too few pixels: changed=$($cableChanges.ChangedPixels), minimum=$minimumCableChangedPixels"
        }
        if ($cableRenderedVertexCount -lt 10 -or
            -not (Test-Position -State $cableState -XKey 'cable_rendered_source_x' -YKey 'cable_rendered_source_y' `
                -ExpectedX $expectedCableSourceX -ExpectedY $expectedCableSourceY -Tolerance $cablePositionTolerance) -or
            -not (Test-Position -State $cableState -XKey 'cable_rendered_anchor_x' -YKey 'cable_rendered_anchor_y' `
                -ExpectedX $expectedCableAnchorX -ExpectedY $expectedCableAnchorY -Tolerance $cablePositionTolerance) -or
            -not (Test-Position -State $cableState -XKey 'cable_rendered_dest_x' -YKey 'cable_rendered_dest_y' `
                -ExpectedX $expectedCableDestX -ExpectedY $expectedCableDestY -Tolerance $cablePositionTolerance) -or
            $cableRenderedMaxDeviation -lt 20 -or
            $cableRenderedMinX -gt $expectedCableAnchorX - $cablePositionTolerance -or
            $cableRenderedMaxX -lt $expectedCableAnchorX + $cablePositionTolerance -or
            $cableRenderedMinY -gt $expectedCableAnchorY - $cablePositionTolerance -or
            $cableRenderedMaxY -lt $expectedCableAnchorY + $cablePositionTolerance) {
            throw "cable rendered geometry did not retain the expected wrapped path: vertices=$cableRenderedVertexCount rendered_waypoints=($cableRenderedSourceX,$cableRenderedSourceY)->($cableRenderedAnchorX,$cableRenderedAnchorY)->($cableRenderedDestX,$cableRenderedDestY) max_deviation=$cableRenderedMaxDeviation bounds=($cableRenderedMinX,$cableRenderedMinY)-($cableRenderedMaxX,$cableRenderedMaxY)"
        }

        $stage = 'verify-background-input'
        $lastInputTickAfterReleaseAck = [AxiomUiSmokeNative]::GetLastInputTick()
        $foregroundAfterInput = Get-UiSmokeForegroundSnapshot
        $cursorAfterInput = [AxiomUiSmokeNative]::GetCursorPosition()
        $cursorMovedDuringInput = $cursorAfterInput.X -ne $cursorBeforePostMessage.X -or
            $cursorAfterInput.Y -ne $cursorBeforePostMessage.Y
        $lastInputChangedDuringInput = $lastInputTickAfterReleaseAck -ne $lastInputTickBeforePostMessage
        $cursorStability = if (-not $cursorMovedDuringInput) {
            'unchanged'
        }
        elseif ($lastInputChangedDuringInput) {
            'inconclusive_external_input'
        }
        else {
            'moved_without_external_input'
        }
        @(
            "foreground_after_input=$($foregroundAfterInput.Handle)"
            "foreground_title_after_input=$($foregroundAfterInput.Title)"
            "foreground_process_id_after_input=$($foregroundAfterInput.ProcessId)"
            "cursor_after_input=($($cursorAfterInput.X),$($cursorAfterInput.Y))"
            "last_input_tick_after_release_ack=$lastInputTickAfterReleaseAck"
            "cursor_stability=$cursorStability"
        ) | Add-Content -LiteralPath $inputFile
        if ($foregroundAfterInput.Handle -eq $windowHandle) {
            throw "game window became foreground during cable wrapping (hwnd=$windowHandle pid=$($process.Id))"
        }
        if ($cursorMovedDuringInput -and -not $lastInputChangedDuringInput) {
            throw "cable wrapping PostMessageW input interval cursor drift without external input: before=($($cursorBeforePostMessage.X),$($cursorBeforePostMessage.Y)) after=($($cursorAfterInput.X),$($cursorAfterInput.Y)) last_input_tick_before=$lastInputTickBeforePostMessage last_input_tick_after=$lastInputTickAfterReleaseAck"
        }
        $succeeded = $true
    }
    elseif ($CombinerProcessing) {
        $seededState = Read-UiState -Path $stateFile
        $expectedPrimarySignature = $seededState['identity_signature']
        $expectedSecondarySignature = $seededState['second_identity_signature']
        if ($null -eq $seededState -or
            [string]::IsNullOrWhiteSpace($expectedPrimarySignature) -or
            [string]::IsNullOrWhiteSpace($expectedSecondarySignature)) {
            throw "combiner scenario could not establish both seeded card signatures: observed=$(Format-UiStateDiagnostic -State $seededState)"
        }
        $combinerPositionTolerance = 25
        $rgbDeltaThreshold = 24
        $minimumCombinerChangedPixels = 500
        @(
            "combiner_primary_signature=$expectedPrimarySignature"
            "combiner_secondary_signature=$expectedSecondarySignature"
            "combiner_position_tolerance=$combinerPositionTolerance"
            "rgb_delta_threshold=$rgbDeltaThreshold"
            "minimum_combiner_changed_pixels=$minimumCombinerChangedPixels"
        ) | Add-Content -LiteralPath $inputFile

        $stage = 'hover-first-combiner-card'
        $foregroundBeforePostMessage = Get-UiSmokeForegroundSnapshot
        $cursorBeforePostMessage = [AxiomUiSmokeNative]::GetCursorPosition()
        $lastInputTickBeforePostMessage = [AxiomUiSmokeNative]::GetLastInputTick()
        if ($foregroundBeforePostMessage.Handle -eq $windowHandle) {
            throw "game window was foreground before combiner input (hwnd=$windowHandle pid=$($process.Id))"
        }
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $startClientX, $startClientY, $false)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState "left_pressed=false, mouse=($startClientX,$startClientY) tolerance=3" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['left_pressed'] -eq 'false' -and
            (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
                -ExpectedX $startClientX -ExpectedY $startClientY -Tolerance 3)
        }

        $stage = 'press-first-combiner-card'
        [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $startClientX, $startClientY)
        $mouseClientX = $startClientX
        $mouseClientY = $startClientY
        $mouseDown = $true
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState 'dragging=true, left_pressed=true, zone=Table' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['zone'] -eq 'Table' -and
            $state['reader_loaded'] -eq 'false'
        }

        $stage = 'drag-first-card-to-reader'
        for ($step = 1; $step -le 10; $step++) {
            $mouseClientX = [int][Math]::Round($startClientX + ($readerClientX - $startClientX) * $step / 10.0)
            $mouseClientY = [int][Math]::Round($startClientY + ($readerClientY - $startClientY) * $step / 10.0)
            [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $mouseClientX, $mouseClientY, $true)
            Start-Sleep -Milliseconds 35
        }
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState "dragging=true, zone=Table, rendered=($readerWorldX,$readerWorldY) tolerance=$combinerPositionTolerance" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['zone'] -eq 'Table' -and
            (Test-Position -State $state -ExpectedX $readerWorldX -ExpectedY $readerWorldY -Tolerance $combinerPositionTolerance)
        }
        $stage = 'insert-first-card-into-reader'
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $readerClientX, $readerClientY)
        $insertedPrimaryState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'dragging=false, zone=Reader, reader_loaded=true, signature space populated' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'false' -and
            $state['left_pressed'] -eq 'false' -and
            $state['zone'] -like 'Reader(*)' -and
            $state['reader_loaded'] -eq 'true' -and
            $state['reader_signature'] -eq $expectedPrimarySignature -and
            $state['reader_space_source_count'] -eq '1'
        }
        $mouseDown = $false

        $stage = 'hover-second-combiner-card'
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $secondCardClientX, $secondCardClientY, $false)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState "left_pressed=false, mouse=($secondCardClientX,$secondCardClientY) tolerance=3" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['left_pressed'] -eq 'false' -and
            (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
                -ExpectedX $secondCardClientX -ExpectedY $secondCardClientY -Tolerance 3)
        }
        $stage = 'press-second-combiner-card'
        [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $secondCardClientX, $secondCardClientY)
        $mouseClientX = $secondCardClientX
        $mouseClientY = $secondCardClientY
        $mouseDown = $true
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState 'second_dragging=true, second_zone=Table' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['second_dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['second_zone'] -eq 'Table'
        }
        $stage = 'drag-second-card-to-reader'
        for ($step = 1; $step -le 10; $step++) {
            $mouseClientX = [int][Math]::Round($secondCardClientX + ($secondReaderClientX - $secondCardClientX) * $step / 10.0)
            $mouseClientY = [int][Math]::Round($secondCardClientY + ($secondReaderClientY - $secondCardClientY) * $step / 10.0)
            [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $mouseClientX, $mouseClientY, $true)
            Start-Sleep -Milliseconds 35
        }
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState "second_dragging=true, second_zone=Table, second_rendered=($secondReaderWorldX,$secondReaderWorldY) tolerance=$combinerPositionTolerance" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['second_dragging'] -eq 'true' -and
            $state['second_zone'] -eq 'Table' -and
            (Test-Position -State $state -XKey 'second_rendered_x' -YKey 'second_rendered_y' `
                -ExpectedX $secondReaderWorldX -ExpectedY $secondReaderWorldY -Tolerance $combinerPositionTolerance)
        }
        $stage = 'insert-second-card-into-reader'
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $secondReaderClientX, $secondReaderClientY)
        $insertedSecondaryState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'second_dragging=false, second_zone=Reader, second_reader_loaded=true, second signature retained' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['second_dragging'] -eq 'false' -and
            $state['left_pressed'] -eq 'false' -and
            $state['second_zone'] -like 'Reader(*)' -and
            $state['second_reader_loaded'] -eq 'true' -and
            $state['second_identity_signature'] -eq $expectedSecondarySignature
        }
        $mouseDown = $false

        $stage = 'connect-first-reader-to-combiner'
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $readerJackClientX, $readerJackClientY, $false)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState "mouse=($readerJackClientX,$readerJackClientY) tolerance=3" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
                -ExpectedX $readerJackClientX -ExpectedY $readerJackClientY -Tolerance 3)
        }
        [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $readerJackClientX, $readerJackClientY)
        $mouseClientX = $readerJackClientX
        $mouseClientY = $readerJackClientY
        $mouseDown = $true
        $stage = 'drag-first-reader-cable'
        for ($step = 1; $step -le 10; $step++) {
            $mouseClientX = [int][Math]::Round($readerJackClientX + ($combinerInputAClientX - $readerJackClientX) * $step / 10.0)
            $mouseClientY = [int][Math]::Round($readerJackClientY + ($combinerInputAClientY - $readerJackClientY) * $step / 10.0)
            [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $mouseClientX, $mouseClientY, $true)
            Start-Sleep -Milliseconds 35
        }
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $combinerInputAClientX, $combinerInputAClientY)
        $firstCableState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'combiner input A connected, one source card propagated' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['combiner_input_a_connected'] -eq 'true' -and
            $state['combiner_input_a_source_count'] -eq '1'
        }
        $mouseDown = $false

        $stage = 'connect-second-reader-to-combiner'
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $secondReaderJackClientX, $secondReaderJackClientY, $false)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState "mouse=($secondReaderJackClientX,$secondReaderJackClientY) tolerance=3" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
                -ExpectedX $secondReaderJackClientX -ExpectedY $secondReaderJackClientY -Tolerance 3)
        }
        [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $secondReaderJackClientX, $secondReaderJackClientY)
        $mouseClientX = $secondReaderJackClientX
        $mouseClientY = $secondReaderJackClientY
        $mouseDown = $true
        $stage = 'drag-second-reader-cable'
        for ($step = 1; $step -le 10; $step++) {
            $mouseClientX = [int][Math]::Round($secondReaderJackClientX + ($combinerInputBClientX - $secondReaderJackClientX) * $step / 10.0)
            $mouseClientY = [int][Math]::Round($secondReaderJackClientY + ($combinerInputBClientY - $secondReaderJackClientY) * $step / 10.0)
            [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $mouseClientX, $mouseClientY, $true)
            Start-Sleep -Milliseconds 35
        }
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $combinerInputBClientX, $combinerInputBClientY)
        $combinedState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'both combiner inputs linked, two source cards, two output control points' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['combiner_input_a_connected'] -eq 'true' -and
            $state['combiner_input_b_connected'] -eq 'true' -and
            $state['combiner_input_a_source_count'] -eq '1' -and
            $state['combiner_input_b_source_count'] -eq '1' -and
            $state['combiner_output_source_count'] -eq '2' -and
            $state['combiner_output_control_points'] -eq '2' -and
            [double]::Parse($state['combiner_output_radius'], [Globalization.CultureInfo]::InvariantCulture) -gt 0 -and
            $state['combiner_output_contains_primary'] -eq 'true' -and
            $state['combiner_output_contains_secondary'] -eq 'true'
        }
        $mouseDown = $false
        $combinedState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'combiner-state.txt')
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $combinerFramePath -Stage 'capture-combiner-frame' -TimeoutSeconds 10

        $baselineFrame = Read-UiSmokeBitmap -Path $baselineFramePath
        $combinerFrame = Read-UiSmokeBitmap -Path $combinerFramePath
        $combinerChanges = Get-UiSmokeChangedPixels -Before $baselineFrame -After $combinerFrame `
            -CenterX ([int][Math]::Round($client.Width / 2.0 + 225.0)) `
            -CenterY ([int][Math]::Round($client.Height / 2.0 - 75.0)) `
            -HalfWidth 220 -HalfHeight 150 -RgbDeltaThreshold $rgbDeltaThreshold
        @(
            "frame_size=$($combinerFrame.Width)x$($combinerFrame.Height)"
            "rgb_delta_threshold=$rgbDeltaThreshold"
            "minimum_changed_pixels=$minimumCombinerChangedPixels"
            "combiner_roi=($($combinerChanges.Left),$($combinerChanges.Top),$($combinerChanges.Width),$($combinerChanges.Height))"
            "combiner_changed_pixels=$($combinerChanges.ChangedPixels)"
            "output_source_count=$($combinedState['combiner_output_source_count'])"
            "output_control_points=$($combinedState['combiner_output_control_points'])"
        ) | Set-Content -LiteralPath (Join-Path $artifactDir 'combiner-visual-diff.txt')
        if ($combinerChanges.ChangedPixels -lt $minimumCombinerChangedPixels) {
            throw "combiner frame changed too few pixels: changed=$($combinerChanges.ChangedPixels), minimum=$minimumCombinerChangedPixels"
        }

        $stage = 'verify-background-input'
        $lastInputTickAfterReleaseAck = [AxiomUiSmokeNative]::GetLastInputTick()
        $foregroundAfterInput = Get-UiSmokeForegroundSnapshot
        $cursorAfterInput = [AxiomUiSmokeNative]::GetCursorPosition()
        $cursorMovedDuringInput = $cursorAfterInput.X -ne $cursorBeforePostMessage.X -or
            $cursorAfterInput.Y -ne $cursorBeforePostMessage.Y
        $lastInputChangedDuringInput = $lastInputTickAfterReleaseAck -ne $lastInputTickBeforePostMessage
        $cursorStability = if (-not $cursorMovedDuringInput) {
            'unchanged'
        }
        elseif ($lastInputChangedDuringInput) {
            'inconclusive_external_input'
        }
        else {
            'moved_without_external_input'
        }
        @(
            "foreground_after_input=$($foregroundAfterInput.Handle)"
            "foreground_title_after_input=$($foregroundAfterInput.Title)"
            "foreground_process_id_after_input=$($foregroundAfterInput.ProcessId)"
            "cursor_after_input=($($cursorAfterInput.X),$($cursorAfterInput.Y))"
            "last_input_tick_after_release_ack=$lastInputTickAfterReleaseAck"
            "cursor_stability=$cursorStability"
        ) | Add-Content -LiteralPath $inputFile
        if ($foregroundAfterInput.Handle -eq $windowHandle) {
            throw "game window became foreground during combiner processing (hwnd=$windowHandle pid=$($process.Id))"
        }
        if ($cursorMovedDuringInput -and -not $lastInputChangedDuringInput) {
            throw "combiner PostMessageW input interval cursor drift without external input: before=($($cursorBeforePostMessage.X),$($cursorBeforePostMessage.Y)) after=($($cursorAfterInput.X),$($cursorAfterInput.Y)) last_input_tick_before=$lastInputTickBeforePostMessage last_input_tick_after=$lastInputTickAfterReleaseAck"
        }
        $succeeded = $true
    }
    elseif (-not $IdentitySignature -and -not $ArtFace) {
    $stage = 'hover-card'
    $foregroundBeforePostMessage = Get-UiSmokeForegroundSnapshot
    if ($foregroundBeforePostMessage.Handle -eq $windowHandle) {
        throw "game window was foreground before PostMessageW input (hwnd=$windowHandle pid=$($process.Id))"
    }
    $cursorBeforePostMessage = [AxiomUiSmokeNative]::GetCursorPosition()
    $lastInputTickBeforePostMessage = [AxiomUiSmokeNative]::GetLastInputTick()
    try {
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $startClientX, $startClientY, $false)
    }
    finally {
        @(
            "foreground_before_postmessage=$($foregroundBeforePostMessage.Handle)"
            "foreground_title_before_postmessage=$($foregroundBeforePostMessage.Title)"
            "foreground_process_id_before_postmessage=$($foregroundBeforePostMessage.ProcessId)"
            "cursor_before_postmessage=($($cursorBeforePostMessage.X),$($cursorBeforePostMessage.Y))"
            "last_input_tick_before_postmessage=$lastInputTickBeforePostMessage"
        ) | Add-Content -LiteralPath $inputFile
    }
    $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
        -Stage $stage -TimeoutSeconds 5 -ExpectedState "left_pressed=false, mouse=($startClientX,$startClientY) tolerance=3" -Predicate {
        param($state)
        $state['scenario'] -eq $scenarioName -and
        $state['left_pressed'] -eq 'false' -and
        (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
            -ExpectedX $startClientX -ExpectedY $startClientY -Tolerance 3)
    }

    $stage = 'press-card'
    [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $mouseClientX, $mouseClientY)
    $mouseDown = $true
    $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
        -Stage $stage -TimeoutSeconds 5 -ExpectedState 'dragging=true, left_pressed=true, zone=Table, rendered=(-160,130) tolerance=25' -Predicate {
        param($state)
        $state['scenario'] -eq $scenarioName -and
        $state['dragging'] -eq 'true' -and
        $state['left_pressed'] -eq 'true' -and
        $state['zone'] -eq 'Table' -and
        (Test-Position -State $state -ExpectedX -160 -ExpectedY 130 -Tolerance 25)
    }

    $dragTargetClientX = if ($StashRoundTrip) {
        $stashClientX
    }
    elseif ($HandRoundTrip -or $ZoneTransition) {
        $handClientX
    }
    else {
        $targetClientX
    }
    $dragTargetClientY = if ($StashRoundTrip) {
        $stashClientY
    }
    elseif ($HandRoundTrip -or $ZoneTransition) {
        $handClientY
    }
    else {
        $targetClientY
    }
    $dragTargetWorldX = if ($StashRoundTrip) {
        $stashWorldX
    }
    elseif ($HandRoundTrip -or $ZoneTransition) {
        $handWorldX
    }
    else {
        -300.0
    }
    $dragTargetWorldY = if ($StashRoundTrip) {
        $stashWorldY
    }
    elseif ($HandRoundTrip -or $ZoneTransition) {
        $handWorldY
    }
    else {
        -150.0
    }
    $stage = 'drag-card'
    for ($step = 1; $step -le 10; $step++) {
        $mouseClientX = [int][Math]::Round($startClientX + ($dragTargetClientX - $startClientX) * $step / 10.0)
        $mouseClientY = [int][Math]::Round($startClientY + ($dragTargetClientY - $startClientY) * $step / 10.0)
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $mouseClientX, $mouseClientY, $true)
        Start-Sleep -Milliseconds 35
    }
    $dragState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
        -Stage $stage -TimeoutSeconds 10 -ExpectedState "dragging=true, left_pressed=true, zone=Table, rendered=($dragTargetWorldX,$dragTargetWorldY) tolerance=25" -Predicate {
        param($state)
        $state['scenario'] -eq $scenarioName -and
        $state['dragging'] -eq 'true' -and
        $state['left_pressed'] -eq 'true' -and
        $state['zone'] -eq 'Table' -and
        (Test-Position -State $state -ExpectedX $dragTargetWorldX -ExpectedY $dragTargetWorldY -Tolerance 25)
    }

    $stage = 'capture-dragged-frame'
    $dragState.GetEnumerator() | ForEach-Object {
        '{0}={1}' -f $_.Key, $_.Value
    } | Set-Content -LiteralPath (Join-Path $artifactDir 'drag-state.txt')
    Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
        -CapturePath $framePath -Stage $stage -TimeoutSeconds 10

    if ($StashRoundTrip) {
        $stage = 'verify-stash-drag-preview'
        $stashDragState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState "dragging=true, zone=Table, stash_visible=true, stash_page=1, stash_cursor_follow=true, rendered=($stashWorldX,$stashWorldY) tolerance=5" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['zone'] -eq 'Table' -and
            $state['stash_visible'] -eq 'true' -and
            $state['stash_page'] -eq '1' -and
            $state['stash_slot_present'] -eq 'false' -and
            $state['stash_cursor_follow'] -eq 'true' -and
            (Test-Position -State $state -ExpectedX $stashWorldX -ExpectedY $stashWorldY -Tolerance 5)
        }
        $stashDragState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'stash-drag-state.txt')

        $stashOpenFrame = Read-UiSmokeBitmap -Path $stashOpenFramePath
        $stashDragFrame = Read-UiSmokeBitmap -Path $framePath
        $rgbDeltaThreshold = 24
        $minimumStashChangedPixels = 250
        $stashSourceChanges = Get-UiSmokeChangedPixels -Before $stashOpenFrame -After $stashDragFrame `
            -CenterX $startClientX -CenterY $startClientY -HalfWidth 80 -HalfHeight 85 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        $stashPreviewChanges = Get-UiSmokeChangedPixels -Before $stashOpenFrame -After $stashDragFrame `
            -CenterX $stashClientX -CenterY $stashClientY -HalfWidth 40 -HalfHeight 55 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        @(
            "frame_size=$($stashDragFrame.Width)x$($stashDragFrame.Height)"
            "rgb_delta_threshold=$rgbDeltaThreshold"
            "minimum_changed_pixels=$minimumStashChangedPixels"
            "source_changed_pixels=$($stashSourceChanges.ChangedPixels)"
            "stash_preview_changed_pixels=$($stashPreviewChanges.ChangedPixels)"
        ) | Set-Content -LiteralPath (Join-Path $artifactDir 'stash-drag-visual-diff.txt')
        if ($stashSourceChanges.ChangedPixels -lt $minimumStashChangedPixels -or
            $stashPreviewChanges.ChangedPixels -lt $minimumStashChangedPixels) {
            throw "stash drag preview changed too few pixels: source=$($stashSourceChanges.ChangedPixels), preview=$($stashPreviewChanges.ChangedPixels), minimum=$minimumStashChangedPixels"
        }

        $stage = 'release-card-to-stash'
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $mouseClientX, $mouseClientY)
        $stashState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState "dragging=false, zone=Stash { page: 0, col: 0, row: 0 }, stash_slot=0:0:0, stash_slot_present=true, rendered=($stashWorldX,$stashWorldY) tolerance=5" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'false' -and
            $state['left_pressed'] -eq 'false' -and
            $state['zone'] -eq 'Stash { page: 0, col: 0, row: 0 }' -and
            $state['stash_slot'] -eq '0:0:0' -and
            $state['stash_slot_present'] -eq 'true' -and
            $state['stash_cursor_follow'] -eq 'false' -and
            (Test-Position -State $state -ExpectedX $stashWorldX -ExpectedY $stashWorldY -Tolerance 5)
        }
        $mouseDown = $false
        $stashState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'stash-state.txt')
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $stashStoredFramePath -Stage 'capture-stash-stored-frame' -TimeoutSeconds 10

        $stashStoredFrame = Read-UiSmokeBitmap -Path $stashStoredFramePath
        $storedPreviewChanges = Get-UiSmokeChangedPixels -Before $stashOpenFrame -After $stashStoredFrame `
            -CenterX $stashClientX -CenterY $stashClientY -HalfWidth 40 -HalfHeight 55 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        "stash_stored_changed_pixels=$($storedPreviewChanges.ChangedPixels)" | Set-Content -LiteralPath (Join-Path $artifactDir 'stash-stored-visual-diff.txt')
        if ($storedPreviewChanges.ChangedPixels -lt $minimumStashChangedPixels) {
            throw "stored stash frame changed too few pixels: changed=$($storedPreviewChanges.ChangedPixels), minimum=$minimumStashChangedPixels"
        }

        $stage = 'switch-to-stash-page-two'
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $stashPageTwoTabX, $stashTabCenterY, $false)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage 'hover-stash-page-two' -TimeoutSeconds 5 -ExpectedState "stash tab mouse=($stashPageTwoTabX,$stashTabCenterY)" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
                -ExpectedX $stashPageTwoTabX -ExpectedY $stashTabCenterY -Tolerance 3)
        }
        [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $stashPageTwoTabX, $stashTabCenterY)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState 'stash_page=2, stash_slot_present=false' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['stash_page'] -eq '2' -and
            $state['stash_slot_present'] -eq 'false'
        }
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $stashPageTwoTabX, $stashTabCenterY)
        $pageTwoState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage 'release-stash-page-two-tab' -TimeoutSeconds 5 -ExpectedState 'stash_page=2, left_pressed=false' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['stash_page'] -eq '2' -and
            $state['left_pressed'] -eq 'false'
        }
        $pageTwoState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'stash-page-two-state.txt')
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $stashPageTwoFramePath -Stage 'capture-stash-page-two-frame' -TimeoutSeconds 10

        $stage = 'return-to-stash-page-one'
        $stashPageOneTabX = [int][Math]::Round($stashTabStartX + 34 + 15)
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $stashPageOneTabX, $stashTabCenterY, $false)
        [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $stashPageOneTabX, $stashTabCenterY)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState 'stash_page=1, stash_slot_present=true' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['stash_page'] -eq '1' -and
            $state['stash_slot_present'] -eq 'true'
        }
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $stashPageOneTabX, $stashTabCenterY)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage 'release-stash-page-one-tab' -TimeoutSeconds 5 -ExpectedState 'stash_page=1, left_pressed=false' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['stash_page'] -eq '1' -and
            $state['left_pressed'] -eq 'false'
        }

        $stage = 'pick-stashed-card'
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $stashClientX, $stashClientY, $false)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage 'hover-stashed-card' -TimeoutSeconds 5 -ExpectedState "stash card mouse=($stashClientX,$stashClientY)" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['stash_slot_present'] -eq 'true' -and
            (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
                -ExpectedX $stashClientX -ExpectedY $stashClientY -Tolerance 3)
        }
        [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $stashClientX, $stashClientY)
        $mouseClientX = $stashClientX
        $mouseClientY = $stashClientY
        $mouseDown = $true
        $retrieveDragState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'dragging=true, zone=Table, stash_origin=0:0:0, stash_slot_present=false' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['zone'] -eq 'Table' -and
            $state['stash_origin'] -eq '0:0:0' -and
            $state['stash_slot_present'] -eq 'false' -and
            $state['stash_cursor_follow'] -eq 'true'
        }
        $retrieveDragState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'stash-retrieve-drag-state.txt')

        $stage = 'drag-stashed-card-to-table'
        for ($step = 1; $step -le 10; $step++) {
            $mouseClientX = [int][Math]::Round($stashClientX + ($startClientX - $stashClientX) * $step / 10.0)
            $mouseClientY = [int][Math]::Round($stashClientY + ($startClientY - $stashClientY) * $step / 10.0)
            [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $mouseClientX, $mouseClientY, $true)
            Start-Sleep -Milliseconds 35
        }
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState "dragging=true, stash_origin=0:0:0, rendered=(-160,130) tolerance=25" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['stash_origin'] -eq '0:0:0' -and
            (Test-Position -State $state -ExpectedX -160 -ExpectedY 130 -Tolerance 25)
        }

        $stage = 'release-stashed-card-to-table'
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $mouseClientX, $mouseClientY)
        $retrievedState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'dragging=false, zone=Table, stash_slot_present=false, rendered=(-160,130) tolerance=35' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'false' -and
            $state['left_pressed'] -eq 'false' -and
            $state['zone'] -eq 'Table' -and
            $state['stash_slot_present'] -eq 'false' -and
            $state['stash_origin'] -eq 'none' -and
            (Test-Position -State $state -ExpectedX -160 -ExpectedY 130 -Tolerance 35)
        }
        $mouseDown = $false
        $retrievedState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'stash-retrieved-state.txt')
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $stashRetrievedFramePath -Stage 'capture-stash-retrieved-frame' -TimeoutSeconds 10

        $stashRetrievedFrame = Read-UiSmokeBitmap -Path $stashRetrievedFramePath
        $retrievedStashChanges = Get-UiSmokeChangedPixels -Before $stashStoredFrame -After $stashRetrievedFrame `
            -CenterX $stashClientX -CenterY $stashClientY -HalfWidth 40 -HalfHeight 55 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        $retrievedTableChanges = Get-UiSmokeChangedPixels -Before $stashStoredFrame -After $stashRetrievedFrame `
            -CenterX $startClientX -CenterY $startClientY -HalfWidth 80 -HalfHeight 85 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        @(
            "frame_size=$($stashRetrievedFrame.Width)x$($stashRetrievedFrame.Height)"
            "rgb_delta_threshold=$rgbDeltaThreshold"
            "minimum_changed_pixels=$minimumStashChangedPixels"
            "stash_changed_pixels=$($retrievedStashChanges.ChangedPixels)"
            "table_changed_pixels=$($retrievedTableChanges.ChangedPixels)"
        ) | Set-Content -LiteralPath (Join-Path $artifactDir 'stash-retrieved-visual-diff.txt')
        if ($retrievedStashChanges.ChangedPixels -lt $minimumStashChangedPixels -or
            $retrievedTableChanges.ChangedPixels -lt $minimumStashChangedPixels) {
            throw "retrieved stash frame changed too few pixels: stash=$($retrievedStashChanges.ChangedPixels), table=$($retrievedTableChanges.ChangedPixels), minimum=$minimumStashChangedPixels"
        }

        $stage = 'verify-background-input'
        $lastInputTickAfterReleaseAck = [AxiomUiSmokeNative]::GetLastInputTick()
        $foregroundAfterInput = Get-UiSmokeForegroundSnapshot
        $cursorAfterInput = [AxiomUiSmokeNative]::GetCursorPosition()
        $cursorMovedDuringInput = $cursorAfterInput.X -ne $cursorBeforePostMessage.X -or
            $cursorAfterInput.Y -ne $cursorBeforePostMessage.Y
        $lastInputChangedDuringInput = $lastInputTickAfterReleaseAck -ne $lastInputTickBeforePostMessage
        $cursorStability = if (-not $cursorMovedDuringInput) {
            'unchanged'
        }
        elseif ($lastInputChangedDuringInput) {
            'inconclusive_external_input'
        }
        else {
            'moved_without_external_input'
        }
        @(
            "foreground_after_input=$($foregroundAfterInput.Handle)"
            "foreground_title_after_input=$($foregroundAfterInput.Title)"
            "foreground_process_id_after_input=$($foregroundAfterInput.ProcessId)"
            "cursor_after_input=($($cursorAfterInput.X),$($cursorAfterInput.Y))"
            "last_input_tick_after_release_ack=$lastInputTickAfterReleaseAck"
            "cursor_stability=$cursorStability"
        ) | Add-Content -LiteralPath $inputFile
        if ($foregroundAfterInput.Handle -eq $windowHandle) {
            throw "game window became foreground during stash round-trip (hwnd=$windowHandle pid=$($process.Id))"
        }
        if ($cursorMovedDuringInput -and -not $lastInputChangedDuringInput) {
            throw "stash round-trip PostMessageW input interval cursor drift without external input: before=($($cursorBeforePostMessage.X),$($cursorBeforePostMessage.Y)) after=($($cursorAfterInput.X),$($cursorAfterInput.Y)) last_input_tick_before=$lastInputTickBeforePostMessage last_input_tick_after=$lastInputTickAfterReleaseAck"
        }
        $succeeded = $true
    }
    elseif ($HandRoundTrip -or $ZoneTransition) {
        $stage = 'release-card-to-hand'
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $mouseClientX, $mouseClientY)
        $handState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState "dragging=false, holder=Hand(0), holder_occupied=true, zone=Hand(0), hand_count=1, zone_config=(physics=false, layer=UI, item_form=false), layout=($handWorldX,$handWorldY) tolerance=25" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'false' -and
            $state['left_pressed'] -eq 'false' -and
            $state['zone'] -eq 'Hand(0)' -and
            $state['hand_contains'] -eq 'true' -and
            $state['hand_count'] -eq '1' -and
            $state['holder'] -eq 'Hand(0)' -and
            $state['holder_occupied'] -eq 'true' -and
            $state['zone_has_physics'] -eq 'false' -and
            $state['zone_render_layer'] -eq 'UI' -and
            $state['zone_has_item_form'] -eq 'false' -and
            $state['face_up'] -eq 'true' -and
            (Test-Position -State $state -ExpectedX $handWorldX -ExpectedY $handWorldY -Tolerance 25)
        }
        $mouseDown = $false
        $handState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath $holderStatePath
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $holderFramePath -Stage 'capture-hand-frame' -TimeoutSeconds 10

        $stage = 'verify-hand-frame'
        $baselineFrame = Read-UiSmokeBitmap -Path $baselineFramePath
        $handFrame = Read-UiSmokeBitmap -Path $holderFramePath
        $rgbDeltaThreshold = 24
        $minimumHandChangedPixels = 500
        $handSourceChanges = Get-UiSmokeChangedPixels -Before $baselineFrame -After $handFrame `
            -CenterX $startClientX -CenterY $startClientY -HalfWidth 80 -HalfHeight 85 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        $handLayoutChanges = Get-UiSmokeChangedPixels -Before $baselineFrame -After $handFrame `
            -CenterX $handClientX -CenterY $handClientY -HalfWidth 100 -HalfHeight 145 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        @(
            "frame_size=$($handFrame.Width)x$($handFrame.Height)"
            "rgb_delta_threshold=$rgbDeltaThreshold"
            "minimum_changed_pixels=$minimumHandChangedPixels"
            "source_changed_pixels=$($handSourceChanges.ChangedPixels)"
            "hand_layout_changed_pixels=$($handLayoutChanges.ChangedPixels)"
        ) | Set-Content -LiteralPath (Join-Path $artifactDir $(if ($ZoneTransition) { 'zone-holder-visual-diff.txt' } else { 'hand-visual-diff.txt' }))
        if ($handSourceChanges.ChangedPixels -lt $minimumHandChangedPixels -or
            $handLayoutChanges.ChangedPixels -lt $minimumHandChangedPixels) {
            throw "hand frame changed too few pixels: source=$($handSourceChanges.ChangedPixels), hand=$($handLayoutChanges.ChangedPixels), minimum=$minimumHandChangedPixels"
        }

        $stage = 'hover-hand-card'
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $handClientX, $handClientY, $false)
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState "hand card mouse=($handClientX,$handClientY), holder=Hand(0), holder_occupied=true" -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['hand_contains'] -eq 'true' -and
            $state['hand_count'] -eq '1' -and
            $state['holder'] -eq 'Hand(0)' -and
            $state['holder_occupied'] -eq 'true' -and
            (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
                -ExpectedX $handClientX -ExpectedY $handClientY -Tolerance 3)
        }

        $stage = 'press-hand-card'
        [AxiomUiSmokeNative]::PostLeftButtonDown($windowHandle, $handClientX, $handClientY)
        $mouseClientX = $handClientX
        $mouseClientY = $handClientY
        $mouseDown = $true
        $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 5 -ExpectedState 'dragging=true, zone=Hand(0), holder=Hand(0), holder_occupied=false, hand_count=0' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['zone'] -eq 'Hand(0)' -and
            $state['hand_contains'] -eq 'false' -and
            $state['hand_count'] -eq '0' -and
            $state['holder'] -eq 'Hand(0)' -and
            $state['holder_occupied'] -eq 'false' -and
            $state['zone_has_physics'] -eq 'false' -and
            $state['zone_render_layer'] -eq 'UI' -and
            $state['zone_has_item_form'] -eq 'false'
        }

        $stage = 'drag-hand-card-to-table'
        for ($step = 1; $step -le 10; $step++) {
            $mouseClientX = [int][Math]::Round($handClientX + ($startClientX - $handClientX) * $step / 10.0)
            $mouseClientY = [int][Math]::Round($handClientY + ($startClientY - $handClientY) * $step / 10.0)
            [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $mouseClientX, $mouseClientY, $true)
            Start-Sleep -Milliseconds 35
        }
        $returnDragState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'dragging=true, zone=Hand(0), holder=Hand(0), holder_occupied=false, rendered=(-160,130) tolerance=25' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'true' -and
            $state['left_pressed'] -eq 'true' -and
            $state['zone'] -eq 'Hand(0)' -and
            $state['hand_contains'] -eq 'false' -and
            $state['holder'] -eq 'Hand(0)' -and
            $state['holder_occupied'] -eq 'false' -and
            (Test-Position -State $state -ExpectedX -160 -ExpectedY 130 -Tolerance 25)
        }
        $returnDragState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath (Join-Path $artifactDir 'return-drag-state.txt')

        $stage = 'release-card-to-table'
        [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $mouseClientX, $mouseClientY)
        $returnedState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
            -Stage $stage -TimeoutSeconds 10 -ExpectedState 'dragging=false, holder=none, holder_occupied=false, zone=Table, hand_count=0, zone_config=(physics=true, layer=World, item_form=false), rendered=(-160,130) tolerance=35' -Predicate {
            param($state)
            $state['scenario'] -eq $scenarioName -and
            $state['dragging'] -eq 'false' -and
            $state['left_pressed'] -eq 'false' -and
            $state['zone'] -eq 'Table' -and
            $state['hand_contains'] -eq 'false' -and
            $state['hand_count'] -eq '0' -and
            $state['holder'] -eq 'none' -and
            $state['holder_occupied'] -eq 'false' -and
            $state['zone_has_physics'] -eq 'true' -and
            $state['zone_render_layer'] -eq 'World' -and
            $state['zone_has_item_form'] -eq 'false' -and
            (Test-Position -State $state -ExpectedX -160 -ExpectedY 130 -Tolerance 35)
        }
        $mouseDown = $false
        Start-Sleep -Milliseconds 250
        $returnedState = Read-UiState -Path $stateFile
        if ($null -eq $returnedState -or
            $returnedState['zone'] -ne 'Table' -or
            $returnedState['hand_contains'] -ne 'false' -or
            $returnedState['holder'] -ne 'none' -or
            $returnedState['holder_occupied'] -ne 'false' -or
            $returnedState['zone_has_physics'] -ne 'true' -or
            $returnedState['zone_render_layer'] -ne 'World' -or
            $returnedState['zone_has_item_form'] -ne 'false' -or
            -not (Test-Position -State $returnedState -ExpectedX -160 -ExpectedY 130 -Tolerance 35)) {
            throw "returned card left the expected table state: observed=$(Format-UiStateDiagnostic -State $returnedState)"
        }
        $returnedState.GetEnumerator() | ForEach-Object {
            '{0}={1}' -f $_.Key, $_.Value
        } | Set-Content -LiteralPath $returnedStatePath
        Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
            -CapturePath $returnedHolderFramePath -Stage 'capture-returned-frame' -TimeoutSeconds 10

        $stage = 'verify-returned-frame'
        $returnedFrame = Read-UiSmokeBitmap -Path $returnedHolderFramePath
        $returnedHandChanges = Get-UiSmokeChangedPixels -Before $handFrame -After $returnedFrame `
            -CenterX $handClientX -CenterY $handClientY -HalfWidth 100 -HalfHeight 145 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        $returnedTableChanges = Get-UiSmokeChangedPixels -Before $handFrame -After $returnedFrame `
            -CenterX $startClientX -CenterY $startClientY -HalfWidth 80 -HalfHeight 85 `
            -RgbDeltaThreshold $rgbDeltaThreshold
        @(
            "frame_size=$($returnedFrame.Width)x$($returnedFrame.Height)"
            "rgb_delta_threshold=$rgbDeltaThreshold"
            "minimum_changed_pixels=$minimumHandChangedPixels"
            "hand_changed_pixels=$($returnedHandChanges.ChangedPixels)"
            "table_changed_pixels=$($returnedTableChanges.ChangedPixels)"
        ) | Set-Content -LiteralPath (Join-Path $artifactDir $(if ($ZoneTransition) { 'zone-returned-visual-diff.txt' } else { 'returned-visual-diff.txt' }))
        if ($returnedHandChanges.ChangedPixels -lt $minimumHandChangedPixels -or
            $returnedTableChanges.ChangedPixels -lt $minimumHandChangedPixels) {
            throw "returned frame changed too few pixels: hand=$($returnedHandChanges.ChangedPixels), table=$($returnedTableChanges.ChangedPixels), minimum=$minimumHandChangedPixels"
        }

        $stage = 'verify-background-input'
        $lastInputTickAfterReleaseAck = [AxiomUiSmokeNative]::GetLastInputTick()
        $foregroundAfterInput = Get-UiSmokeForegroundSnapshot
        $cursorAfterInput = [AxiomUiSmokeNative]::GetCursorPosition()
        $cursorMovedDuringInput = $cursorAfterInput.X -ne $cursorBeforePostMessage.X -or
            $cursorAfterInput.Y -ne $cursorBeforePostMessage.Y
        $lastInputChangedDuringInput = $lastInputTickAfterReleaseAck -ne $lastInputTickBeforePostMessage
        $cursorStability = if (-not $cursorMovedDuringInput) {
            'unchanged'
        }
        elseif ($lastInputChangedDuringInput) {
            'inconclusive_external_input'
        }
        else {
            'moved_without_external_input'
        }
        @(
            "foreground_after_input=$($foregroundAfterInput.Handle)"
            "foreground_title_after_input=$($foregroundAfterInput.Title)"
            "foreground_process_id_after_input=$($foregroundAfterInput.ProcessId)"
            "cursor_after_input=($($cursorAfterInput.X),$($cursorAfterInput.Y))"
            "last_input_tick_after_release_ack=$lastInputTickAfterReleaseAck"
            "cursor_stability=$cursorStability"
        ) | Add-Content -LiteralPath $inputFile
        if ($foregroundAfterInput.Handle -eq $windowHandle) {
            $inputDescription = if ($ZoneTransition) { 'zone transition' } else { 'hand round-trip' }
            throw "game window became foreground during $inputDescription (hwnd=$windowHandle pid=$($process.Id))"
        }
        if ($cursorMovedDuringInput -and -not $lastInputChangedDuringInput) {
            throw "hand round-trip PostMessageW input interval cursor drift without external input: before=($($cursorBeforePostMessage.X),$($cursorBeforePostMessage.Y)) after=($($cursorAfterInput.X),$($cursorAfterInput.Y)) last_input_tick_before=$lastInputTickBeforePostMessage last_input_tick_after=$lastInputTickAfterReleaseAck"
        }
        $succeeded = $true
    }
    else {
    $stage = 'verify-rendered-card-move'
    $baselineFrame = Read-UiSmokeBitmap -Path $baselineFramePath
    $draggedFrame = Read-UiSmokeBitmap -Path $framePath
    if ($baselineFrame.Width -ne $client.Width -or $baselineFrame.Height -ne $client.Height) {
        throw "frame size $($baselineFrame.Width)x$($baselineFrame.Height) does not match client area $($client.Width)x$($client.Height)"
    }
    $rgbDeltaThreshold = 24
    $minimumChangedPixels = 500
    $sourceChanges = Get-UiSmokeChangedPixels -Before $baselineFrame -After $draggedFrame `
        -CenterX $startClientX -CenterY $startClientY -RgbDeltaThreshold $rgbDeltaThreshold
    $targetChanges = Get-UiSmokeChangedPixels -Before $baselineFrame -After $draggedFrame `
        -CenterX $targetClientX -CenterY $targetClientY -RgbDeltaThreshold $rgbDeltaThreshold
    $cardTemplate = New-UiSmokeCardTemplate -Frame $baselineFrame -CenterX $startClientX -CenterY $startClientY
    $templateRgbDeltaMaximum = 80
    $templateSearchRadius = 28
    $minimumTargetMatchPercent = 75
    $maximumSourceMatchPercent = 20
    $minimumTargetMatchPixels = [int][Math]::Ceiling($cardTemplate.PixelCount * $minimumTargetMatchPercent / 100.0)
    $maximumSourceMatchPixels = [int][Math]::Floor($cardTemplate.PixelCount * $maximumSourceMatchPercent / 100.0)
    $targetTemplateMatch = Find-UiSmokeCardTemplate -Frame $draggedFrame -Template $cardTemplate `
        -ExpectedCenterX $targetClientX -ExpectedCenterY $targetClientY `
        -SearchRadius $templateSearchRadius -RgbDeltaMaximum $templateRgbDeltaMaximum
    $sourceTemplateMatch = Find-UiSmokeCardTemplate -Frame $draggedFrame -Template $cardTemplate `
        -ExpectedCenterX $startClientX -ExpectedCenterY $startClientY `
        -SearchRadius $templateSearchRadius -RgbDeltaMaximum $templateRgbDeltaMaximum
    @(
        "frame_size=$($baselineFrame.Width)x$($baselineFrame.Height)"
        "rgb_delta_threshold=$rgbDeltaThreshold"
        "minimum_changed_pixels_per_roi=$minimumChangedPixels"
        "source_roi=$($sourceChanges.Left),$($sourceChanges.Top),$($sourceChanges.Width),$($sourceChanges.Height)"
        "source_changed_pixels=$($sourceChanges.ChangedPixels)"
        "target_roi=$($targetChanges.Left),$($targetChanges.Top),$($targetChanges.Width),$($targetChanges.Height)"
        "target_changed_pixels=$($targetChanges.ChangedPixels)"
        "template_size=$($cardTemplate.Width)x$($cardTemplate.Height)"
        "template_unique_rgb_colors=$($cardTemplate.UniqueColors)"
        "template_rgb_delta_maximum=$templateRgbDeltaMaximum"
        "template_search_radius=$templateSearchRadius"
        "target_template_match=$($targetTemplateMatch.MatchedPixels)/$($targetTemplateMatch.PixelCount)"
        "target_template_minimum_match_percent=$minimumTargetMatchPercent"
        "target_template_offset=$($targetTemplateMatch.OffsetX),$($targetTemplateMatch.OffsetY)"
        "source_template_best_match=$($sourceTemplateMatch.MatchedPixels)/$($sourceTemplateMatch.PixelCount)"
        "source_template_maximum_match_percent_exclusive=$maximumSourceMatchPercent"
        "source_template_offset=$($sourceTemplateMatch.OffsetX),$($sourceTemplateMatch.OffsetY)"
    ) | Set-Content -LiteralPath (Join-Path $artifactDir 'visual-diff.txt')
    if ($sourceChanges.ChangedPixels -lt $minimumChangedPixels -or
        $targetChanges.ChangedPixels -lt $minimumChangedPixels) {
        throw "rendered card move changed too few pixels: source=$($sourceChanges.ChangedPixels), target=$($targetChanges.ChangedPixels), minimum=$minimumChangedPixels"
    }
    if ($targetTemplateMatch.MatchedPixels -lt $minimumTargetMatchPixels -or
        $sourceTemplateMatch.MatchedPixels -ge $maximumSourceMatchPixels) {
        throw "rendered card template verification failed: target=$($targetTemplateMatch.MatchedPixels)/$($targetTemplateMatch.PixelCount) required>=$minimumTargetMatchPixels, source=$($sourceTemplateMatch.MatchedPixels)/$($sourceTemplateMatch.PixelCount) required<$maximumSourceMatchPixels"
    }

    $stage = 'release-card'
    [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $mouseClientX, $mouseClientY)
    $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
        -Stage $stage -TimeoutSeconds 10 -ExpectedState 'dragging=false, left_pressed=false, zone=Table' -Predicate {
        param($state)
        $state['scenario'] -eq $scenarioName -and
        $state['dragging'] -eq 'false' -and
        $state['left_pressed'] -eq 'false' -and
        $state['zone'] -eq 'Table'
    }
    $mouseDown = $false

    $stage = 'verify-background-input'
    $lastInputTickAfterReleaseAck = [AxiomUiSmokeNative]::GetLastInputTick()
    $foregroundAfterInput = Get-UiSmokeForegroundSnapshot
    $cursorAfterInput = [AxiomUiSmokeNative]::GetCursorPosition()
    $cursorMovedDuringInput = $cursorAfterInput.X -ne $cursorBeforePostMessage.X -or
        $cursorAfterInput.Y -ne $cursorBeforePostMessage.Y
    $lastInputChangedDuringInput = $lastInputTickAfterReleaseAck -ne $lastInputTickBeforePostMessage
    $cursorStability = if (-not $cursorMovedDuringInput) {
        'unchanged'
    }
    elseif ($lastInputChangedDuringInput) {
        'inconclusive_external_input'
    }
    else {
        'moved_without_external_input'
    }
    @(
        "foreground_after_input=$($foregroundAfterInput.Handle)"
        "foreground_title_after_input=$($foregroundAfterInput.Title)"
        "foreground_process_id_after_input=$($foregroundAfterInput.ProcessId)"
        "cursor_after_input=($($cursorAfterInput.X),$($cursorAfterInput.Y))"
        "last_input_tick_after_release_ack=$lastInputTickAfterReleaseAck"
        "cursor_stability=$cursorStability"
    ) | Add-Content -LiteralPath $inputFile
    if ($foregroundAfterInput.Handle -eq $windowHandle) {
        throw "game window became foreground during input (hwnd=$windowHandle pid=$($process.Id))"
    }
    if ($cursorMovedDuringInput -and -not $lastInputChangedDuringInput) {
        throw "PostMessageW input interval cursor drift without external input: before=($($cursorBeforePostMessage.X),$($cursorBeforePostMessage.Y)) after_release_ack=($($cursorAfterInput.X),$($cursorAfterInput.Y)) last_input_tick_before=$lastInputTickBeforePostMessage last_input_tick_after=$lastInputTickAfterReleaseAck"
    }
    $succeeded = $true
    }
}
}
catch {
    [Console]::Error.WriteLine("UI smoke failed: scenario=$scenarioName stage=${stage} $($_.Exception.Message)")
    if (Test-Path -LiteralPath $stateFile) {
        $failureState = Read-UiState -Path $stateFile
        if ($null -ne $failureState) {
            $failureState.GetEnumerator() | ForEach-Object {
                '{0}={1}' -f $_.Key, $_.Value
            } | Set-Content -LiteralPath (Join-Path $artifactDir 'failure-state.txt')
        }
    }
    if (Test-Path -LiteralPath $inputFile) {
        [Console]::Error.WriteLine((Get-Content -LiteralPath $inputFile -Raw))
    }
}
finally {
    if (-not $succeeded -and $windowHandle -ne [IntPtr]::Zero) {
        try {
            Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
                -CapturePath $failureFramePath -Stage 'failure-frame-capture' -TimeoutSeconds 10
        }
        catch {
            $failureCaptureError = $_.Exception.Message
            [Console]::Error.WriteLine("UI smoke failure: scenario=$scenarioName stage=failure-frame-capture $failureCaptureError")
        }
    }
    if ($rightMouseDown -and $windowHandle -ne [IntPtr]::Zero) {
        try {
            [AxiomUiSmokeNative]::PostRightButtonUp($windowHandle, $mouseClientX, $mouseClientY)
            $rightMouseDown = $false
        }
        catch {
            $cleanupFailure = "cleanup right-button release failed: $($_.Exception.Message)"
        }
    }
    if ($mouseDown) {
        try {
            $null = Read-UiState -Path $stateFile
            [AxiomUiSmokeNative]::PostLeftButtonUp($windowHandle, $mouseClientX, $mouseClientY)
            if ($leftPressObserved) {
                $cleanupReleaseState = Wait-UiState -Process $process -StateFile $stateFile `
                    -Scenario $scenarioName -Stage 'cleanup-release' -TimeoutSeconds 3 `
                    -ExpectedState 'dragging=false, left_pressed=false' -Predicate {
                        param($state)
                        $state['dragging'] -eq 'false' -and $state['left_pressed'] -eq 'false'
                    }
                $cleanupReleaseState.GetEnumerator() | ForEach-Object {
                    '{0}={1}' -f $_.Key, $_.Value
                } | Set-Content -LiteralPath (Join-Path $artifactDir 'cleanup-release-state.txt')
                $mouseDown = $false
            }
            else {
                $cleanupFailure = 'button-up was posted, but left_pressed=true was never observed; release acknowledgment unavailable'
                $cleanupFailure | Set-Content -LiteralPath (Join-Path $artifactDir 'cleanup-release.error.txt')
            }
        }
        catch {
            $cleanupFailure = "cleanup release was not acknowledged: $($_.Exception.Message)"
            $cleanupFailure | Set-Content -LiteralPath (Join-Path $artifactDir 'cleanup-release.error.txt')
        }
    }
    if ($null -ne $process) {
        try {
            $process.Refresh()
            if (-not $process.HasExited) {
                Stop-Process -Id $process.Id -Force
                if (-not $process.WaitForExit(5000)) {
                    $cleanupFailure = "game process $($process.Id) did not exit within 5 seconds"
                }
            }
        }
        catch {
            $cleanupFailure = "game process cleanup failed: $($_.Exception.Message)"
        }
    }
    if ($previousDpiContext -ne [IntPtr]::Zero) {
        try { [AxiomUiSmokeNative]::RestoreDpiContext($previousDpiContext) } catch { }
    }
}

if ($null -ne $cleanupFailure) {
    $succeeded = $false
    [Console]::Error.WriteLine("UI smoke failed: scenario=$scenarioName stage=cleanup $cleanupFailure")
}

if (-not $succeeded) {
    foreach ($logPath in @($stdoutPath, $stderrPath)) {
        if (Test-Path -LiteralPath $logPath) {
            $logTail = Get-Content -LiteralPath $logPath -Tail 20
            [Console]::Error.WriteLine(($logTail -join [Environment]::NewLine))
        }
    }
    [Console]::Error.WriteLine("Evidence retained at $artifactDir")
    exit 1
}

if ($ArtFace) {
    Write-Output ("UI art-face scenario passed: position=({0},{1}), art_state={2}, art_frame={3}, visual_evidence={4}" -f `
        $artFaceState['rendered_x'], $artFaceState['rendered_y'], `
        (Join-Path $artifactDir 'art-face-state.txt'), $artFaceFramePath, `
        (Join-Path $artifactDir 'art-face-visual.txt'))
    Write-Output ("Art observed: signature={0}, element={1}, aspect={2}, shapes={3}, unique_colors={4}, non_background_pixels={5}/{6}" -f `
        $artFaceState['art_signature'], $artFaceState['art_element'], $artFaceState['art_aspect'], `
        $artFaceState['art_shape_count'], $artEvidence.UniqueColors, $artEvidence.NonBackgroundPixels, $artEvidence.PixelCount)
}
elseif ($IdentitySignature) {
    Write-Output ("UI identity scenario passed: position=({0},{1}), identity_state={2}, identity_frame={3}, visual_evidence={4}" -f `
        $identityState['rendered_x'], $identityState['rendered_y'], `
        (Join-Path $artifactDir 'identity-state.txt'), $identityFramePath, `
        (Join-Path $artifactDir 'identity-visual.txt'))
    Write-Output ("Identity observed: signature={0}, seed={1}, rarity={2}, tier={3}, name={4}" -f `
        $identityState['identity_signature'], $identityState['identity_seed'], `
        $identityState['identity_rarity'], $identityState['identity_tier'], $identityState['identity_name'])
}
elseif ($BoosterOpening) {
    Write-Output ("UI booster opening passed: sealed=({0},{1}), opened=({2},{3}), sealed_state={4}, opening_state={5}, opened_state={6}, opening_frame={7}, opened_frame={8}" -f `
        $boosterWorldX, $boosterWorldY, $openedState['opened_card_x'], $openedState['opened_card_y'], `
        (Join-Path $artifactDir 'booster-sealed-state.txt'), (Join-Path $artifactDir 'booster-opening-state.txt'), `
        (Join-Path $artifactDir 'booster-opened-state.txt'), $boosterOpeningFramePath, $boosterOpenedFramePath)
    Write-Output ("Booster flow verified: expected_seed={0}, opened_seed={1}, sealed_origin_changed={2}, opening_center_changed={3}, opened_center_changed={4} pixels (minimum 250 at RGB delta 24)" -f `
        $openedState['expected_card_seed'], $openedState['opened_card_seed'], $sealedOriginChanges.ChangedPixels, `
        $openingCenterChanges.ChangedPixels, $openedCenterChanges.ChangedPixels)
}
elseif ($StashRoundTrip) {
    Write-Output ("UI stash round-trip passed: stored=({0},{1}) retrieved=({2},{3}), stash_state={4}, page_two_state={5}, retrieved_state={6}, stored_frame={7}, retrieved_frame={8}" -f `
        $stashState['rendered_x'], $stashState['rendered_y'], $retrievedState['rendered_x'], $retrievedState['rendered_y'], `
        (Join-Path $artifactDir 'stash-state.txt'), (Join-Path $artifactDir 'stash-page-two-state.txt'), `
        (Join-Path $artifactDir 'stash-retrieved-state.txt'), $stashStoredFramePath, $stashRetrievedFramePath)
    Write-Output ("Stash workflow verified: preview_changed={0}, stored_changed={1}, page_two={2}, retrieved_stash_changed={3}, retrieved_table_changed={4} pixels (minimum 250 at RGB delta 24)" -f `
        $stashPreviewChanges.ChangedPixels, $storedPreviewChanges.ChangedPixels, $pageTwoState['stash_page'], `
        $retrievedStashChanges.ChangedPixels, $retrievedTableChanges.ChangedPixels)
}
elseif ($ZoneTransition) {
    Write-Output ("UI zone transition passed: holder=({0},{1}) returned=({2},{3}), holder_state={4}, returned_state={5}, holder_frame={6}, returned_frame={7}" -f `
        $handState['rendered_x'], $handState['rendered_y'], $returnedState['rendered_x'], $returnedState['rendered_y'], `
        $holderStatePath, $returnedStatePath, $holderFramePath, $returnedHolderFramePath)
    Write-Output ("Zone contract verified: holder={0}, occupied={1}, holder_config=({2},{3},{4}), returned_zone={5}, returned_config=({6},{7},{8}), holder_changed={9}, returned_table_changed={10} pixels" -f `
        $handState['holder'], $handState['holder_occupied'], $handState['zone_has_physics'],
        $handState['zone_render_layer'], $handState['zone_has_item_form'], $returnedState['zone'],
        $returnedState['zone_has_physics'], $returnedState['zone_render_layer'], $returnedState['zone_has_item_form'],
        $handLayoutChanges.ChangedPixels, $returnedTableChanges.ChangedPixels)
}
elseif ($ReaderRoundTrip) {
    Write-Output ("UI reader round-trip passed: inserted=({0},{1}), returned=({2},{3}), inserted_state={4}, ejected_state={5}, returned_state={6}, inserted_frame={7}, ejected_frame={8}, returned_frame={9}" -f `
        $insertedState['rendered_x'], $insertedState['rendered_y'], $returnedState['rendered_x'], $returnedState['rendered_y'],
        (Join-Path $artifactDir 'reader-inserted-state.txt'), (Join-Path $artifactDir 'reader-ejected-state.txt'),
        $returnedStatePath, $readerInsertedFramePath, $readerEjectedFramePath, $readerReturnedFramePath)
    Write-Output ("Reader contract verified: signature={0}, radius={1}, inserted_feedback={2}, ejected_feedback={3}, inserted_to_ejected_reader_changed={4}, inserted_to_returned_reader_changed={5}, ejected_to_returned_table_changed={6} pixels" -f `
        $insertedState['reader_signature'], $insertedState['reader_space_radius'], $insertedState['reader_feedback'],
        $returnedState['reader_feedback'], $ejectedReaderChanges.ChangedPixels, $returnedReaderChanges.ChangedPixels,
        $returnedTableChanges.ChangedPixels)
}
elseif ($CableWrapping) {
    Write-Output ("UI cable wrapping scenario passed: source=({0},{1}), anchor=({2},{3}), dest=({4},{5}), state={6}, frame={7}, visual_evidence={8}" -f `
        $cableState['cable_source_x'], $cableState['cable_source_y'], `
        $cableState['cable_anchor_0_x'], $cableState['cable_anchor_0_y'], `
        $cableState['cable_dest_x'], $cableState['cable_dest_y'], `
        (Join-Path $artifactDir 'cable-wrapping-state.txt'), $cableWrappingFramePath, `
        (Join-Path $artifactDir 'cable-wrapping-visual-diff.txt'))
    Write-Output ("Cable path verified: connected={0}, anchors={1}, rendered_vertices={2}, changed_pixels={3} (minimum {4} at RGB delta {5})" -f `
        $cableState['cable_connected'], $cableState['cable_anchor_count'], `
        $cableState['cable_rendered_vertex_count'], $cableChanges.ChangedPixels, `
        $minimumCableChangedPixels, $rgbDeltaThreshold)
}
elseif ($CombinerProcessing) {
    Write-Output ("UI combiner scenario passed: primary_reader=({0},{1}), secondary_reader=({2},{3}), combiner_state={4}, combiner_frame={5}, visual_evidence={6}" -f `
        $insertedPrimaryState['rendered_x'], $insertedPrimaryState['rendered_y'],
        $insertedSecondaryState['second_rendered_x'], $insertedSecondaryState['second_rendered_y'],
        (Join-Path $artifactDir 'combiner-state.txt'), $combinerFramePath,
        (Join-Path $artifactDir 'combiner-visual-diff.txt'))
    Write-Output ("Combiner contract verified: input_a={0}, input_b={1}, sources=({2},{3}), output_sources={4}, output_points={5}, contains=({6},{7}), changed_pixels={8} (minimum 500 at RGB delta 24)" -f `
        $combinedState['combiner_input_a_connected'], $combinedState['combiner_input_b_connected'],
        $combinedState['combiner_input_a_source_count'], $combinedState['combiner_input_b_source_count'],
        $combinedState['combiner_output_source_count'], $combinedState['combiner_output_control_points'],
        $combinedState['combiner_output_contains_primary'], $combinedState['combiner_output_contains_secondary'],
        $combinerChanges.ChangedPixels)
}
elseif ($HandRoundTrip) {
    Write-Output ("UI hand round-trip passed: hand=({0},{1}) returned=({2},{3}), hand_state={4}, returned_state={5}, hand_frame={6}, returned_frame={7}" -f `
        $handState['rendered_x'], $handState['rendered_y'], $returnedState['rendered_x'], $returnedState['rendered_y'], `
        $holderStatePath, $returnedStatePath, $holderFramePath, $returnedHolderFramePath)
    Write-Output ("Hand layout verified: hand_contains={0}, hand_count={1}, hand_changed={2}, returned_hand_changed={3}, returned_table_changed={4} pixels (minimum 500 at RGB delta 24)" -f `
        $handState['hand_contains'], $handState['hand_count'], $handLayoutChanges.ChangedPixels, `
        $returnedHandChanges.ChangedPixels, $returnedTableChanges.ChangedPixels)
}
elseif (-not $IdentitySignature -and -not $Interaction) {
    Write-Output ("UI smoke passed: scenario=$scenarioName rendered=({0},{1}) frame={2}" -f `
        $dragState['rendered_x'], $dragState['rendered_y'], $framePath)
    Write-Output ("Visual move verified: source_changed={0} target_changed={1} pixels (minimum 500 at RGB delta 24); baseline={2}" -f `
        $sourceChanges.ChangedPixels, $targetChanges.ChangedPixels, $baselineFramePath)
    Write-Output ("Card template verified: target_match={0}/{1}, source_match={2}/{1}, target_offset=({3},{4})" -f `
        $targetTemplateMatch.MatchedPixels, $targetTemplateMatch.PixelCount, $sourceTemplateMatch.MatchedPixels, `
        $targetTemplateMatch.OffsetX, $targetTemplateMatch.OffsetY)
}
else {
    Write-Output ("UI interaction passed: final=({0},{1}), rotation={2}, face_up={3}, frame={4}, released_frame={5}, flipped_frame={6}" -f `
        $flippedState['rendered_x'], $flippedState['rendered_y'], $flippedState['rotation'], `
        $flippedState['face_up'], $framePath, $releasedFramePath, $flippedFramePath)
    Write-Output ("Interaction verified: spin_rotation={0}, moved_pixels=source:{1} target:{2}, flip_changed={3} (minimum 500 at RGB delta 24)" -f `
        $spinState['rotation'], $sourceChanges.ChangedPixels, $targetChanges.ChangedPixels, $flipChanges.ChangedPixels)
}
if ($IdentitySignature -or $ArtFace) {
    $diagnosticMode = if ($ArtFace) { 'art-face-no-postmessagew' } else { 'identity-no-postmessagew' }
    Write-Output ("Input diagnostics: mode=$diagnosticMode, before-PostMessageW=$identityNoInput, before-PostMessageW-pid=$identityNoInput, cursor-before-PostMessageW=$identityNoInput, last-input-tick-before-PostMessageW=$identityNoInput, cursor-stability=$cursorStability")
}
else {
    Write-Output ("Cursor samples: launch=({0},{1}), preparation=({2},{3}), pre-input=({4},{5}), before-PostMessageW=({6},{7}), after-release-ack=({8},{9}), setup-drift={10}, cursor-stability={11}, last-input-ticks={12}->{13}" -f `
        $previousCursor.X, $previousCursor.Y, $cursorAtPreparation.X, $cursorAtPreparation.Y, `
        $cursorBeforeInputSetup.X, $cursorBeforeInputSetup.Y, $cursorBeforePostMessage.X, $cursorBeforePostMessage.Y, `
        $cursorAfterInput.X, $cursorAfterInput.Y, $cursorSetupDrift, $cursorStability, `
        $lastInputTickBeforePostMessage, $lastInputTickAfterReleaseAck)
    Write-Output ("Foreground samples: launch={0} pid={1}, pre-PostMessageW={2} pid={3}, post-release={4} pid={5}" -f `
        $foregroundBeforeLaunch.Handle, $foregroundBeforeLaunch.ProcessId, $foregroundBeforePostMessage.Handle, `
        $foregroundBeforePostMessage.ProcessId, $foregroundAfterInput.Handle, $foregroundAfterInput.ProcessId)
}
