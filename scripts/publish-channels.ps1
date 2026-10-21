[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Repository,
    [string]$CatalogDirectory = (Join-Path $PSScriptRoot "../dist"),
    [string]$PublicKeyPath = (Join-Path $PSScriptRoot "../assets/catalog-public-key.blob"),
    [switch]$FunctionsOnly
)
$ErrorActionPreference = "Stop"
function Get-DriverVersion {
    param([string]$Value)
    if ($Value.Length -gt 64 -or $Value -cnotmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$') { throw "Invalid driver version: $Value" }
    $core = @($Matches[1], $Matches[2], $Matches[3])
    $pre = if ($Matches[4]) { @($Matches[4].Split('.')) } else { @() }
    foreach ($part in $pre) {
        if ($part -cmatch '^[0-9]+$' -and $part.Length -gt 1 -and $part[0] -eq '0') { throw "Invalid numeric prerelease identifier." }
    }
    return [pscustomobject]@{ Core = $core; Pre = @($pre) }
}
function Compare-DriverVersion {
    param([string]$Left, [string]$Right)
    $a = Get-DriverVersion $Left
    $b = Get-DriverVersion $Right
    for ($i = 0; $i -lt 3; $i++) {
        $result = ([Numerics.BigInteger]::Parse($a.Core[$i])).CompareTo([Numerics.BigInteger]::Parse($b.Core[$i]))
        if ($result -ne 0) { return [Math]::Sign($result) }
    }
    if ($a.Pre.Count -eq 0 -and $b.Pre.Count -eq 0) { return 0 }
    if ($a.Pre.Count -eq 0) { return 1 }
    if ($b.Pre.Count -eq 0) { return -1 }
    for ($i = 0; $i -lt [Math]::Min($a.Pre.Count, $b.Pre.Count); $i++) {
        $an = $a.Pre[$i] -cmatch '^[0-9]+$'
        $bn = $b.Pre[$i] -cmatch '^[0-9]+$'
        if ($an -and $bn) { $result = ([Numerics.BigInteger]::Parse($a.Pre[$i])).CompareTo([Numerics.BigInteger]::Parse($b.Pre[$i])) }
        elseif ($an -ne $bn) { $result = if ($an) { -1 } else { 1 } }
        else { $result = [StringComparer]::Ordinal.Compare($a.Pre[$i], $b.Pre[$i]) }
        if ($result -ne 0) { return [Math]::Sign($result) }
    }
    return [Math]::Sign($a.Pre.Count - $b.Pre.Count)
}
function Assert-DriverCatalog {
    param($Catalog, [ValidateSet('stable', 'preview')][string]$Channel)
    if ($null -eq $Catalog -or $Catalog.schema -ne 1 -or $Catalog.channel -cne $Channel -or $null -eq $Catalog.drivers) { throw "Channel/schema/drivers mismatch." }
    $ids = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($entry in @($Catalog.drivers)) {
        if ($entry.id -cnotmatch '^[a-z0-9]+(?:-[a-z0-9]+)*$' -or -not $ids.Add($entry.id)) { throw "Invalid or duplicate driver id." }
        $null = Get-DriverVersion $entry.version
        if ($entry.channel -cne $Channel -or $entry.sha256 -notmatch '^[0-9a-fA-F]{64}$' -or $entry.dll_sha256 -notmatch '^[0-9a-fA-F]{64}$') { throw "Invalid driver channel/hash." }
        $prefix = '^https://github\.com/chenran0408/peripheral-desk-drivers/releases/download/v[0-9]+\.[0-9]+\.[0-9]+(?:[-.][a-zA-Z0-9.-]+)?/'
        $filename = [regex]::Escape($entry.id + '-' + $entry.version + '-windows-x64.zip')
        if ($entry.package_url -cnotmatch ($prefix + $filename + '$')) { throw "Package must remain in an immutable official version release." }
    }
}
function Merge-DriverCatalog {
    param($Previous, $Incoming, [ValidateSet('stable', 'preview')][string]$Channel)
    Assert-DriverCatalog $Incoming $Channel
    if ($null -ne $Previous) { Assert-DriverCatalog $Previous $Channel }
    $entries = [Collections.Generic.List[object]]::new()
    $indices = [Collections.Generic.Dictionary[string, int]]::new([StringComparer]::Ordinal)
    if ($null -ne $Previous) {
        foreach ($entry in @($Previous.drivers)) { $indices.Add($entry.id, $entries.Count); $entries.Add($entry) }
    }
    foreach ($entry in @($Incoming.drivers)) {
        if ($indices.ContainsKey($entry.id)) {
            $index = $indices[$entry.id]; $old = $entries[$index]
            $comparison = Compare-DriverVersion $entry.version $old.version
            if ($comparison -lt 0) { throw "Driver version rollback rejected: $($entry.id) $($entry.version) < $($old.version)" }
            if ($comparison -eq 0) {
                if ($entry.sha256 -ine $old.sha256 -or $entry.dll_sha256 -ine $old.dll_sha256) { throw "Immutable driver version has different hashes: $($entry.id) $($entry.version)" }
                # Keep every old field, including its URL and future metadata.
                continue
            }
            $entries[$index] = $entry
        } else { $indices.Add($entry.id, $entries.Count); $entries.Add($entry) }
    }
    return [pscustomobject]@{ schema = 1; channel = $Channel; drivers = @($entries.ToArray()) }
}
if ($FunctionsOnly) { return }
if ($Repository -cne "chenran0408/peripheral-desk-drivers") { throw "Unexpected publishing repository." }
# Serialized publisher: version-release catalogs stay subsets. Only channel
# staging merges. A failed download/signature never means an empty old catalog.
& (Join-Path $PSScriptRoot "verify-signatures.ps1") -CatalogDirectory $CatalogDirectory -PublicKeyPath $PublicKeyPath
$temporaryBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
$temporary = Join-Path $temporaryBase ("pd-channel-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $temporary | Out-Null
try {
    $plans = @(); $merged = @{}
    foreach ($channel in @("stable", "preview")) {
        $name = "catalog-$channel.json"
        $incoming = Get-Content -LiteralPath (Join-Path $CatalogDirectory $name) -Raw | ConvertFrom-Json
        Assert-DriverCatalog $incoming $channel
        $tag = "catalog-$channel"
        $releaseOutput = @(gh api "repos/$Repository/releases/tags/$tag" 2>&1)
        $exists = $LASTEXITCODE -eq 0
        if (-not $exists -and ($releaseOutput -join "`n") -notmatch '\bHTTP 404\b|"status"\s*:\s*"?404"?') { throw "Cannot determine whether the existing $channel directory exists; refusing to overwrite it." }
        if ($exists -and @($incoming.drivers).Count -eq 0) { Write-Output "Preserving existing $channel directory (no packages in this release)."; continue }
        $previous = $null
        if ($exists) {
            $oldDirectory = Join-Path $temporary "old-$channel"
            New-Item -ItemType Directory -Path $oldDirectory | Out-Null
            gh release download $tag --repo $Repository --dir $oldDirectory --pattern $name --pattern ($name + ".sig")
            if ($LASTEXITCODE -ne 0) { throw "Failed to download existing $channel directory and signature." }
            & (Join-Path $PSScriptRoot "verify-signatures.ps1") -CatalogDirectory $oldDirectory -CatalogName $name -PublicKeyPath $PublicKeyPath
            $previous = Get-Content -LiteralPath (Join-Path $oldDirectory $name) -Raw | ConvertFrom-Json
        }
        $merged[$channel] = Merge-DriverCatalog $previous $incoming $channel
        $plans += [pscustomobject]@{ Channel = $channel; Name = $name; Tag = $tag; Exists = $exists }
    }
    if ($plans.Count -eq 0) { return }
    # Signer handles all three names. Unplanned channels are never uploaded.
    foreach ($channel in @('stable', 'preview')) {
        $value = if ($merged.ContainsKey($channel)) { $merged[$channel] } else { [pscustomobject]@{schema=1;channel=$channel;drivers=@()} }
        [IO.File]::WriteAllText((Join-Path $temporary "catalog-$channel.json"), ($value | ConvertTo-Json -Depth 100) + "`n", [Text.UTF8Encoding]::new($false))
    }
    Copy-Item -LiteralPath (Join-Path $temporary 'catalog-stable.json') -Destination (Join-Path $temporary 'catalog.json')
    & (Join-Path $PSScriptRoot "sign-catalog.ps1") -CatalogDirectory $temporary
    & (Join-Path $PSScriptRoot "verify-signatures.ps1") -CatalogDirectory $temporary -PublicKeyPath $PublicKeyPath
    foreach ($plan in $plans) {
        $assets = @((Join-Path $temporary $plan.Name), (Join-Path $temporary ($plan.Name + '.sig')))
        if ($plan.Channel -eq 'stable') { $assets += @((Join-Path $temporary 'catalog.json'), (Join-Path $temporary 'catalog.json.sig')) }
        if ($plan.Exists) { gh release upload $plan.Tag --repo $Repository --clobber @assets }
        else { gh release create $plan.Tag --repo $Repository --prerelease --title "$($plan.Channel) signed driver catalog" --notes "Signed catalog for Peripheral Desk. Device ZIP downloads remain in their version releases." @assets }
        if ($LASTEXITCODE -ne 0) { throw "Failed to update $($plan.Channel) directory." }
        $published = Join-Path $temporary ("published-" + $plan.Channel)
        New-Item -ItemType Directory -Path $published | Out-Null
        gh release download $plan.Tag --repo $Repository --dir $published --pattern $plan.Name --pattern ($plan.Name + '.sig')
        if ($LASTEXITCODE -ne 0) { throw "Failed to download published $($plan.Channel) directory." }
        & (Join-Path $PSScriptRoot "verify-signatures.ps1") -CatalogDirectory $published -CatalogName $plan.Name -PublicKeyPath $PublicKeyPath
        if ((Get-FileHash -LiteralPath (Join-Path $published $plan.Name) -Algorithm SHA256).Hash -ne (Get-FileHash -LiteralPath (Join-Path $temporary $plan.Name) -Algorithm SHA256).Hash) { throw "Published merged channel differs from the signed bytes." }
        Write-Output "Published https://github.com/$Repository/releases/download/$($plan.Tag)/$($plan.Name)"
    }
} finally {
    $resolved = [IO.Path]::GetFullPath($temporary)
    if (-not $resolved.StartsWith($temporaryBase, [StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($resolved) -notmatch '^pd-channel-[0-9a-f]{32}$') { throw "Refusing unsafe temporary cleanup." }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
