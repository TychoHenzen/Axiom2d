param(
    [switch]$RunnerChild,
    [string]$RunnerArtifactDirectory,
    [switch]$Interaction
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
$failureFramePath = Join-Path $artifactDir 'failure.bmp'
$frameCaptureRequestFile = Join-Path $artifactDir 'frame-capture.request'
$scenarioName = if ($Interaction) { 'seeded-card-interaction' } else { 'seeded-card-drag' }
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
    return ("scenario={0}, dragging={1}, zone={2}, rendered=({3},{4}), rotation={5}, face_up={6}, mouse=({7},{8}), left_pressed={9}, right_pressed={10}" -f `
        $State['scenario'], $State['dragging'], $State['zone'], $State['rendered_x'], `
        $State['rendered_y'], $State['rotation'], $State['face_up'], $State['mouse_x'], `
        $State['mouse_y'], $State['left_pressed'], $State['right_pressed'])
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
    $null = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
        -Stage $stage -TimeoutSeconds 30 -ExpectedState 'dragging=false, zone=Table, rendered=(-160,130) tolerance=1' -Predicate {
        param($state)
        $state['scenario'] -eq $scenarioName -and
        $state['dragging'] -eq 'false' -and
        $state['zone'] -eq 'Table' -and
        (Test-Position -State $state -ExpectedX -160 -ExpectedY 130 -Tolerance 1)
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

    $startScreenX = $client.Left + [int][Math]::Round($client.Width / 2.0 - 160)
    $startScreenY = $client.Top + [int][Math]::Round($client.Height / 2.0 + 130)
    $targetScreenX = $client.Left + [int][Math]::Round($client.Width / 2.0 - 300)
    $targetScreenY = $client.Top + [int][Math]::Round($client.Height / 2.0 - 150)
    $startClientX = [int][Math]::Round($client.Width / 2.0 - 160)
    $startClientY = [int][Math]::Round($client.Height / 2.0 + 130)
    $targetClientX = [int][Math]::Round($client.Width / 2.0 - 300)
    $targetClientY = [int][Math]::Round($client.Height / 2.0 - 150)
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
    ) | Set-Content -LiteralPath $inputFile
    if ($foregroundAtPreparation.Handle -eq $windowHandle) {
        throw "game window became foreground during preparation (hwnd=$windowHandle pid=$($process.Id))"
    }
    if ($foregroundBeforeInput.Handle -eq $windowHandle) {
        throw "game window was foreground before input (hwnd=$windowHandle pid=$($process.Id))"
    }

    if ($Interaction) {
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
    else {
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

    $stage = 'drag-card'
    for ($step = 1; $step -le 10; $step++) {
        $mouseClientX = [int][Math]::Round($startClientX + ($targetClientX - $startClientX) * $step / 10.0)
        $mouseClientY = [int][Math]::Round($startClientY + ($targetClientY - $startClientY) * $step / 10.0)
        [AxiomUiSmokeNative]::PostMouseMove($windowHandle, $mouseClientX, $mouseClientY, $true)
        Start-Sleep -Milliseconds 35
    }
    $dragState = Wait-UiState -Process $process -StateFile $stateFile -Scenario $scenarioName `
        -Stage $stage -TimeoutSeconds 10 -ExpectedState 'dragging=true, left_pressed=true, zone=Table, rendered=(-300,-150) tolerance=25' -Predicate {
        param($state)
        $state['scenario'] -eq $scenarioName -and
        $state['dragging'] -eq 'true' -and
        $state['left_pressed'] -eq 'true' -and
        $state['zone'] -eq 'Table' -and
        (Test-Position -State $state -ExpectedX -300 -ExpectedY -150 -Tolerance 25)
    }

    $stage = 'capture-dragged-frame'
    $dragState.GetEnumerator() | ForEach-Object {
        '{0}={1}' -f $_.Key, $_.Value
    } | Set-Content -LiteralPath (Join-Path $artifactDir 'drag-state.txt')
    Request-GameFrameCapture -Process $process -RequestFile $frameCaptureRequestFile `
        -CapturePath $framePath -Stage $stage -TimeoutSeconds 10

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

if (-not $Interaction) {
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
Write-Output ("Cursor samples: launch=({0},{1}), preparation=({2},{3}), pre-input=({4},{5}), before-PostMessageW=({6},{7}), after-release-ack=({8},{9}), setup-drift={10}, cursor-stability={11}, last-input-ticks={12}->{13}" -f `
    $previousCursor.X, $previousCursor.Y, $cursorAtPreparation.X, $cursorAtPreparation.Y, `
    $cursorBeforeInputSetup.X, $cursorBeforeInputSetup.Y, $cursorBeforePostMessage.X, $cursorBeforePostMessage.Y, `
    $cursorAfterInput.X, $cursorAfterInput.Y, $cursorSetupDrift, $cursorStability, `
    $lastInputTickBeforePostMessage, $lastInputTickAfterReleaseAck)
Write-Output ("Foreground samples: launch={0} pid={1}, pre-PostMessageW={2} pid={3}, post-release={4} pid={5}" -f `
    $foregroundBeforeLaunch.Handle, $foregroundBeforeLaunch.ProcessId, $foregroundBeforePostMessage.Handle, `
    $foregroundBeforePostMessage.ProcessId, $foregroundAfterInput.Handle, $foregroundAfterInput.ProcessId)
