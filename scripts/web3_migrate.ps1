param([switch]$Apply)

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$envPath = Join-Path $repoRoot '.env.migration'
if (-not (Test-Path -LiteralPath $envPath)) { throw 'Missing backend/.env.migration' }
$line = Get-Content -LiteralPath $envPath | Where-Object { $_ -like 'MIGRATION_DATABASE_URL=*' } | Select-Object -First 1
if (-not $line) { throw 'Missing MIGRATION_DATABASE_URL' }
$migrationUrl = $line.Substring('MIGRATION_DATABASE_URL='.Length)
$uri = [uri]$migrationUrl
if ($uri.AbsolutePath.Trim('/') -ne 'ihuza_web3_testnet' -or $uri.Host -ne '127.0.0.1') {
    throw 'Migration URL must point to local ihuza_web3_testnet'
}
$previousUrl = [System.Environment]::GetEnvironmentVariable('DATABASE_URL', 'Process')
try {
    $env:DATABASE_URL = $migrationUrl
    Push-Location $repoRoot
    try {
        if ($Apply) { sqlx migrate run --source db/web3_testnet }
        else { sqlx migrate info --source db/web3_testnet }
        if ($LASTEXITCODE -ne 0) { throw 'sqlx migration command failed' }
    } finally {
        Pop-Location
    }
} finally {
    if ($null -eq $previousUrl) { Remove-Item Env:DATABASE_URL -ErrorAction SilentlyContinue }
    else { $env:DATABASE_URL = $previousUrl }
}
