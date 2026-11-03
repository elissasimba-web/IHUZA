$ErrorActionPreference = 'Stop'

$phoneNumber = (Read-Host 'Nimero ya telefoni').Trim()
$firstName = (Read-Host 'Izina rya mbere').Trim()
$lastName = (Read-Host 'Izina rya nyuma (ushobora gusiga ubusa)').Trim()
$email = (Read-Host 'Email (ushobora gusiga ubusa)').Trim()

if ([string]::IsNullOrWhiteSpace($phoneNumber) -or [string]::IsNullOrWhiteSpace($firstName)) {
    throw 'Nimero ya telefoni n izina rya mbere birakenewe.'
}

$securePassword = Read-Host "Ijambo ry'ibanga (inyuguti 12 cyangwa zirenga)" -AsSecureString
$passwordPointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($securePassword)

try {
    $password = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($passwordPointer)
    if ($password.Length -lt 12) {
        throw "Ijambo ry'ibanga rigomba kugira inyuguti nibura 12."
    }

    $request = @{
        phone_number = $phoneNumber
        first_name = $firstName
        password = $password
    }
    if ($lastName) { $request.last_name = $lastName }
    if ($email) { $request.email = $email }

    $body = $request | ConvertTo-Json -Compress
    Invoke-RestMethod -Uri 'http://127.0.0.1:3000/auth/register' -Method Post `
        -ContentType 'application/json; charset=utf-8' -Body $body -TimeoutSec 10 | Out-Null
    Write-Host 'Konti yakozwe. Ushobora gukomeza kuri bootstrap_admin.'
}
finally {
    [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($passwordPointer)
    Remove-Variable password, body, request -ErrorAction SilentlyContinue
}
