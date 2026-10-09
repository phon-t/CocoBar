param(
    [string]$Executable = "$PSScriptRoot\..\target\debug\cocobar.exe",
    [string]$OutputDirectory = "$PSScriptRoot\..\target\ui-check",
    [switch]$Rebuild,
    [switch]$RestartRunningApp,
    [switch]$LiveUpdates
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class CatUi {
    public delegate bool EnumProc(IntPtr hwnd, IntPtr data);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc callback, IntPtr data);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr hwnd, StringBuilder text, int length);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
    [DllImport("user32.dll")] public static extern IntPtr GetDlgItem(IntPtr hwnd, int id);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr SendMessage(IntPtr hwnd, uint msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll", CharSet=CharSet.Unicode, EntryPoint="SendMessageW")] public static extern IntPtr SendText(IntPtr hwnd, uint msg, IntPtr w, string text);
    [DllImport("user32.dll", CharSet=CharSet.Unicode, EntryPoint="SendMessageW")] public static extern IntPtr ReadText(IntPtr hwnd, uint msg, IntPtr w, StringBuilder text);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int length);
    [DllImport("user32.dll")] public static extern int GetWindowTextLength(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out Rect rect);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hwnd, IntPtr dc, uint flags);
    [DllImport("user32.dll")] public static extern uint GetGuiResources(IntPtr process, uint flags);
    [DllImport("user32.dll", EntryPoint="GetWindowLongPtrW")] public static extern IntPtr GetWindowStyle(IntPtr hwnd, int index);
    [DllImport("user32.dll")] public static extern int GetWindowRgn(IntPtr hwnd, IntPtr region);
    [DllImport("gdi32.dll")] public static extern IntPtr CreateRectRgn(int left, int top, int right, int bottom);
    [DllImport("gdi32.dll")] public static extern bool PtInRegion(IntPtr region, int x, int y);
    [DllImport("gdi32.dll")] public static extern bool DeleteObject(IntPtr obj);
    [DllImport("user32.dll")] public static extern bool GetCursorPos(out Point point);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hwnd, IntPtr after, int x, int y, int w, int h, uint flags);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr CreateWindowEx(uint exStyle, string className, string title, uint style, int x, int y, int w, int h, IntPtr parent, IntPtr menu, IntPtr instance, IntPtr parameter);
    [DllImport("user32.dll")] public static extern bool UpdateWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern bool DestroyWindow(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern int GetSystemMetrics(int index);
    [DllImport("user32.dll")] public static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extra);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern bool PostMessage(IntPtr hwnd, uint msg, IntPtr w, IntPtr l);
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int left, top, right, bottom; }
    [StructLayout(LayoutKind.Sequential)] public struct Point { public int x, y; }
    public static bool Contains(IntPtr hwnd, int x, int y) {
        var region = CreateRectRgn(0, 0, 0, 0);
        try { return GetWindowRgn(hwnd, region) != 0 && PtInRegion(region, x, y); }
        finally { DeleteObject(region); }
    }
    public static IntPtr Find(uint processId, string className) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((hwnd, data) => {
            uint pid; GetWindowThreadProcessId(hwnd, out pid);
            var text = new StringBuilder(128); GetClassName(hwnd, text, text.Capacity);
            if (pid == processId && text.ToString() == className) { found = hwnd; return false; }
            return true;
        }, IntPtr.Zero);
        return found;
    }
    public static string Text(IntPtr hwnd) {
        var text = new StringBuilder(SendMessage(hwnd, 0x000E, IntPtr.Zero, IntPtr.Zero).ToInt32() + 1);
        ReadText(hwnd, 0x000D, (IntPtr)text.Capacity, text); return text.ToString();
    }
}
'@

