$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$artifactDir = Join-Path $repoRoot (Join-Path 'target' ("ui-smoke-{0}" -f [guid]::NewGuid().ToString('N')))
$stateFile = Join-Path $artifactDir 'state.txt'
$inputFile = Join-Path $artifactDir 'input.txt'
$stdoutPath = Join-Path $artifactDir 'app.stdout.log'
$stderrPath = Join-Path $artifactDir 'app.stderr.log'
$framePath = Join-Path $artifactDir 'frame.bmp'
$process = $null
$windowHandle = [IntPtr]::Zero
$previousForeground = $null
$previousCursor = $null
$previousDpiContext = [IntPtr]::Zero
$mouseDown = $false
$succeeded = $false
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

function Wait-UiState {
    param(
        [System.Diagnostics.Process]$Process,
        [string]$StateFile,
        [string]$Stage,
        [int]$TimeoutSeconds,
        [scriptblock]$Predicate
    )

    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    $lastState = $null
    while ($watch.Elapsed.TotalSeconds -lt $TimeoutSeconds) {
        $Process.Refresh()
        if ($Process.HasExited) {
            throw "stage=${Stage}: game process exited with code $($Process.ExitCode)"
        }
        $lastState = Read-UiState -Path $StateFile
        if ($null -ne $lastState -and (& $Predicate $lastState)) {
            return $lastState
        }
        Start-Sleep -Milliseconds 200
    }

    if ($null -eq $lastState) {
        throw "stage=${Stage}: no complete state snapshot within $TimeoutSeconds seconds"
    }
    throw ("stage={0}: timed out after {1}s; observed dragging={2}, zone={3}, rendered=({4},{5}), mouse=({6},{7}), left_pressed={8}" -f `
        $Stage, $TimeoutSeconds, $lastState['dragging'], $lastState['zone'], `
        $lastState['rendered_x'], $lastState['rendered_y'], $lastState['mouse_x'], `
        $lastState['mouse_y'], $lastState['left_pressed'])
}

try {
    Add-Type -TypeDefinition @'
using System;
using System.IO;
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
    private struct MouseInput
    {
        public int Dx;
        public int Dy;
        public uint MouseData;
        public uint Flags;
        public uint Time;
        public UIntPtr ExtraInfo;
    }

    [StructLayout(LayoutKind.Explicit)]
    private struct InputUnion
    {
        [FieldOffset(0)]
        public MouseInput Mouse;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct Input
    {
        public uint Type;
        public InputUnion Union;
    }

    private delegate bool EnumWindowsCallback(IntPtr window, IntPtr parameter);

    [StructLayout(LayoutKind.Sequential)]
    private struct BitmapInfoHeader
    {
        public uint Size;
        public int Width;
        public int Height;
        public ushort Planes;
        public ushort BitCount;
        public uint Compression;
        public uint SizeImage;
        public int XPelsPerMeter;
        public int YPelsPerMeter;
        public uint ColorsUsed;
        public uint ColorsImportant;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct BitmapInfo
    {
        public BitmapInfoHeader Header;
        public uint Colors;
    }

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool GetClientRect(IntPtr window, out Rect rect);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool ClientToScreen(IntPtr window, ref Point point);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool SetCursorPos(int screenX, int screenY);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool GetCursorPos(out Point point);

    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();

    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr window);

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr window, int command);

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
    private static extern uint SendInput(uint count, Input[] inputs, int inputSize);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr GetDC(IntPtr window);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern int ReleaseDC(IntPtr window, IntPtr deviceContext);

    [DllImport("gdi32.dll", SetLastError = true)]
    private static extern IntPtr CreateCompatibleDC(IntPtr deviceContext);

    [DllImport("gdi32.dll", SetLastError = true)]
    private static extern IntPtr CreateCompatibleBitmap(IntPtr deviceContext, int width, int height);

    [DllImport("gdi32.dll", SetLastError = true)]
    private static extern IntPtr SelectObject(IntPtr deviceContext, IntPtr graphicsObject);

    [DllImport("gdi32.dll", SetLastError = true)]
    private static extern bool BitBlt(IntPtr destination, int x, int y, int width, int height,
        IntPtr source, int sourceX, int sourceY, uint operation);

    [DllImport("gdi32.dll", SetLastError = true)]
    private static extern int GetDIBits(IntPtr deviceContext, IntPtr bitmap, uint startScan,
        uint scanLines, byte[] bits, ref BitmapInfo info, uint usage);

    [DllImport("gdi32.dll")]
    private static extern bool DeleteObject(IntPtr graphicsObject);

    [DllImport("gdi32.dll")]
    private static extern bool DeleteDC(IntPtr deviceContext);

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

    public static Point GetCursorPosition()
    {
        Point point;
        if (!GetCursorPos(out point))
            throw new InvalidOperationException("Win32 cursor query failed: " + Marshal.GetLastWin32Error());
        return point;
    }

    public static void MoveCursor(int screenX, int screenY)
    {
        if (!SetCursorPos(screenX, screenY))
            throw new InvalidOperationException("Win32 cursor move failed: " + Marshal.GetLastWin32Error());
    }

    public static void LeftDown()
    {
        SendMouseInput(0x0002);
    }

    public static void LeftUp()
    {
        SendMouseInput(0x0004);
    }

    private static void SendMouseInput(uint flags)
    {
        Input input = new Input {
            Type = 0,
            Union = new InputUnion {
                Mouse = new MouseInput { Flags = flags, ExtraInfo = UIntPtr.Zero }
            }
        };
        if (SendInput(1, new[] { input }, Marshal.SizeOf(typeof(Input))) != 1)
            throw new InvalidOperationException("Win32 mouse input failed: " + Marshal.GetLastWin32Error());
    }

    public static void CaptureClient(IntPtr window, string path)
    {
        Rect rect = GetClientScreenRect(window);
        if (rect.Width <= 0 || rect.Height <= 0)
            throw new InvalidOperationException("Win32 client area has no pixels to capture");
        IntPtr screen = GetDC(IntPtr.Zero);
        IntPtr memory = IntPtr.Zero;
        IntPtr bitmap = IntPtr.Zero;
        IntPtr previous = IntPtr.Zero;
        try
        {
            if (screen == IntPtr.Zero)
                throw new InvalidOperationException("Win32 screen capture context failed: " + Marshal.GetLastWin32Error());
            memory = CreateCompatibleDC(screen);
            bitmap = CreateCompatibleBitmap(screen, rect.Width, rect.Height);
            if (memory == IntPtr.Zero || bitmap == IntPtr.Zero)
                throw new InvalidOperationException("Win32 capture bitmap allocation failed: " + Marshal.GetLastWin32Error());
            previous = SelectObject(memory, bitmap);
            if (previous == IntPtr.Zero || !BitBlt(memory, 0, 0, rect.Width, rect.Height,
                screen, rect.Left, rect.Top, 0x00CC0020))
                throw new InvalidOperationException("Win32 frame copy failed: " + Marshal.GetLastWin32Error());
            SelectObject(memory, previous);
            previous = IntPtr.Zero;

            int byteCount = checked(rect.Width * rect.Height * 4);
            byte[] pixels = new byte[byteCount];
            BitmapInfo info = new BitmapInfo {
                Header = new BitmapInfoHeader {
                    Size = 40,
                    Width = rect.Width,
                    Height = -rect.Height,
                    Planes = 1,
                    BitCount = 32,
                    SizeImage = (uint)byteCount
                }
            };
            if (GetDIBits(screen, bitmap, 0, (uint)rect.Height, pixels, ref info, 0) != rect.Height)
                throw new InvalidOperationException("Win32 frame read failed: " + Marshal.GetLastWin32Error());

            using (FileStream stream = File.Create(path))
            using (BinaryWriter writer = new BinaryWriter(stream))
            {
                writer.Write((ushort)0x4D42);
                writer.Write((uint)(54 + byteCount));
                writer.Write((ushort)0);
                writer.Write((ushort)0);
                writer.Write((uint)54);
                writer.Write((uint)40);
                writer.Write(rect.Width);
                writer.Write(-rect.Height);
                writer.Write((ushort)1);
                writer.Write((ushort)32);
                writer.Write((uint)0);
                writer.Write((uint)byteCount);
                writer.Write(0);
                writer.Write(0);
                writer.Write((uint)0);
                writer.Write((uint)0);
                writer.Write(pixels);
            }
        }
        finally
        {
            if (previous != IntPtr.Zero)
                SelectObject(memory, previous);
            if (bitmap != IntPtr.Zero)
                DeleteObject(bitmap);
            if (memory != IntPtr.Zero)
                DeleteDC(memory);
            if (screen != IntPtr.Zero)
                ReleaseDC(IntPtr.Zero, screen);
        }
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
    $previousForeground = [AxiomUiSmokeNative]::GetForegroundWindow()
    $previousCursor = [AxiomUiSmokeNative]::GetCursorPosition()
    $previousStateFile = $env:AXIOM_UI_TEST_STATE_FILE
    $previousBackend = $env:WGPU_BACKEND
    try {
        $env:AXIOM_UI_TEST_STATE_FILE = $stateFile
        $env:WGPU_BACKEND = 'dx12'
        $process = Start-Process -FilePath $appPath `
            -WorkingDirectory $repoRoot `
            -WindowStyle Normal `
            -PassThru `
            -RedirectStandardOutput $stdoutPath `
            -RedirectStandardError $stderrPath
    }
    finally {
        $env:AXIOM_UI_TEST_STATE_FILE = $previousStateFile
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

    $stage = 'seeded-state'
    $initialState = Wait-UiState -Process $process -StateFile $stateFile -Stage $stage -TimeoutSeconds 30 -Predicate {
        param($state)
        $state['scenario'] -eq 'seeded-card-drag' -and
        $state['dragging'] -eq 'false' -and
        $state['zone'] -eq 'Table' -and
        (Test-Position -State $state -ExpectedX -160 -ExpectedY 130 -Tolerance 1)
    }

    $stage = 'focus-window'
    [void][AxiomUiSmokeNative]::ShowWindow($windowHandle, 9)
    $focusRequested = [AxiomUiSmokeNative]::SetForegroundWindow($windowHandle)
    Start-Sleep -Milliseconds 250
    $foregroundHandle = [AxiomUiSmokeNative]::GetForegroundWindow()
    $focusWatch = [System.Diagnostics.Stopwatch]::StartNew()
    while ($foregroundHandle -ne $windowHandle -and $focusWatch.Elapsed.TotalSeconds -lt 2) {
        [void][AxiomUiSmokeNative]::SetForegroundWindow($windowHandle)
        Start-Sleep -Milliseconds 100
        $foregroundHandle = [AxiomUiSmokeNative]::GetForegroundWindow()
    }

    $client = [AxiomUiSmokeNative]::GetClientScreenRect($windowHandle)
    if ($client.Width -lt 640 -or $client.Height -lt 480) {
        throw "game client area is too small for the smoke scenario ($($client.Width)x$($client.Height))"
    }
    $startScreenX = $client.Left + [int][Math]::Round($client.Width / 2.0 - 160)
    $startScreenY = $client.Top + [int][Math]::Round($client.Height / 2.0 + 130)
    $targetScreenX = $client.Left + [int][Math]::Round($client.Width / 2.0 - 300)
    $targetScreenY = $client.Top + [int][Math]::Round($client.Height / 2.0 - 150)
    $startClientX = [int][Math]::Round($client.Width / 2.0 - 160)
    $startClientY = [int][Math]::Round($client.Height / 2.0 + 130)
    $targetClientX = [int][Math]::Round($client.Width / 2.0 - 300)
    $targetClientY = [int][Math]::Round($client.Height / 2.0 - 150)
    @(
        "process_id=$($process.Id)"
        "window_handle=$windowHandle"
        "focus_requested=$focusRequested"
        "foreground_handle=$foregroundHandle"
        "client_screen=($($client.Left),$($client.Top),$($client.Width),$($client.Height))"
        "start_screen=($startScreenX,$startScreenY)"
        "target_screen=($targetScreenX,$targetScreenY)"
        "start_client=($startClientX,$startClientY)"
        "target_client=($targetClientX,$targetClientY)"
    ) | Set-Content -LiteralPath $inputFile

    if ($foregroundHandle -ne $windowHandle) {
        throw "could not focus game window (requested=$focusRequested foreground=$foregroundHandle)"
    }

    $stage = 'hover-card'
    [AxiomUiSmokeNative]::MoveCursor($startScreenX, $startScreenY)
    $null = Wait-UiState -Process $process -StateFile $stateFile -Stage $stage -TimeoutSeconds 5 -Predicate {
        param($state)
        $state['scenario'] -eq 'seeded-card-drag' -and
        $state['left_pressed'] -eq 'false' -and
        (Test-Position -State $state -XKey 'mouse_x' -YKey 'mouse_y' `
            -ExpectedX $startClientX -ExpectedY $startClientY -Tolerance 3)
    }

    $stage = 'press-card'
    if ([AxiomUiSmokeNative]::GetForegroundWindow() -ne $windowHandle) {
        throw 'game window lost foreground before mouse-down'
    }
    [AxiomUiSmokeNative]::LeftDown()
    $mouseDown = $true
    $null = Wait-UiState -Process $process -StateFile $stateFile -Stage $stage -TimeoutSeconds 5 -Predicate {
        param($state)
        $state['scenario'] -eq 'seeded-card-drag' -and
        $state['dragging'] -eq 'true' -and
        $state['left_pressed'] -eq 'true' -and
        $state['zone'] -eq 'Table' -and
        (Test-Position -State $state -ExpectedX -160 -ExpectedY 130 -Tolerance 25)
    }

    $stage = 'drag-card'
    for ($step = 1; $step -le 10; $step++) {
        $screenX = [int][Math]::Round($startScreenX + ($targetScreenX - $startScreenX) * $step / 10.0)
        $screenY = [int][Math]::Round($startScreenY + ($targetScreenY - $startScreenY) * $step / 10.0)
        [AxiomUiSmokeNative]::MoveCursor($screenX, $screenY)
        Start-Sleep -Milliseconds 35
    }
    $dragState = Wait-UiState -Process $process -StateFile $stateFile -Stage $stage -TimeoutSeconds 10 -Predicate {
        param($state)
        $state['scenario'] -eq 'seeded-card-drag' -and
        $state['dragging'] -eq 'true' -and
        $state['left_pressed'] -eq 'true' -and
        $state['zone'] -eq 'Table' -and
        (Test-Position -State $state -ExpectedX -300 -ExpectedY -150 -Tolerance 25)
    }

    $stage = 'capture-dragged-frame'
    $dragState.GetEnumerator() | ForEach-Object {
        '{0}={1}' -f $_.Key, $_.Value
    } | Set-Content -LiteralPath (Join-Path $artifactDir 'drag-state.txt')
    [AxiomUiSmokeNative]::CaptureClient($windowHandle, $framePath)
    if (-not (Test-Path -LiteralPath $framePath) -or (Get-Item -LiteralPath $framePath).Length -eq 0) {
        throw 'rendered frame capture was empty'
    }

    $stage = 'release-card'
    [AxiomUiSmokeNative]::LeftUp()
    $mouseDown = $false
    $releasedState = Wait-UiState -Process $process -StateFile $stateFile -Stage $stage -TimeoutSeconds 10 -Predicate {
        param($state)
        $state['scenario'] -eq 'seeded-card-drag' -and
        $state['dragging'] -eq 'false' -and
        $state['zone'] -eq 'Table'
    }
    $succeeded = $true
}
catch {
    [Console]::Error.WriteLine("UI smoke failed: scenario=seeded-card-drag stage=${stage} $($_.Exception.Message)")
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
            [AxiomUiSmokeNative]::CaptureClient($windowHandle, (Join-Path $artifactDir 'failure.bmp'))
        }
        catch {
        }
    }
    if ($mouseDown) {
        try { [AxiomUiSmokeNative]::LeftUp() } catch { }
    }
    $restoreForeground = $false
    if ($windowHandle -ne [IntPtr]::Zero) {
        try { $restoreForeground = [AxiomUiSmokeNative]::GetForegroundWindow() -eq $windowHandle } catch { }
    }
    if ($null -ne $process) {
        try {
            $process.Refresh()
            if (-not $process.HasExited) {
                Stop-Process -Id $process.Id -Force
                $null = $process.WaitForExit(5000)
            }
        }
        catch {
        }
    }
    if ($null -ne $previousCursor) {
        try { [AxiomUiSmokeNative]::MoveCursor($previousCursor.X, $previousCursor.Y) } catch { }
    }
    if ($restoreForeground -and $null -ne $previousForeground -and $previousForeground -ne [IntPtr]::Zero) {
        try { [void][AxiomUiSmokeNative]::SetForegroundWindow($previousForeground) } catch { }
    }
    if ($previousDpiContext -ne [IntPtr]::Zero) {
        try { [AxiomUiSmokeNative]::RestoreDpiContext($previousDpiContext) } catch { }
    }
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

Write-Output ("UI smoke passed: scenario=seeded-card-drag rendered=({0},{1}) frame={2}" -f `
    $dragState['rendered_x'], $dragState['rendered_y'], $framePath)
