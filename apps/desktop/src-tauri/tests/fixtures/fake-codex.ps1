#Requires -Version 5.1
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('success', 'unicode', 'ansi', 'mixed', 'invalid-utf8', 'unauthenticated', 'rate-limit', 'large', 'slow', 'child', 'nonzero')]
    [string]$Scenario,

    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Remaining
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$OutputEncoding = [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)

if ($Remaining -contains '--version') {
    Write-Output 'codex-cli 99.0.0-fake'
    exit 0
}

if (($Remaining -contains '--help') -and -not ($Remaining -contains 'exec')) {
    Write-Output '--ask-for-approval never'
    exit 0
}

if (($Remaining -contains 'exec') -and ($Remaining -contains '--help')) {
    Write-Output '--ephemeral --sandbox --skip-git-repo-check --ignore-user-config --ignore-rules'
    exit 0
}

if (($Remaining -contains 'login') -and ($Remaining -contains 'status')) {
    if ($Scenario -eq 'unauthenticated') {
        [Console]::Error.WriteLine('Not logged in')
        exit 1
    }
    Write-Output 'Logged in with fake test identity'
    exit 0
}

$null = [Console]::In.ReadToEnd()
switch ($Scenario) {
    'success' { Write-Output 'fake response' }
    'unicode' {
        $text = ([char]0x00A1) + 'Respuesta r' + ([char]0x00E1) + 'pida! ' + [char]::ConvertFromUtf32(0x1F436)
        [Console]::Out.WriteLine($text)
    }
    'ansi' { [Console]::Out.Write("$([char]27)[32mgreen answer$([char]27)[0m") }
    'mixed' { [Console]::Error.WriteLine('diagnostic warning'); Write-Output 'safe stdout answer' }
    'invalid-utf8' {
        $bytes = [byte[]](0x66, 0x6f, 0x80)
        [Console]::OpenStandardOutput().Write($bytes, 0, $bytes.Length)
    }
    'unauthenticated' { [Console]::Error.WriteLine('Not logged in'); exit 1 }
    'rate-limit' { [Console]::Error.WriteLine('rate limit 429'); exit 1 }
    'large' { [Console]::Out.Write(('x' * 70000)) }
    'slow' { Start-Sleep -Seconds 15; Write-Output 'late response' }
    'child' {
        $child = Start-Process -FilePath 'powershell.exe' -ArgumentList @('-NoProfile', '-Command', 'Start-Sleep -Seconds 30') -PassThru -WindowStyle Hidden
        try { Wait-Process -Id $child.Id } finally { if (-not $child.HasExited) { Stop-Process -Id $child.Id -Force } }
    }
    'nonzero' { [Console]::Error.WriteLine('process failed'); exit 7 }
}
