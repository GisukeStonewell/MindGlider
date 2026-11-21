[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$SourceExe,

    [Parameter(Mandatory)]
    [string]$InstallDir
)

$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'The MindGlider executable updater requires Windows.'
}

$source = Get-Item -LiteralPath $SourceExe
$install = Get-Item -LiteralPath $InstallDir
if ($source.PSIsContainer -or !$install.PSIsContainer) {
    throw 'SourceExe must be a file and InstallDir must be a directory.'
}
$destination = Join-Path $install.FullName 'mindglider.exe'
if ($source.FullName -ieq $destination) {
    throw 'SourceExe must be separate from the installed executable.'
}

# Keep one stable lock file so simultaneous updaters cannot move each other's files.
$lockPath = Join-Path $install.FullName '.mindglider-update.lock'
try {
    $lock = [IO.File]::Open($lockPath, 'OpenOrCreate', 'ReadWrite', 'None')
} catch {
    throw "Cannot acquire the MindGlider update lock at $lockPath. $($_.Exception.Message)"
}
try {
    if (!(Test-Path -LiteralPath $destination -PathType Leaf)) {
        throw "Installed MindGlider executable is missing: $destination"
    }
    foreach ($path in @($source.FullName, $destination)) {
        if ([Diagnostics.FileVersionInfo]::GetVersionInfo($path).ProductName -ne 'MindGlider') {
            throw "File is not a MindGlider executable: $path"
        }
    }

    $sourceHash = (Get-FileHash -LiteralPath $source.FullName -Algorithm SHA256).Hash
    $installedHash = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash
    if ($installedHash -eq $sourceHash) {
        return [pscustomobject]@{
            Installed = $destination
            Backup = $null
            Sha256 = $sourceHash
            Changed = $false
        }
    }

    $id = [guid]::NewGuid().ToString('N').Substring(0, 8)
    $backupName = 'backup-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + $id
    $backupDir = (New-Item -Path (Join-Path $install.FullName $backupName) -ItemType Directory).FullName
    $staged = Join-Path $backupDir 'new-mindglider.exe'
    $backup = Join-Path $backupDir 'mindglider.exe'
    Copy-Item -LiteralPath $source.FullName -Destination $staged
    if ((Get-FileHash -LiteralPath $staged -Algorithm SHA256).Hash -ne $sourceHash) {
        throw "Staged executable failed verification: $staged"
    }

    # Same-volume renames preserve the image already mapped by the running process.
    [IO.File]::Move($destination, $backup)
    try {
        [IO.File]::Move($staged, $destination)
    } catch {
        $publishError = $_.Exception.Message
        try {
            [IO.File]::Move($backup, $destination)
        } catch {
            throw "Publishing and rollback failed. Old executable: $backup. Staged executable: $staged. Publish: $publishError. Rollback: $($_.Exception.Message)"
        }
        throw "Publishing the update failed and the previous executable was restored. $publishError"
    }

    [pscustomobject]@{
        Installed = $destination
        Backup = $backup
        Sha256 = $sourceHash
        Changed = $true
    }
} finally {
    $lock.Dispose()
}
