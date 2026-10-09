# The Rust worker supplies literal paths and waits for the ready marker. The
# running app authorizes replacement only after its final data/settings save.
$ErrorActionPreference = 'Stop'
$replacementMade = $false
$authorized = $false
$newProcess = $null
try {
    if ((Get-Item -LiteralPath $exe).IsReadOnly) { throw 'The application executable is read-only.' }
    Copy-Item -LiteralPath $staged -Destination $candidate -ErrorAction Stop
    [IO.File]::WriteAllText($ready, 'ready')
    $deadline = [DateTime]::UtcNow.AddSeconds(60)
    while (!(Test-Path -LiteralPath $go)) {
        if ((Test-Path -LiteralPath $cancel) -or [DateTime]::UtcNow -gt $deadline) { return }
        if ($oldPid -ne 0 -and !(Get-Process -Id $oldPid -ErrorAction SilentlyContinue)) { return }
        Start-Sleep -Milliseconds 100
    }
    $authorized = $true
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while ($oldPid -ne 0 -and (Get-Process -Id $oldPid -ErrorAction SilentlyContinue)) {
        if ([DateTime]::UtcNow -gt $deadline) { throw 'The running application did not close.' }
        Start-Sleep -Milliseconds 100
    }
    # Candidate and executable share a directory, so replacement is atomic.
    # Only this successful replacement creates the backup used for rollback.
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        try {
            [IO.File]::Replace($candidate, $exe, $backup, $true)
            $replacementMade = $true
            break
        } catch {
            if ($attempt -eq 29) { throw }
            Start-Sleep -Milliseconds 200
        }
    }
    [IO.File]::WriteAllText($resultPath, "Updated to $version.")
    $newProcess = Start-Process -FilePath $exe -WorkingDirectory (Split-Path -LiteralPath $exe) -WindowStyle Hidden -PassThru -ErrorAction Stop
    if ($newProcess.WaitForExit(3000)) { throw 'The updated application exited during startup.' }
} catch {
    $message = $_.Exception.Message
    if ($replacementMade) {
        try {
            if ($null -ne $newProcess -and !$newProcess.HasExited) { throw 'The new application is still running.' }
            Copy-Item -LiteralPath $backup -Destination $candidate -ErrorAction Stop
            # PowerShell 5 coerces a null string argument into an empty path.
            [IO.File]::Replace($candidate, $exe, ($candidate + '.failed'), $true)
            [IO.File]::WriteAllText($resultPath, "Update failed; restored the previous version. $message")
            Start-Process -FilePath $exe -WorkingDirectory (Split-Path -LiteralPath $exe) -WindowStyle Hidden -ErrorAction Stop
            $message = "Update failed; restored the previous version. $message"
        } catch { $message = "Update failed. Backup: $backup. $message $($_.Exception.Message)" }
    } elseif ($authorized -and ($oldPid -eq 0 -or !(Get-Process -Id $oldPid -ErrorAction SilentlyContinue))) {
        # Replacement can fail after the app exits (for example an antivirus
        # file lock). In that case the unchanged original must reopen too.
        try {
            [IO.File]::WriteAllText($resultPath, "Update failed; kept the previous version. $message")
            Start-Process -FilePath $exe -WorkingDirectory (Split-Path -LiteralPath $exe) -WindowStyle Hidden -ErrorAction Stop
        } catch { $message += " $($_.Exception.Message)" }
    }
    [IO.File]::WriteAllText($errorPath, $message)
    [IO.File]::WriteAllText($resultPath, $message)
} finally {
    foreach ($file in @($candidate, ($candidate + '.failed'), $staged, $ready, $go, $cancel, $helperPath)) {
        Remove-Item -LiteralPath $file -Force -ErrorAction SilentlyContinue
    }
    # A failed preparation keeps its error for the Rust worker to read.
    if ($authorized -or !(Test-Path -LiteralPath $errorPath)) {
        Remove-Item -LiteralPath $errorPath -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $stageDir -ErrorAction SilentlyContinue
    }
}
