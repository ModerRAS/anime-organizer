# qBittorrent AutoRun template:
# powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\Software\bin\qb_aniorg.ps1 "%L" "%F" "%I"

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Category,
    [Parameter(Mandatory = $true, Position = 1)]
    [string]$ContentPath,
    [Parameter(Mandatory = $true, Position = 2)]
    [string]$TorrentHash
)

$ErrorActionPreference = 'Stop'
$apiUrl = 'http://127.0.0.1:32145/api/v1/jobs'
$targetDir = 'S:\' + [char]0x52A8 + [char]0x6F2B
$logDir = Join-Path $env:LOCALAPPDATA 'anime-organizer'
$logPath = Join-Path $logDir 'qb_aniorg.log'
New-Item -ItemType Directory -Force -Path $logDir | Out-Null

function Write-HookLog([string]$Message) {
    $timestamp = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
    Add-Content -LiteralPath $logPath -Value "[$timestamp] $Message" -Encoding utf8
}

function Exit-Hook([int]$Code, [string]$Message) {
    Write-HookLog "$Message; exit_code=$Code"
    exit $Code
}

if ($Category -ne 'Ani') {
    Exit-Hook 0 "ignored category: $Category"
}

if ([string]::IsNullOrWhiteSpace($TorrentHash)) {
    Exit-Hook 2 'missing torrent hash'
}

if (-not (Test-Path -LiteralPath $ContentPath)) {
    Exit-Hook 2 "content path does not exist: $ContentPath"
}

$idempotencyKey = "qbittorrent:$($TorrentHash.ToLowerInvariant())"
$payload = @{
    idempotency_key = $idempotencyKey
    origin = 'qbittorrent'
    confirmed = $false
    job = @{
        type = 'organize'
        args = @{
            source = $ContentPath
            target = $targetDir
            mode = 'copy'
            mlip = $true
            verbose = $true
        }
    }
} | ConvertTo-Json -Depth 8 -Compress

for ($attempt = 1; $attempt -le 3; $attempt++) {
    try {
        $response = Invoke-WebRequest -Uri $apiUrl -Method Post -Body $payload -ContentType 'application/json; charset=utf-8' -UseBasicParsing
        if ([int]$response.StatusCode -ne 202) {
            Exit-Hook 1 "HTTP failure: status=$($response.StatusCode) attempt=$attempt"
        }

        try {
            $accepted = $response.Content | ConvertFrom-Json
            Exit-Hook 0 "accepted: job_id=$($accepted.job.id) duplicate=$($accepted.duplicate) status=$($response.StatusCode)"
        } catch {
            Exit-Hook 0 "accepted: job_id=unknown duplicate=unknown status=$($response.StatusCode); response parse failure=$($_.Exception.Message)"
        }
    } catch {
        $httpResponse = $_.Exception.Response
        if ($null -ne $httpResponse) {
            $status = [int]$httpResponse.StatusCode
            Exit-Hook 1 "HTTP failure: status=$status message=$($_.Exception.Message)"
        }

        Write-HookLog "connection failure: attempt=$attempt message=$($_.Exception.Message)"
        if ($attempt -lt 3) {
            Start-Sleep -Seconds 2
        }
    }
}

Exit-Hook 1 'request failed after 3 connection attempts'
