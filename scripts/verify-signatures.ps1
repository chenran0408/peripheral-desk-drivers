[CmdletBinding()]
param(
    [string]$CatalogDirectory = (Join-Path $PSScriptRoot "../dist"),
    [string]$PublicKeyPath = (Join-Path $PSScriptRoot "../assets/catalog-public-key.blob"),
    [ValidateSet("catalog-stable.json", "catalog-preview.json", "catalog.json")]
    [string[]]$CatalogName = @("catalog-stable.json", "catalog-preview.json", "catalog.json")
)
$ErrorActionPreference = "Stop"
$blob = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $PublicKeyPath).Path)
if ($blob.Length -lt 24) { throw "Invalid RSA public blob." }
$magic = [BitConverter]::ToUInt32($blob, 0)
$bits = [BitConverter]::ToUInt32($blob, 4)
$exponentLength = [BitConverter]::ToUInt32($blob, 8)
$modulusLength = [BitConverter]::ToUInt32($blob, 12)
if ($magic -ne 0x31415352 -or $bits -lt 2048 -or $bits -ne $modulusLength * 8 -or $exponentLength -lt 1 -or $exponentLength -gt 8 -or $blob.Length -ne 24 + $exponentLength + $modulusLength -or [BitConverter]::ToUInt32($blob, 16) -ne 0 -or [BitConverter]::ToUInt32($blob, 20) -ne 0) { throw "Invalid BCRYPT_RSAPUBLIC_BLOB." }
$parameters = [Security.Cryptography.RSAParameters]::new()
$parameters.Exponent = [byte[]]$blob[24..(23 + $exponentLength)]
$parameters.Modulus = [byte[]]$blob[(24 + $exponentLength)..($blob.Length - 1)]
$rsa = [Security.Cryptography.RSA]::Create()
try {
    $rsa.ImportParameters($parameters)
    foreach ($name in $CatalogName) {
        $catalog = Join-Path $CatalogDirectory $name
        $bytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $catalog).Path)
        $signature = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath ($catalog + ".sig")).Path)
        if (-not $rsa.VerifyData($bytes, $signature, [Security.Cryptography.HashAlgorithmName]::SHA256, [Security.Cryptography.RSASignaturePadding]::Pkcs1)) { throw ("Signature verification failed: " + $name) }
        Write-Output ("Verified signature " + $name)
    }
} finally { $rsa.Dispose() }
