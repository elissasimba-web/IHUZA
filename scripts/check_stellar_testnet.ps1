# Read-only check of the official Stellar testnet RPC network identity.
# This script never handles wallet keys, transactions, or customer funds.
$ErrorActionPreference = 'Stop'

$rpcUrl = 'https://soroban-testnet.stellar.org'
$expectedPassphrase = 'Test SDF Network ; September 2015'
$requestBody = '{"jsonrpc":"2.0","id":1,"method":"getNetwork"}'

$response = Invoke-RestMethod -Method Post -Uri $rpcUrl -ContentType 'application/json' -Body $requestBody -TimeoutSec 15

if ($response.jsonrpc -ne '2.0' -or $response.id -ne 1 -or $response.error) {
    throw 'Stellar RPC returned an invalid getNetwork response.'
}
if ($response.result.passphrase -ne $expectedPassphrase) {
    throw 'Stellar RPC is not connected to the expected testnet.'
}
if ($response.result.protocolVersion -isnot [int] -or $response.result.protocolVersion -le 0) {
    throw 'Stellar RPC did not report a valid protocol version.'
}

[pscustomobject]@{
    RpcUrl = $rpcUrl
    Network = 'testnet'
    ProtocolVersion = $response.result.protocolVersion
}
