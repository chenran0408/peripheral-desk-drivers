[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$publisher = Join-Path $PSScriptRoot 'publish-channels.ps1'
. $publisher -Repository 'chenran0408/peripheral-desk-drivers' -FunctionsOnly
function Assert-True($Value, [string]$Message) { if (-not $Value) { throw $Message } }
function Assert-Fails([scriptblock]$Action, [string]$Pattern) {
    $failed = $false
    try { & $Action | Out-Null } catch { $failed = $true; if ($_.Exception.Message -notmatch $Pattern) { throw } }
    if (-not $failed) { throw "Expected failure: $Pattern" }
}
function New-Entry([string]$Id, [string]$Version, [string]$Channel, [string]$Digit) {
    return [pscustomobject]@{
        id=$Id; version=$Version; channel=$Channel; sha256=($Digit * 64); dll_sha256=('a' * 64)
        package_url="https://github.com/chenran0408/peripheral-desk-drivers/releases/download/v$Version/$Id-$Version-windows-x64.zip"
        device=[pscustomobject]@{ key=$Id; name="设备 $Id"; validation=[pscustomobject]@{status='recognition-only';firmware='unknown'} }
        changelog='原有说明'; future_metadata=[pscustomobject]@{nested=@('保留未知字段',42)}
    }
}
function Catalog([string]$Channel, [object[]]$Entries) { return [pscustomobject]@{schema=1;channel=$Channel;drivers=@($Entries)} }
$a = New-Entry 'mchose-a5-ultra' '0.4.0' 'stable' '1'
$b = New-Entry 'mchose-g75-pro' '0.4.0' 'stable' '2'
$a2 = New-Entry 'mchose-a5-ultra' '0.4.1' 'stable' '3'
$c = New-Entry 'new-device' '0.4.1' 'stable' '4'
$old = Catalog 'stable' @($a,$b)
$before = $old | ConvertTo-Json -Depth 100 -Compress
$result = Merge-DriverCatalog $old (Catalog 'stable' @($a2,$c)) 'stable'
Assert-True ($result.drivers.Count -eq 3) 'Subset dropped an unrelated device.'
Assert-True (($result.drivers[1] | ConvertTo-Json -Depth 100 -Compress) -ceq ($b | ConvertTo-Json -Depth 100 -Compress)) 'Old metadata changed.'
Assert-True (($old | ConvertTo-Json -Depth 100 -Compress) -ceq $before) 'Merge mutated its input.'
$same = $a | ConvertTo-Json -Depth 100 | ConvertFrom-Json
$same.changelog = 'Cannot replace an immutable version metadata'
$same.package_url = $same.package_url.Replace('/v0.4.0/','/v0.4.1/')
Assert-True ((Merge-DriverCatalog $old (Catalog 'stable' @($same)) 'stable').drivers[0].changelog -ceq $a.changelog) 'Same version replaced metadata.'
$conflict = New-Entry $a.id $a.version 'stable' '9'
Assert-Fails { Merge-DriverCatalog $old (Catalog 'stable' @($conflict)) 'stable' } 'different hashes'
Assert-Fails { Merge-DriverCatalog $old (Catalog 'stable' @((New-Entry $a.id '0.3.9' 'stable' '3'))) 'stable' } 'rollback rejected'
Assert-Fails { Merge-DriverCatalog $old (Catalog 'stable' @($a2,$a2)) 'stable' } 'duplicate'
Assert-Fails { Merge-DriverCatalog $old (Catalog 'preview' @()) 'stable' } 'mismatch'
Assert-Fails { Get-DriverVersion '../0.4.1' } 'Invalid'
Assert-True ((Compare-DriverVersion '0.4.1-preview.10' '0.4.1-preview.2') -gt 0) 'Numeric prerelease order.'
Assert-True ((Compare-DriverVersion '0.4.1' '0.4.1-preview.10') -gt 0) 'Stable must follow its prereleases.'
$badUrl = New-Entry 'new-device' '0.4.1' 'stable' '4'
$badUrl.package_url = 'https://github.com/foreign/repository/actions/artifacts/1'
Assert-Fails { Merge-DriverCatalog $null (Catalog 'stable' @($badUrl)) 'stable' } 'immutable official'

# Signed offline fixtures use a fresh ephemeral test key, never a publishing key.
$base = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
$temporary = Join-Path $base ('pd-channel-test-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $temporary | Out-Null
$savedKey = $env:DRIVER_CATALOG_SIGNING_KEY
$rsa = [Security.Cryptography.RSA]::Create(2048)
try {
    $env:DRIVER_CATALOG_SIGNING_KEY = $rsa.ToXmlString($true)
    $public = Join-Path $temporary 'public.blob'
    $parameters = $rsa.ExportParameters($false)
    $stream = [IO.MemoryStream]::new()
    $writer = [IO.BinaryWriter]::new($stream)
    foreach ($number in @([uint32]0x31415352,[uint32]2048,[uint32]$parameters.Exponent.Length,[uint32]$parameters.Modulus.Length,[uint32]0,[uint32]0)) { $writer.Write([uint32]$number) }
    $writer.Write([byte[]]$parameters.Exponent); $writer.Write([byte[]]$parameters.Modulus)
    [IO.File]::WriteAllBytes($public,$stream.ToArray()); $writer.Dispose(); $stream.Dispose()
    function Write-Fixtures([string]$Directory, [object[]]$Stable, [object[]]$Preview) {
        New-Item -ItemType Directory -Path $Directory -Force | Out-Null
        foreach ($channel in @('stable','preview')) {
            $entries = if ($channel -eq 'stable') { @($Stable) } else { @($Preview) }
            $text = (Catalog $channel @($entries)) | ConvertTo-Json -Depth 100
            [IO.File]::WriteAllText((Join-Path $Directory "catalog-$channel.json"),$text,[Text.UTF8Encoding]::new($false))
        }
        Copy-Item -LiteralPath (Join-Path $Directory 'catalog-stable.json') -Destination (Join-Path $Directory 'catalog.json') -Force
        & (Join-Path $PSScriptRoot 'sign-catalog.ps1') -CatalogDirectory $Directory | Out-Null
    }
    $incoming = Join-Path $temporary 'incoming'
    $oldSigned = Join-Path $temporary 'old-signed'
    $previewEntry = New-Entry 'logitech-g305' '0.4.1-preview.1' 'preview' '5'
    Write-Fixtures $oldSigned @($a,$b) @($previewEntry)
    Write-Fixtures $incoming @($a2) @()
    $global:PdFixtureRemote = Join-Path $temporary 'remote'
    foreach ($channel in @('stable','preview')) {
        $directory = Join-Path $global:PdFixtureRemote "catalog-$channel"
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
        Copy-Item -LiteralPath (Join-Path $oldSigned "catalog-$channel.json"),(Join-Path $oldSigned "catalog-$channel.json.sig") -Destination $directory
    }
    $global:PdFixtureUploads = 0; $global:PdFixtureMode = 'normal'
    function global:gh {
        $arguments = @($args)
        $global:LASTEXITCODE = 0
        if ($arguments[0] -eq 'api') {
            if ($global:PdFixtureMode -eq 'network-error') { $global:LASTEXITCODE=1; return 'network unreachable' }
            $tag = ($arguments[1] -split '/')[-1]
            if (Test-Path -LiteralPath (Join-Path $global:PdFixtureRemote $tag)) { return '{"id":1}' }
            $global:LASTEXITCODE=1; return 'gh: Not Found (HTTP 404)'
        }
        if ($arguments[0] -ne 'release') { throw 'Offline fixture refuses any unknown gh call.' }
        $directory = Join-Path $global:PdFixtureRemote $arguments[2]
        if ($arguments[1] -eq 'download') {
            $destination = $arguments[([Array]::IndexOf($arguments,'--dir')+1)]
            for ($i=0;$i -lt $arguments.Count;$i++) {
                if ($arguments[$i] -eq '--pattern') { Copy-Item -LiteralPath (Join-Path $directory $arguments[$i+1]) -Destination $destination }
            }
        } elseif ($arguments[1] -in @('upload','create')) {
            $global:PdFixtureUploads++
            New-Item -ItemType Directory -Path $directory -Force | Out-Null
            foreach ($argument in $arguments) {
                if (Test-Path -LiteralPath $argument -PathType Leaf) { Copy-Item -LiteralPath $argument -Destination $directory -Force }
            }
        } else { throw 'Offline fixture refuses any unknown gh operation.' }
    }
    $sourceHashes = @(Get-ChildItem -LiteralPath $incoming -File | ForEach-Object { (Get-FileHash -LiteralPath $_.FullName).Hash }) -join ','
    $previewHash = (Get-FileHash -LiteralPath (Join-Path $global:PdFixtureRemote 'catalog-preview/catalog-preview.json')).Hash
    & $publisher -Repository 'chenran0408/peripheral-desk-drivers' -CatalogDirectory $incoming -PublicKeyPath $public | Out-Null
    Assert-True ($global:PdFixtureUploads -eq 1) 'Empty preview should not be overwritten.'
    $stableDirectory = Join-Path $global:PdFixtureRemote 'catalog-stable'
    & (Join-Path $PSScriptRoot 'verify-signatures.ps1') -CatalogDirectory $stableDirectory -PublicKeyPath $public -CatalogName 'catalog-stable.json','catalog.json' | Out-Null
    $published = Get-Content -LiteralPath (Join-Path $stableDirectory 'catalog-stable.json') -Raw | ConvertFrom-Json
    Assert-True ($published.drivers.Count -eq 2 -and $published.drivers[0].version -eq '0.4.1') 'Signed merged channel is incomplete.'
    Assert-True (($published.drivers[1] | ConvertTo-Json -Depth 100 -Compress) -ceq ($b | ConvertTo-Json -Depth 100 -Compress)) 'Signed merge lost old metadata.'
    Assert-True (((Get-FileHash -LiteralPath (Join-Path $global:PdFixtureRemote 'catalog-preview/catalog-preview.json')).Hash) -eq $previewHash) 'Stable subset changed preview.'
    Assert-True ((@(Get-ChildItem -LiteralPath $incoming -File | ForEach-Object { (Get-FileHash -LiteralPath $_.FullName).Hash }) -join ',') -eq $sourceHashes) 'Version-release subset or signature was changed.'
    $stableHash = (Get-FileHash -LiteralPath (Join-Path $stableDirectory 'catalog-stable.json')).Hash
    $preview2 = New-Entry 'logitech-g305' '0.4.1-preview.2' 'preview' '6'
    Write-Fixtures $incoming @() @($preview2)
    & $publisher -Repository 'chenran0408/peripheral-desk-drivers' -CatalogDirectory $incoming -PublicKeyPath $public | Out-Null
    Assert-True ($global:PdFixtureUploads -eq 2) 'Preview subset should update only preview.'
    Assert-True (((Get-FileHash -LiteralPath (Join-Path $stableDirectory 'catalog-stable.json')).Hash) -eq $stableHash) 'Preview subset changed stable.'
    Write-Fixtures $incoming @((New-Entry $a.id '0.4.2' 'stable' '7')) @()
    Add-Content -LiteralPath (Join-Path $stableDirectory 'catalog-stable.json') -Value ' '
    Assert-Fails { & $publisher -Repository 'chenran0408/peripheral-desk-drivers' -CatalogDirectory $incoming -PublicKeyPath $public } 'Signature verification failed'
    Assert-True ($global:PdFixtureUploads -eq 2) 'Invalid old signature reached upload.'
    Copy-Item -LiteralPath (Join-Path $oldSigned 'catalog-stable.json') -Destination (Join-Path $stableDirectory 'catalog-stable.json') -Force
    $oldSignature = Join-Path $stableDirectory 'catalog-stable.json.sig'
    Remove-Item -LiteralPath $oldSignature -Force
    Assert-Fails { & $publisher -Repository 'chenran0408/peripheral-desk-drivers' -CatalogDirectory $incoming -PublicKeyPath $public } '.'
    Assert-True ($global:PdFixtureUploads -eq 2) 'Missing old signature reached upload.'
    $global:PdFixtureMode = 'network-error'
    Assert-Fails { & $publisher -Repository 'chenran0408/peripheral-desk-drivers' -CatalogDirectory $incoming -PublicKeyPath $public } 'Cannot determine'
    Assert-True ($global:PdFixtureUploads -eq 2) 'Network error was treated as an empty old directory.'
    $global:PdFixtureMode = 'normal'
    $global:PdFixtureRemote = Join-Path $temporary 'initial-remote'
    & $publisher -Repository 'chenran0408/peripheral-desk-drivers' -CatalogDirectory $incoming -PublicKeyPath $public | Out-Null
    Assert-True ($global:PdFixtureUploads -eq 4) 'First publication should create both canonical channels.'
    $first = Get-Content -LiteralPath (Join-Path $global:PdFixtureRemote 'catalog-stable/catalog-stable.json') -Raw | ConvertFrom-Json
    Assert-True ($first.drivers.Count -eq 1 -and $first.drivers[0].version -eq '0.4.2') 'Initial subset publication failed.'
    Write-Output 'Passed offline channel fixtures: subset preservation, metadata, immutable versions, ordering, re-signing, channel isolation, input preservation, initial publication, tamper, missing signature and network failure.'
} finally {
    $env:DRIVER_CATALOG_SIGNING_KEY = $savedKey
    $rsa.Dispose()
    Remove-Item -LiteralPath Function:\global:gh -ErrorAction SilentlyContinue
    Remove-Variable -Scope Global -Name PdFixtureRemote,PdFixtureUploads,PdFixtureMode -ErrorAction SilentlyContinue
    $resolved = [IO.Path]::GetFullPath($temporary)
    if (-not $resolved.StartsWith($base,[StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($resolved) -notmatch '^pd-channel-test-[0-9a-f]{32}$') { throw 'Refusing unsafe fixture cleanup.' }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