function Assert-Ui([bool]$Condition, [string]$Message) {
    if (!$Condition) { throw $Message }
    Write-Output "PASS $Message"
}
function Send-Ui([IntPtr]$Window, [uint32]$Message, [long]$W = 0, [long]$L = 0) {
    [void][CatUi]::SendMessage($Window, $Message, [IntPtr]$W, [IntPtr]$L)
}
function Edit-Ui([IntPtr]$Control, [string]$Text) {
    Send-Ui $Control 0xB1 0 -1 # Select all
    [void][CatUi]::SendText($Control, 0xC2, [IntPtr]1, $Text) # Replace selection, with undo
}
function Find-Ui([string]$Class) {
    for ($attempt = 0; $attempt -lt 240; $attempt++) {
        $window = [CatUi]::Find($script:appProcess.Id, $Class)
        if ($window -ne [IntPtr]::Zero) { return $window }
        if ($script:appProcess.HasExited) { throw "Test application exited before $Class appeared" }
        Start-Sleep -Milliseconds 250
    }
    throw "Timed out waiting for $Class"
}
function Open-Menu {
    Send-Ui (Find-Ui 'cocoBarWnd') 0x205
    return Find-Ui 'CatMenuWnd'
}
function Read-Data { [System.IO.File]::ReadAllText((Join-Path $script:dataDirectory 'mydata.txt')) }
function Read-LatestNote { @((Read-Data) -split "`n" | Where-Object { $_.StartsWith("N`t") })[0] }
function Read-NoteRecords { (Read-Data) -split "`n" | Where-Object { $_.StartsWith("S`t") } }
function Open-NoteId([IntPtr]$Menu, [long]$Id) {
    Send-Ui $Menu 0x111 402
    Send-Ui $Menu 0x111 423
    $records = @(Read-NoteRecords)
    $index = -1
    for ($i = 0; $i -lt $records.Count; $i++) { if (($records[$i] -split "`t", 5)[1] -eq "$Id") { $index = $i; break } }
    Assert-Ui ($index -ge 0) "Saved note $Id is available"
    for ($page = 0; $page -lt [math]::Floor($index / 3); $page++) { Send-Ui $Menu 0x111 428 }
    Send-Ui $Menu 0x111 (600 + $index % 3)
}
function Escape-Data([string]$Text) { $Text.Replace('\', '\\').Replace("`r", '\r').Replace("`n", '\n') }
function Get-UiRect([IntPtr]$Window) {
    $rect = New-Object CatUi+Rect
    [void][CatUi]::GetWindowRect($Window, [ref]$rect)
    return $rect
}
function Assert-PanelShape([IntPtr]$Window, [string]$Name) {
    Assert-Ui ([CatUi]::IsWindow($Window)) "$Name window is ready"
    $rect = Get-UiRect $Window
    $height = $rect.bottom - $rect.top
    Assert-Ui (($rect.right - $rect.left) -eq 470) "$Name uses the shared panel width"
    Assert-Ui ([CatUi]::Contains($Window, 0, 0) -and [CatUi]::Contains($Window, 469, 0) -and [CatUi]::Contains($Window, 469, $height - 1)) "$Name has three sharp corners"
    Assert-Ui (![CatUi]::Contains($Window, 0, $height - 1) -and [CatUi]::Contains($Window, 18, $height - 1)) "$Name has a rounded bottom-left corner (height $height)"
    $close = Get-UiRect ([CatUi]::GetDlgItem($Window, 434))
    Assert-Ui (($close.left - $rect.left) -eq 438 -and ($close.top - $rect.top) -eq 0 -and $close.right -eq $rect.right -and ![CatUi]::Contains($Window, 436, 16) -and ![CatUi]::Contains($Window, 454, 33) -and [CatUi]::Contains($Window, 454, 16)) "$Name keeps its close button in the top-right corner with a thin L-shaped gap"
}
function Press-Hotkey([byte]$Key) {
    try {
        [CatUi]::keybd_event(0x11, 0, 0, [UIntPtr]::Zero)
        [CatUi]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero)
        [CatUi]::keybd_event($Key, 0, 0, [UIntPtr]::Zero)
    } finally {
        [CatUi]::keybd_event($Key, 0, 2, [UIntPtr]::Zero)
        [CatUi]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
        [CatUi]::keybd_event(0x11, 0, 2, [UIntPtr]::Zero)
    }
    Start-Sleep -Milliseconds 150
}
function Capture-Pet([IntPtr]$Window, [string]$Name) {
    $rect = Get-UiRect $Window
    $bitmap = New-Object System.Drawing.Bitmap(($rect.right - $rect.left), ($rect.bottom - $rect.top))
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        # The entire capture is inside our plain white test backdrop.
        $graphics.CopyFromScreen($rect.left, $rect.top, 0, 0, $bitmap.Size)
        $bitmap.Save((Join-Path $script:outputPath $Name), [System.Drawing.Imaging.ImageFormat]::Png)
    } finally { $graphics.Dispose(); $bitmap.Dispose() }
}
function Capture-Ui([IntPtr]$Window, [string]$Name) {
    $rect = New-Object CatUi+Rect
    [void][CatUi]::GetWindowRect($Window, [ref]$rect)
    $bitmap = New-Object System.Drawing.Bitmap(($rect.right - $rect.left), ($rect.bottom - $rect.top))
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $dc = $graphics.GetHdc()
    try { [void][CatUi]::PrintWindow($Window, $dc, 0) }
    finally { $graphics.ReleaseHdc($dc); $graphics.Dispose() }
    $regionHandle = [CatUi]::CreateRectRgn(0, 0, 0, 0)
    try {
        if ([CatUi]::GetWindowRgn($Window, $regionHandle) -ne 0) {
            $inside = [System.Drawing.Region]::FromHrgn($regionHandle)
            $outside = New-Object System.Drawing.Region([System.Drawing.Rectangle]::new(0, 0, $bitmap.Width, $bitmap.Height))
            $outside.Exclude($inside)
            $maskGraphics = [System.Drawing.Graphics]::FromImage($bitmap)
            try {
                $maskGraphics.CompositingMode = [System.Drawing.Drawing2D.CompositingMode]::SourceCopy
                $maskGraphics.FillRegion([System.Drawing.Brushes]::Transparent, $outside)
            } finally { $maskGraphics.Dispose(); $inside.Dispose(); $outside.Dispose() }
        }
    } finally { [void][CatUi]::DeleteObject($regionHandle) }
    try { $bitmap.Save((Join-Path $script:outputPath $Name), [System.Drawing.Imaging.ImageFormat]::Png) }
    finally { $bitmap.Dispose() }
}
function Assert-TabColors([IntPtr]$Window, [bool]$NotesActive) {
    [void][CatUi]::UpdateWindow($Window)
    $bitmap = New-Object System.Drawing.Bitmap(470, 454)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $dc = $graphics.GetHdc()
    try { [void][CatUi]::PrintWindow($Window, $dc, 0) }
    finally { $graphics.ReleaseHdc($dc); $graphics.Dispose() }
    try {
        $todo = $bitmap.GetPixel(48, 22).ToArgb() -band 0xffffff
        $notes = $bitmap.GetPixel(158, 22).ToArgb() -band 0xffffff
        Assert-Ui ($todo -eq $(if ($NotesActive) { 0xffffff } else { 0x4967d9 })) 'To Do shows blue only when selected and white otherwise'
        Assert-Ui ($notes -eq $(if ($NotesActive) { 0x4967d9 } else { 0xffffff })) 'Notes shows blue only when selected and white otherwise'
    } finally { $bitmap.Dispose() }
}
function Start-TestApp {
    Write-Output 'Starting isolated test application'
    $script:appProcess = Start-Process -FilePath $script:executablePath -WindowStyle Hidden -PassThru
    [void](Find-Ui 'cocoBarWnd')
    Assert-Ui ($script:appProcess.WaitForInputIdle(30000)) 'Window initialization and global shortcuts are ready'
}

