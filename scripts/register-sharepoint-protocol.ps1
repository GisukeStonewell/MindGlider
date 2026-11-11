param(
    [string]$Executable,
    [switch]$Unregister
)

$ErrorActionPreference = 'Stop'
$scheme = 'HKCU:\Software\Classes\mindglider'

if ($Unregister) {
    if (Test-Path -LiteralPath $scheme) {
        Remove-Item -LiteralPath $scheme -Recurse
    }
    return
}

if (!$Executable) {
    throw 'Specify -Executable with the path to mindglider.exe.'
}
$binary = Get-Item -LiteralPath $Executable
if ($binary.PSIsContainer -or $binary.Name -ine 'mindglider.exe') {
    throw 'Specify the MindGlider executable.'
}

New-Item -Path $scheme -Force | Out-Null
Set-Item -LiteralPath $scheme -Value 'URL:MindGlider'
New-ItemProperty -LiteralPath $scheme -Name 'URL Protocol' -Value '' -PropertyType String -Force | Out-Null
$command = Join-Path $scheme 'shell\open\command'
New-Item -Path $command -Force | Out-Null
Set-Item -LiteralPath $command -Value ('"{0}" "%1"' -f $binary.FullName)
