[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Version,
    [string]$ReleaseName,
    [string]$ReleaseNotes,
    [string[]]$AssetPaths,
    [string]$TargetCommitish = 'main'
)

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

if (-not $ReleaseName) {
    $ReleaseName = "Windows全局字体替换器 v$Version"
}

$tag = if ($Version.StartsWith('v')) { $Version } else { 'v' + $Version }

if (-not $ReleaseNotes) {
    $ReleaseNotes = "- 发布 $ReleaseName"
}

function Get-GitHubToken {
    $credentialQuery = @"
protocol=https
host=github.com
path=soberbw-hash/WindowsFontTuner.git
"@
    $creds = $credentialQuery | git credential fill
    $passwordLine = $creds | Select-String '^password=' | Select-Object -First 1
    if (-not $passwordLine) {
        throw '没有读取到 GitHub token。'
    }

    return $passwordLine.ToString().Substring(9)
}

function Invoke-GitHubJson {
    param(
        [Parameter(Mandatory = $true)][string]$Method,
        [Parameter(Mandatory = $true)][string]$Uri,
        [Parameter(Mandatory = $true)][hashtable]$Headers,
        [object]$Body
    )

    if ($null -eq $Body) {
        return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $Headers
    }

    $json = $Body | ConvertTo-Json -Depth 8
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($json)
    return Invoke-RestMethod -Method $Method -Uri $Uri -Headers $Headers -Body $bytes -ContentType 'application/json; charset=utf-8'
}

function Remove-ExistingAsset {
    param(
        [Parameter(Mandatory = $true)][object]$Release,
        [Parameter(Mandatory = $true)][string]$AssetName,
        [Parameter(Mandatory = $true)][hashtable]$Headers
    )

    $assets = @($Release.assets)
    foreach ($asset in $assets) {
        if ($asset -and $asset.name -eq $AssetName) {
            Invoke-RestMethod -Method Delete -Uri ("https://api.github.com/repos/soberbw-hash/WindowsFontTuner/releases/assets/" + $asset.id) -Headers $Headers | Out-Null
        }
    }
}

$token = Get-GitHubToken
$headers = @{
    Authorization = "Bearer $token"
    Accept = 'application/vnd.github+json'
    'X-GitHub-Api-Version' = '2022-11-28'
    'User-Agent' = 'WindowsFontTuner-ReleasePublisher'
}

$body = @{
    tag_name = $tag
    target_commitish = $TargetCommitish
    name = $ReleaseName
    body = $ReleaseNotes
    draft = $false
    prerelease = $false
}

try {
    $release = Invoke-GitHubJson -Method Post -Uri 'https://api.github.com/repos/soberbw-hash/WindowsFontTuner/releases' -Headers $headers -Body $body
}
catch {
    $response = $_.Exception.Response
    if ($response -and $response.StatusCode.Value__ -eq 422) {
        $release = Invoke-RestMethod -Method Get -Uri ("https://api.github.com/repos/soberbw-hash/WindowsFontTuner/releases/tags/" + $tag) -Headers $headers
        $release = Invoke-GitHubJson -Method Patch -Uri ("https://api.github.com/repos/soberbw-hash/WindowsFontTuner/releases/" + $release.id) -Headers $headers -Body @{
            name = $ReleaseName
            body = $ReleaseNotes
            draft = $false
            prerelease = $false
        }
    }
    else {
        throw
    }
}

foreach ($assetPath in ($AssetPaths | Where-Object { $_ })) {
    $resolvedPath = Resolve-Path $assetPath
    $assetName = Split-Path -Leaf $resolvedPath
    Remove-ExistingAsset -Release $release -AssetName $assetName -Headers $headers

    $uploadUrl = "https://uploads.github.com/repos/soberbw-hash/WindowsFontTuner/releases/$($release.id)/assets?name=$([uri]::EscapeDataString($assetName))"
    & curl.exe --http1.1 -L -X POST `
        -H "Authorization: Bearer $token" `
        -H "Accept: application/vnd.github+json" `
        -H "Content-Type: application/octet-stream" `
        --data-binary "@$resolvedPath" `
        $uploadUrl | Out-Null
}

[PSCustomObject]@{
    Tag = $tag
    ReleaseUrl = $release.html_url
    Assets = @($AssetPaths | Where-Object { $_ } | ForEach-Object { Split-Path -Leaf $_ })
} | Format-List






