$ErrorActionPreference = "Stop"
Set-StrictMode -Version 5.0

$RootDir = Split-Path $PSScriptRoot
$StagingDir = Join-Path $RootDir "kin_decrypt\all_executables"
$DestZip = Join-Path $RootDir "kin\src\compile\decrypt_executables.zip"

Push-Location $RootDir
try {
    cargo build --release --package kin_decrypt
    $result = $LASTEXITCODE
    if ($result -ne 0) {
        throw "Cargo exited with code $result"
    }
} finally {
    Pop-Location
}

Copy-Item "$RootDir\target\release\decrypt.exe" "$StagingDir\decrypt-windows.exe"

if (Test-Path $DestZip) {
    Remove-Item $DestZip
}

$ignoreFiles = ".gitignore", "README.md"
$filesToPackage = Get-ChildItem $StagingDir `
    | Where-Object { $ignoreFiles -notcontains $_.Name } `
    | ForEach-Object { $_.FullName }

Compress-Archive -Path $filesToPackage `
    -DestinationPath $DestZip `
    -CompressionLevel Optimal
