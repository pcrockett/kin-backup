$ErrorActionPreference = "Stop"
Set-StrictMode -Version 5.0

$prepareScript = Join-Path $PSScriptRoot "scripts\prepare-decrypt-exes.ps1"
& $prepareScript

Push-Location $PSScriptRoot
try {

    cargo build --release --package kin
    $result = $LASTEXITCODE
    if ($result -ne 0) {
        throw "cargo exited with code $result"
    }

} finally {
    Pop-Location
}
