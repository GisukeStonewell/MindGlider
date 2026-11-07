param([switch]$Package, [switch]$Test)
$ErrorActionPreference = 'Stop'
$projectDir = Split-Path -Parent $PSScriptRoot
Push-Location $projectDir
try {
    $localCargo = Join-Path $projectDir '.tools/cargo/bin/cargo.exe'
    if (Test-Path -LiteralPath $localCargo) {
        $env:CARGO_HOME = Join-Path $projectDir '.tools/cargo'
        $env:RUSTUP_HOME = Join-Path $projectDir '.tools/rustup'
        $cargoExe = $localCargo
    } else { $cargoExe = (Get-Command cargo -ErrorAction Stop).Source }
    New-Item -ItemType Directory -Force artifacts/tmp | Out-Null
    $env:TEMP = Join-Path $projectDir 'artifacts/tmp'
    $env:TMP = $env:TEMP
    if ($Test) {
        & $cargoExe test --locked --tests
        if ($LASTEXITCODE -ne 0) { throw 'Tests failed.' }
    }
    & $cargoExe build --locked --release --bin mindglider
    if ($LASTEXITCODE -ne 0) { throw 'Release build failed.' }
    if ($Package) {
        $versionMatch = Select-String -Path 'Cargo.toml' -Pattern '^version = "([^"]+)"' | Select-Object -First 1
        $bundleName = 'MindGlider-' + $versionMatch.Matches.Groups[1].Value + '-windows-x64'

        # Regenerate the bundled map with the current writer, never ship a stale header.
        & $cargoExe run --locked --example showcase -- artifacts/showcase
        if ($LASTEXITCODE -ne 0) { throw 'Example generation failed.' }
        # A fresh staging directory prevents packaging saved settings or CLI history.
        $bundleDir = Join-Path $projectDir ('artifacts/package-staging/' + [guid]::NewGuid().ToString() + '/' + $bundleName)
        New-Item -ItemType Directory -Force $bundleDir | Out-Null
        $publicFiles = @('README.md','README.en.md','FEATURES.md','LICENSE.txt','NOTICE.md','THIRD_PARTY.md','CONTRIBUTING.md','SECURITY.md','CHANGELOG.md')
        Copy-Item -LiteralPath 'target/release/mindglider.exe' -Destination $bundleDir
        Copy-Item -LiteralPath $publicFiles -Destination $bundleDir
        foreach ($folder in @('docs','locales','licenses','assets')) {
            Copy-Item -LiteralPath $folder -Destination $bundleDir -Recurse
        }
        $exampleDir = Join-Path $bundleDir 'examples'
        New-Item -ItemType Directory -Force $exampleDir | Out-Null
        Copy-Item -LiteralPath 'artifacts/showcase/showcase.mm' -Destination (Join-Path $exampleDir 'Example.mm')
        New-Item -ItemType Directory -Force dist | Out-Null
        $archive = Join-Path $projectDir ('dist/' + $bundleName + '.zip')
        # ZipFile includes dotfiles, unlike Compress-Archive on Windows.
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $temporaryArchive = Join-Path (Split-Path -Parent $bundleDir) 'package.zip'
        [System.IO.Compression.ZipFile]::CreateFromDirectory($bundleDir, $temporaryArchive)
        Move-Item -LiteralPath $temporaryArchive -Destination $archive -Force
        $hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
        [System.IO.File]::WriteAllText((Join-Path $projectDir 'dist/SHA256SUMS.txt'), "$hash  $bundleName.zip`n", [System.Text.UTF8Encoding]::new($false))
        python scripts/verify_package.py $archive
        if ($LASTEXITCODE -ne 0) { throw 'Package validation failed.' }
        Write-Output "Package: $archive ($hash)"
    }
} finally { Pop-Location }
