[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Repository,
    [string]$CatalogDirectory = (Join-Path $PSScriptRoot "../dist")
)
$ErrorActionPreference = "Stop"
if ($Repository -ne "chenran0408/peripheral-desk-drivers") { throw "Unexpected publishing repository." }
# Call this only from the serialized signed-release job or an equivalent single
# maintainer session. Invalid intermediate pairs fail closed in the host, which
# keeps its previously verified catalog until both new assets are available.
& (Join-Path $PSScriptRoot "verify-signatures.ps1") -CatalogDirectory $CatalogDirectory
foreach ($channel in @("stable", "preview")) {
    $name = "catalog-$channel.json"
    $catalog = Join-Path $CatalogDirectory $name
    $content = Get-Content -LiteralPath $catalog -Raw | ConvertFrom-Json
    if ($content.schema -ne 1 -or $content.channel -ne $channel) { throw "Channel/schema mismatch." }
    $tag = "catalog-$channel"
    gh release view $tag --repo $Repository *> $null
    $exists = $LASTEXITCODE -eq 0
    # A stable-only release must not erase an existing preview directory, or the
    # reverse. Create the empty preview directory only on the initial release.
    if ($content.drivers.Count -eq 0 -and $exists) {
        Write-Output "Preserving existing $channel directory (no packages in this release)."
        continue
    }
    $assets = @((Resolve-Path -LiteralPath $catalog).Path, (Resolve-Path -LiteralPath ($catalog + ".sig")).Path)
    if ($channel -eq "stable") {
        $assets += @((Resolve-Path -LiteralPath (Join-Path $CatalogDirectory "catalog.json")).Path,
            (Resolve-Path -LiteralPath (Join-Path $CatalogDirectory "catalog.json.sig")).Path)
    }
    if ($exists) {
        gh release upload $tag --repo $Repository --clobber @assets
    } else {
        gh release create $tag --repo $Repository --prerelease --title "$channel signed driver catalog" --notes "Signed catalog for Peripheral Desk. Device ZIP downloads remain in their version releases." @assets
    }
    if ($LASTEXITCODE -ne 0) { throw "Failed to update $channel directory." }
    $temporary = Join-Path ([IO.Path]::GetTempPath()) ([Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $temporary | Out-Null
    try {
        gh release download $tag --repo $Repository --dir $temporary --pattern $name --pattern ($name + ".sig")
        if ($LASTEXITCODE -ne 0) { throw "Failed to download published $channel directory." }
        & (Join-Path $PSScriptRoot "verify-signatures.ps1") -CatalogDirectory $temporary -CatalogName $name
        if ((Get-FileHash -LiteralPath (Join-Path $temporary $name) -Algorithm SHA256).Hash -ne (Get-FileHash -LiteralPath $catalog -Algorithm SHA256).Hash) { throw "Published $channel catalog differs from this release." }
    } finally {
        Get-ChildItem -LiteralPath $temporary -File | ForEach-Object { Remove-Item -LiteralPath $_.FullName -Force }
        Remove-Item -LiteralPath $temporary -Force
    }
    Write-Output "Published https://github.com/$Repository/releases/download/$tag/$name"
}
