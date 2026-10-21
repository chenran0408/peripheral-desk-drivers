[CmdletBinding()]
param(
    [string]$PrivateKeyPath,
    [string]$CatalogDirectory = (Join-Path $PSScriptRoot "../dist")
)
$ErrorActionPreference = "Stop"
$keyXml = $env:DRIVER_CATALOG_SIGNING_KEY
if ($PrivateKeyPath) {
    if ($keyXml) { throw "Specify either DRIVER_CATALOG_SIGNING_KEY or -PrivateKeyPath, not both." }
    $keyXml = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $PrivateKeyPath).Path)
}
if ([string]::IsNullOrWhiteSpace($keyXml)) { throw "Catalog signing key is missing." }
$rsa = [Security.Cryptography.RSA]::Create()
try {
    $rsa.FromXmlString($keyXml)
    if ($rsa.KeySize -lt 2048) { throw "RSA signing key must be at least 2048 bits." }
    foreach ($name in @("catalog-stable.json", "catalog-preview.json", "catalog.json")) {
        $catalog = Join-Path $CatalogDirectory $name
        $bytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $catalog).Path)
        $signature = $rsa.SignData($bytes, [Security.Cryptography.HashAlgorithmName]::SHA256, [Security.Cryptography.RSASignaturePadding]::Pkcs1)
        [IO.File]::WriteAllBytes(($catalog + ".sig"), $signature)
        Write-Output ("Signed " + $name)
    }
} finally {
    $rsa.Dispose()
    $keyXml = $null
}
