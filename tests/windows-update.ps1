param([switch]$Live)
$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$root = Join-Path $workspace ('target\update-check\' + [guid]::NewGuid())
[void](New-Item -ItemType Directory -Path $root)
$utf8 = [Text.UTF8Encoding]::new($true)
function Assert-Update([bool]$condition, [string]$message) {
    if (!$condition) { throw "FAIL: $message" }
    Write-Output "PASS: $message"
}
function Quote-Update([string]$value) { "'" + $value.Replace("'", "''") + "'" }
$rustc = Join-Path $env:USERPROFILE '.cargo\bin\rustc.exe'
$fixtures = @{}
foreach ($kind in @('old', 'new', 'fail')) {
    $file = Join-Path $root "$kind.exe"
    $arguments = @((Join-Path $PSScriptRoot 'fixtures\update-app.rs'), '--edition=2021', '-O', '-o', $file)
    if ($kind -eq 'new') { $arguments += @('--cfg', 'new_version') }
    if ($kind -eq 'fail') { $arguments += @('--cfg', 'fail_startup') }
    & $rustc @arguments
    if ($LASTEXITCODE -ne 0) { throw "Could not build $kind update fixture" }
    $fixtures[$kind] = $file
}
$helperBody = Get-Content (Join-Path $workspace 'src\install-update.ps1') -Raw
foreach ($kind in @('success', 'wait-for-exit', 'rollback', 'cancel', 'read-only', 'replace-denied')) {
    # Spaces, apostrophes, and Unicode exercise the production script quoting.
    $case = Join-Path $root "$kind O'Brien 猫"
    $stageDir = Join-Path $case 'stage'
    [void](New-Item -ItemType Directory -Path $stageDir -Force)
    $exe = Join-Path $case 'cocobar.exe'
    Copy-Item -LiteralPath $fixtures.old -Destination $exe
    $oldHash = (Get-FileHash -LiteralPath $exe).Hash
    $newFixture = if ($kind -eq 'rollback') { $fixtures.fail } else { $fixtures.new }
    $staged = Join-Path $stageDir 'cocobar.exe'
    Copy-Item -LiteralPath $newFixture -Destination $staged
    $expectedHash = (Get-FileHash -LiteralPath $staged).Hash
    $backup = $exe + '.update-backup'
    [IO.File]::WriteAllText($backup, 'stale backup must never be used')
    $candidate = $exe + '.test.new'
    $ready = Join-Path $stageDir 'ready'
    $go = Join-Path $stageDir 'go'
    $cancel = Join-Path $stageDir 'cancel'
    $errorPath = Join-Path $stageDir 'error'
    $resultPath = Join-Path $case 'update-result.txt'
    $helperPath = Join-Path $stageDir 'install.ps1'
    $notes = Join-Path $case 'mydata.txt'
    [IO.File]::WriteAllText($notes, "N`tKeep these notes`nT`t0`tKeep this task")
    $dataHash = (Get-FileHash -LiteralPath $notes).Hash
    $oldPid = 0
    $oldProcess = $null
    $helper = $null
    try {
        if ($kind -eq 'read-only') { (Get-Item -LiteralPath $exe).IsReadOnly = $true }
        if ($kind -eq 'wait-for-exit') {
            $oldProcess = Start-Process -FilePath $exe -WorkingDirectory $case -WindowStyle Hidden -PassThru
            $oldPid = $oldProcess.Id
        }
        $prefix = ''
        foreach ($name in @('exe', 'staged', 'candidate', 'backup', 'stageDir', 'helperPath', 'ready', 'go', 'cancel', 'errorPath', 'resultPath')) {
            $prefix += '$' + $name + ' = ' + (Quote-Update (Get-Variable -Name $name -ValueOnly)) + "`n"
        }
        $prefix += '$oldPid = ' + $oldPid + "`n" + '$version = ''v9.0.0''' + "`n"
        [IO.File]::WriteAllText($helperPath, $prefix + $helperBody, $utf8)
        $helper = Start-Process -FilePath 'powershell.exe' -ArgumentList @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', ('"' + $helperPath + '"')) -RedirectStandardError (Join-Path $case 'helper-error.txt') -RedirectStandardOutput (Join-Path $case 'helper-output.txt') -WindowStyle Hidden -PassThru
        $deadline = [DateTime]::UtcNow.AddSeconds(15)
        while (!(Test-Path -LiteralPath $ready) -and !$helper.HasExited -and [DateTime]::UtcNow -lt $deadline) { Start-Sleep -Milliseconds 100; $helper.Refresh() }
        Assert-Update ((Get-FileHash -LiteralPath $exe).Hash -eq $oldHash) "$kind preserves the executable before authorization"
        if ($kind -eq 'read-only') {
            Assert-Update ($helper.WaitForExit(10000)) 'A denied replacement exits before requesting app shutdown'
            Assert-Update (!(Test-Path -LiteralPath $ready)) 'A read-only install never signals ready'
            Assert-Update ((Get-Content -LiteralPath $resultPath -Raw) -match 'read-only') 'A read-only install reports the actual error'
        } else {
            Assert-Update (Test-Path -LiteralPath $ready) "$kind prepares the replacement before signaling ready"
            if ($kind -eq 'replace-denied') { (Get-Item -LiteralPath $exe).IsReadOnly = $true }
            if ($kind -eq 'cancel') { [IO.File]::WriteAllText($cancel, 'cancel') }
            else { [IO.File]::WriteAllText($go, 'go') }
            if ($kind -eq 'wait-for-exit') {
                Start-Sleep -Milliseconds 500
                Assert-Update ((Get-FileHash -LiteralPath $exe).Hash -eq $oldHash) 'Replacement waits until the original process exits'
                [IO.File]::WriteAllText((Join-Path $case 'old-stop'), 'stop')
                Assert-Update ($oldProcess.WaitForExit(5000)) 'The disposable original process closes normally'
            }
            Assert-Update ($helper.WaitForExit(15000)) "$kind helper completes"
            if ($kind -eq 'cancel') {
                Assert-Update ((Get-FileHash -LiteralPath $exe).Hash -eq $oldHash) 'Canceling leaves the original executable untouched'
                Assert-Update (!(Test-Path -LiteralPath (Join-Path $case 'started-new.txt'))) 'Canceling never starts the staged executable'
            } elseif ($kind -eq 'replace-denied') {
                Assert-Update ((Get-FileHash -LiteralPath $exe).Hash -eq $oldHash) 'A failed replacement keeps the original executable'
                Assert-Update (Test-Path -LiteralPath (Join-Path $case 'started-old.txt')) 'A failed replacement reopens the original application'
            } elseif ($kind -eq 'rollback') {
                Assert-Update ((Get-FileHash -LiteralPath $exe).Hash -eq $oldHash) 'A startup failure restores the original executable'
                Assert-Update (Test-Path -LiteralPath (Join-Path $case 'started-old.txt')) 'Rollback relaunches the original executable'
                Assert-Update ((Get-Content -LiteralPath $resultPath -Raw) -match 'restored') 'Rollback reports its result'
                Assert-Update ((Get-FileHash -LiteralPath $backup).Hash -eq $oldHash) 'Rollback uses the current original, never a stale backup'
            } else {
                Assert-Update ((Get-FileHash -LiteralPath $exe).Hash -eq $expectedHash) "$kind atomically installs the verified executable"
                Assert-Update ((Get-FileHash -LiteralPath $backup).Hash -eq $oldHash) "$kind backs up the current original"
                Assert-Update (Test-Path -LiteralPath (Join-Path $case 'started-new.txt')) "$kind starts the new executable"
                Assert-Update ((Get-Content -LiteralPath $resultPath -Raw) -match 'Updated to v9.0.0') "$kind reports successful installation"
            }
        }
        Assert-Update ((Get-FileHash -LiteralPath $notes).Hash -eq $dataHash) "$kind preserves notes and tasks"
        Assert-Update (!(Test-Path -LiteralPath $candidate)) "$kind cleans up the replacement candidate"
    } finally {
        (Get-Item -LiteralPath $exe).IsReadOnly = $false
        # Stop only fixture processes created in this exact disposable case.
        foreach ($marker in @('started-new.txt', 'started-old.txt')) {
            $markerPath = Join-Path $case $marker
            if (Test-Path -LiteralPath $markerPath) {
                $fixturePid = [int](Get-Content -LiteralPath $markerPath)
                $fixtureProcess = Get-Process -Id $fixturePid -ErrorAction SilentlyContinue
                if ($null -ne $fixtureProcess -and $fixtureProcess.Path -eq $exe) { Stop-Process -Id $fixturePid }
            }
        }
        if ($null -ne $helper -and !$helper.HasExited) { [IO.File]::WriteAllText($cancel, 'cancel'); [void]$helper.WaitForExit(5000) }
    }
}
if ($Live) {
    $liveScript = Join-Path $root 'check-live.ps1'
    $download = Join-Path $root 'github-cocobar.exe'
    $metadata = Join-Path $root 'github-release.txt'
    $liveBody = @'
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$line = & $checkScript
[IO.File]::WriteAllText($metadata, $line)
$fields = $line.Split('|')
if ($fields.Length -ne 4 -or !$fields[1]) { throw 'Missing GitHub executable' }
Invoke-WebRequest -Uri $fields[1] -OutFile $download -UseBasicParsing -TimeoutSec 120
if ((Get-Item -LiteralPath $download).Length -ne [long]$fields[2]) { throw 'Download size mismatch' }
if ($fields[3] -and ('sha256:' + (Get-FileHash -LiteralPath $download -Algorithm SHA256).Hash.ToLowerInvariant()) -ne $fields[3]) { throw 'Checksum mismatch' }
$bytes = [IO.File]::ReadAllBytes($download)
if ($bytes[0] -ne 77 -or $bytes[1] -ne 90) { throw 'Invalid Windows executable' }
$offset = [BitConverter]::ToUInt32($bytes, 60)
if ([BitConverter]::ToUInt32($bytes, $offset) -ne 17744 -or [BitConverter]::ToUInt16($bytes, $offset + 4) -ne 34404) { throw 'Invalid PE headers or architecture' }
'Verified GitHub ' + $fields[0] + ': size, SHA-256 and Windows executable headers match.'
'@
    $prefix = '$checkScript = ' + (Quote-Update (Join-Path $workspace 'src\check-release.ps1')) + "`n" + '$download = ' + (Quote-Update $download) + "`n" + '$metadata = ' + (Quote-Update $metadata) + "`n"
    [IO.File]::WriteAllText($liveScript, $prefix + $liveBody, $utf8)
    & powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $liveScript
    if ($LASTEXITCODE -ne 0) { throw 'Live GitHub update verification failed' }
    Assert-Update (Test-Path -LiteralPath $download) 'The real GitHub release downloads and verifies without replacing the user app'
}
Write-Output "Update artifacts: $root"