$executablePath = (Resolve-Path -LiteralPath $Executable).Path
$outputPath = [System.IO.Path]::GetFullPath($OutputDirectory)
[void][System.IO.Directory]::CreateDirectory($outputPath)
$dataDirectory = Join-Path $outputPath ('data-' + [guid]::NewGuid().ToString('N'))
[void][System.IO.Directory]::CreateDirectory($dataDirectory)
[System.IO.File]::WriteAllText((Join-Path $dataDirectory 'config.txt'), "{hotkeys: 0}`n{notes_hotkey: 0}`n{todo_hotkey: 0}`n{desktop_shortcut: 0}`n{size_px: 200}`n")
[System.IO.File]::WriteAllText((Join-Path $dataDirectory 'mydata.txt'), "N`tLegacy note")
$oldDataDir = $env:COCOBAR_DATA_DIR
$oldUiTest = $env:COCOBAR_UI_TEST
$env:COCOBAR_DATA_DIR = $dataDirectory
# Keep the isolated test panels open if the user interacts with Codex while the
# checks run. The normal app still closes its menu when it loses foreground.
$env:COCOBAR_UI_TEST = '1'
$appProcess = $null
$restoreRunningApp = $false
$releaseBuilt = $false
$previousExecutable = Join-Path $dataDirectory 'cocobar-before-update.exe'
$backdrop = [IntPtr]::Zero
$originalCursor = New-Object CatUi+Point
[void][CatUi]::GetCursorPos([ref]$originalCursor)
try {
    if ($Rebuild) {
        $runningApps = @(Get-Process cocobar -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $executablePath })
        if ($runningApps.Count -gt 0 -and !$RestartRunningApp) { throw 'Close the running release app or pass -RestartRunningApp before rebuilding.' }
        if ($runningApps.Count -gt 0) { Copy-Item -LiteralPath $executablePath -Destination $previousExecutable }
        foreach ($runningApp in $runningApps) {
            $runningMenu = [CatUi]::Find($runningApp.Id, 'CatMenuWnd')
            if ($runningMenu -ne [IntPtr]::Zero) {
                Send-Ui $runningMenu 0x10
                if ([CatUi]::Find($runningApp.Id, 'CatMenuWnd') -ne [IntPtr]::Zero) {
                    throw 'The running app could not save its draft. It has been left open.'
                }
            }
            $runningPet = [CatUi]::Find($runningApp.Id, 'cocoBarWnd')
            if ($runningPet -eq [IntPtr]::Zero) { throw 'Could not find the running pet window; it has been left open.' }
            Send-Ui $runningPet 0x10
            if (!$runningApp.WaitForExit(5000)) { throw 'The running app has been left open because it did not close normally.' }
            $restoreRunningApp = $true
        }
        & cargo build --release --offline
        if ($LASTEXITCODE -ne 0) { throw 'The release build failed.' }
        $releaseBuilt = $true
    }
    Start-TestApp
    $menu = Open-Menu
    Assert-PanelShape $menu 'Menu'
    Assert-TabColors $menu $false
    Assert-Ui (![CatUi]::IsWindowVisible([CatUi]::GetDlgItem($menu, 413))) 'Empty task lists have no redundant page controls'
    Send-Ui $menu 0x111 402
    Assert-TabColors $menu $true
    Assert-Ui (![CatUi]::IsWindowVisible([CatUi]::GetDlgItem($menu, 427))) 'A single saved note has no redundant page controls'
    Assert-Ui ([CatUi]::IsWindowVisible([CatUi]::GetDlgItem($menu, 600))) 'Legacy notes appear as editable cards'
    Send-Ui $menu 0x111 600
    $editor = [CatUi]::GetDlgItem($menu, 420)
    Assert-Ui ([CatUi]::Text($editor) -eq 'Legacy note') 'Legacy note text is preserved during migration'
    Assert-Ui ([CatUi]::SendMessage($editor, 0xD5, [IntPtr]::Zero, [IntPtr]::Zero).ToInt64() -gt 32767) 'Notes accept more than 32,767 characters'
    $note = "猫 🐈 café & literal \n C:\notes\new`r`nSecond line`r`n" * 1000
    Edit-Ui $editor $note
    Start-Sleep -Milliseconds 1100
    Assert-Ui ((Read-LatestNote) -ceq ("N`t" + (Escape-Data $note))) 'Autosave preserves long Unicode notes, CRLF, and backslashes'
    $savedSnapshot = Read-Data
    $savedMetadata = [CatUi]::Text([CatUi]::GetDlgItem($menu, 425))
    Capture-Ui $menu 'notes.png'

    $dataFile = Get-Item -LiteralPath (Join-Path $dataDirectory 'mydata.txt')
    $dataFile.IsReadOnly = $true
    try {
        Edit-Ui $editor 'Pending after a failed save'
        Start-Sleep -Milliseconds 1100
        Assert-Ui ((Read-Data) -ceq $savedSnapshot) 'A failed autosave preserves the previous file'
        Assert-Ui ([CatUi]::Text([CatUi]::GetDlgItem($menu, 425)) -ceq $savedMetadata) 'Failed writes keep the previous saved timestamp'
        Send-Ui $menu 0x10
        Assert-Ui ([CatUi]::Find($appProcess.Id, 'CatMenuWnd') -ne [IntPtr]::Zero) 'Save errors keep the pending draft open'
        Send-Ui (Find-Ui 'cocoBarWnd') 0x10
        Assert-Ui (!$appProcess.HasExited -and [CatUi]::Find($appProcess.Id, 'CatMenuWnd') -ne [IntPtr]::Zero) 'Closing the pet also protects a draft after a failed save'
    } finally { $dataFile.IsReadOnly = $false }
    Send-Ui $menu 0x111 421
    Assert-Ui ((Read-LatestNote) -ceq "N`tPending after a failed save") 'Save retries successfully after the file becomes writable'

    $draft = "Draft before closing`r`nLiteral \n and \r"
    Edit-Ui $editor $draft
    Send-Ui $menu 0x10 # Close immediately, before debounce
    Assert-Ui ((Read-LatestNote) -ceq ("N`t" + (Escape-Data $draft))) 'Closing the menu saves pending edits immediately'
    $menu = Open-Menu
    Send-Ui $menu 0x111 402
    Send-Ui $menu 0x111 600
    Assert-Ui ([CatUi]::Text([CatUi]::GetDlgItem($menu, 420)) -ceq $draft) 'Notes survive reopening the menu'

    # All task writes must include the current draft, even before autosave fires.
    $draft = "New draft while adding tasks`r`n猫 & \n"
    Edit-Ui ([CatUi]::GetDlgItem($menu, 420)) $draft
    Send-Ui $menu 0x111 401
    for ($i = 1; $i -le 12; $i++) {
        Edit-Ui ([CatUi]::GetDlgItem($menu, 410)) "Task $i"
        Send-Ui $menu 0x111 411
    }
    Assert-TabColors $menu $false
    Assert-Ui ([CatUi]::Text([CatUi]::GetDlgItem($menu, 415)) -eq 'Page 2 of 2') 'Tasks beyond the first eight are accessible'
    Assert-Ui ([CatUi]::IsWindowVisible([CatUi]::GetDlgItem($menu, 413))) 'Task page navigation appears when there are multiple pages'
    $previousRect = Get-UiRect ([CatUi]::GetDlgItem($menu, 413))
    $nextRect = Get-UiRect ([CatUi]::GetDlgItem($menu, 414))
    $counterRect = Get-UiRect ([CatUi]::GetDlgItem($menu, 415))
    Assert-Ui ($previousRect.top -eq $nextRect.top -and $previousRect.bottom -eq $nextRect.bottom -and ($counterRect.top + $counterRect.bottom) -eq ($previousRect.top + $previousRect.bottom)) 'Task arrows and page label share the same vertical center'
    Send-Ui $menu 0x202 0 (30 + (94 -shl 16)) # Complete task 9 on page 2
    Assert-Ui ((Read-Data).Contains("T`t1`tTask 9")) 'Task completion acts on the current page'
    Assert-Ui ((Read-LatestNote) -ceq ("N`t" + (Escape-Data $draft))) 'Task writes preserve the newest note draft'
    Send-Ui $menu 0x202 0 (434 + (94 -shl 16)) # Delete task 9
    Assert-Ui (!(Read-Data).Contains("T`t1`tTask 9")) 'Individual task deletion works on later pages'
    Send-Ui $menu 0x111 413
    Capture-Ui $menu 'tasks.png'

    Send-Ui $menu 0x111 402
    Send-Ui $menu 0x111 423
    foreach ($title in @('Weekend plans', 'Project ideas', 'Shopping list', 'Reading notes')) {
        Send-Ui $menu 0x111 422
        Edit-Ui ([CatUi]::GetDlgItem($menu, 420)) "$title`r`nA separate saved note with its own card."
        Send-Ui $menu 0x111 421
        Assert-Ui ([CatUi]::Text([CatUi]::GetDlgItem($menu, 425)).Contains('Saved ')) 'The editor shows a saved date and time'
        Send-Ui $menu 0x111 423
    }
    Assert-Ui (@(Read-NoteRecords).Count -eq 5) 'New notes keep all previous cards'
    $missingDates = @(Read-NoteRecords | Where-Object { $fields = $_ -split "`t", 5; $fields[1] -ne '1' -and ([long]$fields[2] -le 0 -or [long]$fields[3] -le 0) })
    Assert-Ui ($missingDates.Count -eq 0) 'New cards persist creation and saved timestamps'
    Assert-Ui ([CatUi]::Text([CatUi]::GetDlgItem($menu, 429)) -eq 'Page 1 of 2') 'Notes use pages when the card collection grows'
    Send-Ui $menu 0x111 428
    Assert-Ui ([CatUi]::Text([CatUi]::GetDlgItem($menu, 429)) -eq 'Page 2 of 2') 'Older cards remain accessible on later pages'
    Send-Ui $menu 0x111 600
    Edit-Ui ([CatUi]::GetDlgItem($menu, 420)) "Edited card`r`nChanged one note without replacing the others."
    Send-Ui $menu 0x111 421
    Send-Ui $menu 0x111 423
    Assert-Ui (@(Read-NoteRecords).Count -eq 5) 'Editing a card preserves the rest of the collection'
    $beforeDelete = @(Read-NoteRecords)
    Send-Ui $menu 0x111 610
    Assert-Ui (@(Read-NoteRecords).Count -eq 4) 'Deleting a card removes only that note'
    Send-Ui $menu 0x111 426
    Assert-Ui ((@(Read-NoteRecords) -join "`n") -ceq ($beforeDelete -join "`n")) 'Undo restores the deleted note and its timestamps'
    Capture-Ui $menu 'note-cards.png'
    Open-NoteId $menu 1
    Edit-Ui ([CatUi]::GetDlgItem($menu, 420)) $draft
    Send-Ui $menu 0x111 421
    Send-Ui $menu 0x111 423

    Send-Ui $menu 0x111 430
    $customize = Find-Ui 'CatCustomizeWnd'
    Assert-PanelShape $customize 'Customize'
    foreach ($id in @(470, 471, 472, 450, 451, 452, 453, 454, 455, 441, 442, 443, 460, 461, 462, 463)) {
        Assert-Ui (([CatUi]::GetWindowStyle([CatUi]::GetDlgItem($customize, $id), -16).ToInt64() -band 15) -eq 2) "Customize option $id uses a square checkbox"
    }
    Send-Ui $customize 0x111 452 # Red scarf
    Send-Ui $customize 0x111 443 # Red bell
    Assert-Ui ([CatUi]::SendMessage([CatUi]::GetDlgItem($customize, 470), 0xF0, [IntPtr]::Zero, [IntPtr]::Zero).ToInt64() -eq 1) 'Cosmetics preserve the color checkbox selection'
    Assert-Ui ([CatUi]::SendMessage([CatUi]::GetDlgItem($customize, 452), 0xF0, [IntPtr]::Zero, [IntPtr]::Zero).ToInt64() -eq 1) 'Bell selection preserves the scarf checkbox selection'
    Send-Ui $customize 0x111 462 # Orange tie
    Assert-Ui ([CatUi]::SendMessage([CatUi]::GetDlgItem($customize, 441), 0xF0, [IntPtr]::Zero, [IntPtr]::Zero).ToInt64() -eq 1) 'Tie selection clears the bell'
    Send-Ui ([CatUi]::GetDlgItem($customize, 462)) 0xF5 # BM_CLICK through the actual control
    Assert-Ui ([CatUi]::SendMessage([CatUi]::GetDlgItem($customize, 460), 0xF0, [IntPtr]::Zero, [IntPtr]::Zero).ToInt64() -eq 1) 'Clicking a selected accessory removes it'
    Send-Ui ([CatUi]::GetDlgItem($customize, 462)) 0xF5
    Send-Ui ([CatUi]::GetDlgItem($customize, 470)) 0xF5
    Assert-Ui ([CatUi]::SendMessage([CatUi]::GetDlgItem($customize, 470), 0xF0, [IntPtr]::Zero, [IntPtr]::Zero).ToInt64() -eq 1) 'Clicking the selected color keeps one color selected'
    $black = Get-UiRect ([CatUi]::GetDlgItem($customize, 470))
    $white = Get-UiRect ([CatUi]::GetDlgItem($customize, 471))
    $orange = Get-UiRect ([CatUi]::GetDlgItem($customize, 472))
    Assert-Ui ($black.top -eq $white.top -and $white.top -eq $orange.top -and ($white.left - $black.left) -eq ($orange.left - $white.left)) 'Color choices align in an evenly spaced row'
    $sizeEdit = [CatUi]::GetDlgItem($customize, 484)
    Edit-Ui $sizeEdit '240'
    [void][CatUi]::PostMessage($sizeEdit, 0x100, [IntPtr]13, [IntPtr]::Zero)
    Start-Sleep -Milliseconds 300
    $petRect = Get-UiRect (Find-Ui 'cocoBarWnd')
    Assert-Ui (($petRect.right - $petRect.left) -eq 240) 'Enter applies the size from the Customize editor'
    Edit-Ui $sizeEdit '200'
    [void][CatUi]::PostMessage($sizeEdit, 0x100, [IntPtr]13, [IntPtr]::Zero)
    Start-Sleep -Milliseconds 300
    Capture-Ui $customize 'customize.png'
    Send-Ui ([CatUi]::GetDlgItem($customize, 434)) 0xF5
    Assert-Ui ([CatUi]::Find($appProcess.Id, 'CatCustomizeWnd') -eq [IntPtr]::Zero) 'The cutout close button closes Customize'
    Send-Ui $menu 0x111 430
    $customize = Find-Ui 'CatCustomizeWnd'
    [void][CatUi]::PostMessage([CatUi]::GetDlgItem($customize, 470), 0x100, [IntPtr]27, [IntPtr]::Zero)
    Start-Sleep -Milliseconds 100
    Assert-Ui ([CatUi]::Find($appProcess.Id, 'CatCustomizeWnd') -eq [IntPtr]::Zero -and [CatUi]::Find($appProcess.Id, 'CatMenuWnd') -ne [IntPtr]::Zero) 'Escape closes only the active Customize panel'

    Send-Ui $menu 0x111 433
    $settings = Find-Ui 'CatSettingsWnd'
    Assert-PanelShape $settings 'Settings'
    $view = [CatUi]::GetDlgItem($settings, 520)
    $autoUpdate = [CatUi]::GetDlgItem($view, 492)
    Assert-Ui ([CatUi]::SendMessage($autoUpdate, 0xF0, [IntPtr]::Zero, [IntPtr]::Zero).ToInt64() -eq 0) 'Automatic GitHub updates default off for existing settings'
    Send-Ui $autoUpdate 0xF5
    Assert-Ui ((Get-Content (Join-Path $dataDirectory 'config.txt') -Raw).Contains('{auto_update: 1}')) 'Opting in persists automatic updates'
    Send-Ui $settings 0x10
    Send-Ui $menu 0x111 433
    $settings = Find-Ui 'CatSettingsWnd'
    $view = [CatUi]::GetDlgItem($settings, 520)
    $autoUpdate = [CatUi]::GetDlgItem($view, 492)
    Assert-Ui ([CatUi]::SendMessage($autoUpdate, 0xF0, [IntPtr]::Zero, [IntPtr]::Zero).ToInt64() -eq 1) 'Reopening Settings preserves the automatic update checkbox'
    Send-Ui $autoUpdate 0xF5
    Assert-Ui ((Get-Content (Join-Path $dataDirectory 'config.txt') -Raw).Contains('{auto_update: 0}')) 'Disabling automatic updates is persisted'
    $settingsFile = Join-Path $dataDirectory 'config.txt'
    $settingsAttributes = [IO.File]::GetAttributes($settingsFile)
    try {
        [IO.File]::SetAttributes($settingsFile, ($settingsAttributes -bor [IO.FileAttributes]::ReadOnly))
        Send-Ui $autoUpdate 0xF5
        Assert-Ui ([CatUi]::SendMessage($autoUpdate, 0xF0, [IntPtr]::Zero, [IntPtr]::Zero).ToInt64() -eq 0) 'A settings save failure keeps automatic updates off'
        [void][CatUi]::UpdateWindow($settings)
        Assert-Ui ([CatUi]::Text([CatUi]::GetDlgItem($settings, 514)).StartsWith('Could not save settings:')) 'Settings save failures show readable feedback'
    } finally { [IO.File]::SetAttributes($settingsFile, $settingsAttributes) }
    if ($LiveUpdates) {
        $releaseLine = & (Join-Path $PSScriptRoot '..\src\check-release.ps1')
        $buildVersion = ([regex]::Match((Get-Content (Join-Path $PSScriptRoot '..\Cargo.toml') -Raw), '(?m)^version = "([^"]+)"')).Groups[1].Value
        # Never replace the working build with a future upstream release.
        if ([version]($releaseLine.Split('|')[0].TrimStart('v')) -le [version]$buildVersion) {
            Send-Ui ([CatUi]::GetDlgItem($settings, 432)) 0xF5
            [void][CatUi]::UpdateWindow($settings)
            $deadline = [DateTime]::UtcNow.AddSeconds(60)
            do {
                Start-Sleep -Milliseconds 100
                [void][CatUi]::UpdateWindow($settings)
                $updateStatus = [CatUi]::Text([CatUi]::GetDlgItem($settings, 514))
            } while ($updateStatus.Contains('Checking') -and [DateTime]::UtcNow -lt $deadline)
            Assert-Ui ($updateStatus -eq "Up to date (v$buildVersion).") "The manual GitHub update check returns Up to date (v$buildVersion), actual status: $updateStatus"
            Assert-Ui (!$appProcess.HasExited) 'An up-to-date check keeps the app and current notes open'
        }
    }
    Capture-Ui $settings 'settings-updates.png'
    Send-Ui $settings 0x115 7
    $viewRect = Get-UiRect $view
    $exitRect = Get-UiRect ([CatUi]::GetDlgItem($view, 506))
    Assert-Ui ($exitRect.top -ge $viewRect.top -and $exitRect.bottom -le $viewRect.bottom) 'Scrolling reveals the complete last shortcut row'
    foreach ($id in @(465, 490, 491, 492, 500, 501, 502, 503, 504, 505, 506, 507, 508)) {
        $control = [CatUi]::GetDlgItem($view, $id)
        $controlRect = Get-UiRect $control
        Assert-Ui (($controlRect.left - $viewRect.left) -eq 12 -and $controlRect.bottom -le $viewRect.bottom -and $controlRect.right -le $viewRect.right) "Setting $id aligns inside its card"
        Assert-Ui (([CatUi]::GetWindowStyle($control, -16).ToInt64() -band 15) -eq 3) "Setting $id uses a square checkbox"
    }
    $topmost = [CatUi]::GetDlgItem($view, 465)
    $wasTopmost = [CatUi]::SendMessage($topmost, 0xF0, [IntPtr]::Zero, [IntPtr]::Zero).ToInt64()
    Send-Ui $topmost 0xF5
    Assert-Ui ([CatUi]::SendMessage($topmost, 0xF0, [IntPtr]::Zero, [IntPtr]::Zero).ToInt64() -ne $wasTopmost) 'Window settings toggle through checkbox clicks'
    Send-Ui $topmost 0xF5
    foreach ($id in @(507, 508)) {
        $control = [CatUi]::GetDlgItem($view, $id)
        Send-Ui $control 0xF5
        Assert-Ui ([CatUi]::SendMessage($control, 0xF0, [IntPtr]::Zero, [IntPtr]::Zero).ToInt64() -eq 1) "Direct-open shortcut $id registers successfully"
    }
    Capture-Ui $settings 'settings-shortcuts.png'
    Send-Ui $settings 0x115 6
    Capture-Ui $settings 'settings.png'
    Send-Ui $settings 0x10

    $initialGdi = [CatUi]::GetGuiResources($appProcess.Handle, 0)
    for ($i = 0; $i -lt 15; $i++) {
        Send-Ui $menu 0x111 430
        $customize = Find-Ui 'CatCustomizeWnd'
        Send-Ui $customize 0x10
        Send-Ui $menu 0x111 433
        $settings = Find-Ui 'CatSettingsWnd'
        Send-Ui $settings 0x10
    }
    $finalGdi = [CatUi]::GetGuiResources($appProcess.Handle, 0)
    Assert-Ui ($finalGdi -le $initialGdi + 8) "Repeated panel opens keep GDI resources bounded ($initialGdi -> $finalGdi)"

    Send-Ui $menu 0x10
    Press-Hotkey 0x4E
    $menu = Find-Ui 'CatMenuWnd'
    Assert-Ui ([CatUi]::IsWindowVisible([CatUi]::GetDlgItem($menu, 422))) 'Ctrl+Alt+N opens the saved Notes cards directly'
    Send-Ui $menu 0x111 600
    Edit-Ui ([CatUi]::GetDlgItem($menu, 420)) 'Draft while switching directly between tabs'
    Press-Hotkey 0x54
    Assert-Ui ([CatUi]::IsWindowVisible([CatUi]::GetDlgItem($menu, 410))) 'Ctrl+Alt+T switches directly to To Do'
    Press-Hotkey 0x4E
    Assert-Ui ([CatUi]::Text([CatUi]::GetDlgItem($menu, 420)) -eq 'Draft while switching directly between tabs') 'Direct tab shortcuts preserve an unsaved note draft'
    Open-NoteId $menu 1
    Edit-Ui ([CatUi]::GetDlgItem($menu, 420)) $draft
    Send-Ui $menu 0x111 421
    Send-Ui $menu 0x111 430
    [void](Find-Ui 'CatCustomizeWnd')
    Press-Hotkey 0x54
    Assert-Ui ([CatUi]::Find($appProcess.Id, 'CatCustomizeWnd') -eq [IntPtr]::Zero -and [CatUi]::IsWindowVisible([CatUi]::GetDlgItem($menu, 410))) 'Direct shortcuts bring the requested tab forward from Customize'
    Send-Ui ([CatUi]::GetDlgItem($menu, 434)) 0xF5
    Assert-Ui ([CatUi]::Find($appProcess.Id, 'CatMenuWnd') -eq [IntPtr]::Zero) 'The cutout menu close button closes the panel'

    # Verify actual mouse handlers with a neutral backdrop, never capturing user content.
    $backdropWidth = [math]::Min(960, [CatUi]::GetSystemMetrics(0) - 40)
    $backdropHeight = [math]::Min(720, [CatUi]::GetSystemMetrics(1) - 80)
    $backdrop = [CatUi]::CreateWindowEx(0x08000008, 'STATIC', '', [uint32]2415919110, 20, 20, $backdropWidth, $backdropHeight, [IntPtr]::Zero, [IntPtr]::Zero, [IntPtr]::Zero, [IntPtr]::Zero)
    Assert-Ui ($backdrop -ne [IntPtr]::Zero) 'Pet checks use their own neutral backdrop'
    [void][CatUi]::UpdateWindow($backdrop)
    $pet = Find-Ui 'cocoBarWnd'
    [void][CatUi]::SetWindowPos($pet, [IntPtr](-1), 180, 100, 0, 0, 0x11)
    [void][CatUi]::SetCursorPos(280, 230)
    Start-Sleep -Milliseconds 500
    $normalRect = Get-UiRect $pet
    Capture-Pet $pet 'pet-normal.png'
    Send-Ui $pet 0x201 1
    Start-Sleep -Milliseconds 50
    Capture-Pet $pet 'pet-annoyed.png'
    Send-Ui $pet 0x202
    Start-Sleep -Milliseconds 1650
    Capture-Pet $pet 'pet-recovered.png'
    Send-Ui $pet 0x201 1
    [void][CatUi]::SetCursorPos(292, 230)
    Send-Ui $pet 0x200 1
    Start-Sleep -Milliseconds 50
    $tiltedRect = Get-UiRect $pet
    Assert-Ui (($tiltedRect.right - $tiltedRect.left) -gt ($normalRect.right - $normalRect.left)) 'Dragging immediately switches to the wider tilted pose'
    Assert-Ui (($tiltedRect.bottom - $tiltedRect.top) -ge [math]::Floor(1980 * 0.9 * 200 / 1192)) 'The drag canvas includes room below the artwork for the whole tail'
    Capture-Pet $pet 'pet-tilted.png'
    Send-Ui $pet 0x20A (120 -shl 16)
    Start-Sleep -Milliseconds 50
    $resizedTilt = Get-UiRect $pet
    Assert-Ui (($resizedTilt.right - $resizedTilt.left) -gt ($tiltedRect.right - $tiltedRect.left)) 'Resizing while held preserves the tilted pose at its new size'
    Send-Ui $pet 0x202
    Start-Sleep -Milliseconds 60
    $droppedRect = Get-UiRect $pet
    Assert-Ui (($droppedRect.right - $droppedRect.left) -eq 240) 'Dropping after a wheel resize restores the requested normal width'
    Send-Ui $pet 0x201 1
    [void][CatUi]::SetCursorPos(305, 230)
    Send-Ui $pet 0x200 1
    Send-Ui $pet 0x215 # WM_CAPTURECHANGED
    Start-Sleep -Milliseconds 50
    $cancelledRect = Get-UiRect $pet
    Assert-Ui (($cancelledRect.right - $cancelledRect.left) -eq 240) 'Losing mouse capture restores the normal pose'
    Send-Ui $pet 0x202
    [void][CatUi]::SetCursorPos(310, 230)
    Send-Ui $pet 0x201 1
    for ($step = 1; $step -le 12; $step++) {
        [void][CatUi]::SetCursorPos(310 + 25 * $step, 230)
        Send-Ui $pet 0x200 1
        Start-Sleep -Milliseconds 8
    }
    Send-Ui $pet 0x202
    Start-Sleep -Milliseconds 50
    Capture-Pet $pet 'pet-dizzy.png'
    Start-Sleep -Milliseconds 3450
    Capture-Pet $pet 'pet-after-dizzy.png'
    Assert-Ui (!$appProcess.HasExited) 'Pet reactions, dragging, and resizing keep the application running'
    [void][CatUi]::DestroyWindow($backdrop)
    $backdrop = [IntPtr]::Zero
    [void][CatUi]::SetCursorPos($originalCursor.x, $originalCursor.y)

    Send-Ui (Find-Ui 'cocoBarWnd') 0x10
    Assert-Ui ($appProcess.WaitForExit(5000)) 'Application closes cleanly'
    Start-TestApp
    Press-Hotkey 0x4E
    $menu = Find-Ui 'CatMenuWnd'
    Assert-Ui ([CatUi]::IsWindowVisible([CatUi]::GetDlgItem($menu, 422))) 'Direct Notes shortcut remains enabled after restart'
    $menu = Open-Menu
    Send-Ui $menu 0x111 402
    Assert-Ui (@(Read-NoteRecords).Count -eq 5) 'The entire card collection survives restart'
    Open-NoteId $menu 1
    Assert-Ui ([CatUi]::Text([CatUi]::GetDlgItem($menu, 420)) -ceq $draft) 'Notes survive a full application restart'
    Assert-Ui ((Read-Data).Contains("T`t0`tTask 12")) 'Tasks survive a full application restart'
    Send-Ui $menu 0x111 433
    $settings = Find-Ui 'CatSettingsWnd'
    $view = [CatUi]::GetDlgItem($settings, 520)
    Assert-Ui ([CatUi]::SendMessage([CatUi]::GetDlgItem($view, 492), 0xF0, [IntPtr]::Zero, [IntPtr]::Zero).ToInt64() -eq 0) 'Automatic updates remain disabled after a full app restart'
    Send-Ui $settings 0x10
    Send-Ui $menu 0x111 412
    Assert-Ui (![CatUi]::IsWindowVisible([CatUi]::GetDlgItem($menu, 413))) 'Clearing tasks hides the obsolete page controls'
    Send-Ui $menu 0x111 423
    while (@(Read-NoteRecords).Count -gt 0) { Send-Ui $menu 0x111 610 }
    Assert-Ui ((Read-Data) -eq '') 'Clearing notes and tasks persists an empty file'
    Write-Output "Screenshots and isolated test data: $outputPath"
} finally {
    if ($backdrop -ne [IntPtr]::Zero) { [void][CatUi]::DestroyWindow($backdrop) }
    [void][CatUi]::SetCursorPos($originalCursor.x, $originalCursor.y)
    if ($appProcess -and !$appProcess.HasExited) {
        $window = [CatUi]::Find($appProcess.Id, 'cocoBarWnd')
        if ($window -ne [IntPtr]::Zero) { Send-Ui $window 0x10 }
        if (!$appProcess.WaitForExit(5000)) { Stop-Process -Id $appProcess.Id -Force }
    }
    $env:COCOBAR_DATA_DIR = $oldDataDir
    $env:COCOBAR_UI_TEST = $oldUiTest
    if ($restoreRunningApp) {
        if (!$releaseBuilt -and (Test-Path -LiteralPath $previousExecutable)) {
            Copy-Item -LiteralPath $previousExecutable -Destination $executablePath -Force
        }
        [void](Start-Process -FilePath $executablePath -WindowStyle Hidden)
        Write-Output 'Reopened the updated app with its original data folder.'
    }
}
