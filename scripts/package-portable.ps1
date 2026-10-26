$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$sourceExe = Join-Path $projectRoot 'src-tauri/target/release/rust-office-modern.exe'
$outputDir = Join-Path $projectRoot 'dist'
if (!(Test-Path -LiteralPath $sourceExe)) {
    throw 'EXE not found. Run npm run desktop:build first.'
}
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
$outputExe = Join-Path $outputDir 'RustOfficeModern.exe'
Copy-Item -LiteralPath $sourceExe -Destination $outputExe -Force
Copy-Item -LiteralPath (Join-Path $projectRoot 'README.md') -Destination (Join-Path $outputDir 'README.md') -Force
$hash = (Get-FileHash -LiteralPath $outputExe -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash  RustOfficeModern.exe" | Set-Content -LiteralPath (Join-Path $outputDir 'SHA256SUMS.txt') -Encoding ascii
$signature = Get-AuthenticodeSignature -LiteralPath $outputExe
$report = [ordered]@{
    file = 'RustOfficeModern.exe'
    sha256 = $hash
    signatureStatus = $signature.Status.ToString()
    publisher = $(if ($signature.SignerCertificate) { $signature.SignerCertificate.Subject } else { $null })
    commit = $env:GITHUB_SHA
    runUrl = $(if ($env:GITHUB_ACTIONS) { "$env:GITHUB_SERVER_URL/$env:GITHUB_REPOSITORY/actions/runs/$env:GITHUB_RUN_ID" } else { $null })
}
$report | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $outputDir 'BUILD-INFO.json') -Encoding utf8
Write-Host "EXE: $outputExe"
Write-Host "SHA256: $hash"
Write-Host "Authenticode status: $($signature.Status)"
if ($signature.Status -ne 'Valid') { Write-Warning 'This build does not have a valid trusted signature. Browser warnings may still occur.' }
