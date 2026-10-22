$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$source = Get-Content -Raw -Encoding UTF8 (Join-Path $PSScriptRoot '..\Install.ps1')
. ([scriptblock]::Create($source)) -SkipLanguageSetup

function Assert([bool]$condition, [string]$message) { if (-not $condition) { throw $message } }
function Expect-Failure([scriptblock]$operation) {
    $failed = $false
    try { & $operation | Out-Null } catch { $failed = $true }
    Assert $failed 'Expected the installer to report failure'
}
function New-Package([string]$Identifier) {
    $directory = Join-Path $script:temporary $Identifier
    New-Item -ItemType Directory $directory | Out-Null
    Set-Content -Encoding ASCII (Join-Path $directory 'payload.txt') $Identifier
    $hash = (Get-FileHash (Join-Path $directory 'payload.txt') -Algorithm SHA256).Hash
    @{ identifier = $Identifier; files = @(@{ path = 'payload.txt'; sha256 = $hash }) } |
        ConvertTo-Json -Depth 5 | Set-Content -Encoding UTF8 (Join-Path $directory 'package.json')
    return $directory
}
# Replace only the OS registration/version boundary. File staging, checksums,
# locking, persistent state and rollback order run through the shipped installer.
function Confirm-Version([string]$Directory, [string]$Identifier) {}
function Confirm-Registration([string]$Directory) {}
function Register-Package([string]$Directory, [switch]$Remove) {
    $script:events.Add(@{ directory = $Directory; remove = [bool]$Remove })
    if ($script:failNextRegistration) {
        $script:failNextRegistration = $false
        throw 'Injected OS registration failure'
    }
}

$script:temporary = Join-Path ([IO.Path]::GetTempPath()) ('amu-install-tests-' + [Guid]::NewGuid().ToString())
$originalProgramFiles = $env:ProgramFiles
$script:events = [Collections.Generic.List[object]]::new()
$script:failNextRegistration = $false
try {
    New-Item -ItemType Directory $script:temporary | Out-Null
    $env:ProgramFiles = Join-Path $script:temporary 'programs'
    $first = New-Package 'v0.1.0-local.first'
    $second = New-Package 'v0.1.0-local.second'
    $Action = 'Install'
    Invoke-AmuInstaller $first | Out-Null
    $statePath = Join-Path $env:ProgramFiles 'Amu\active.json'
    $previous = Get-Content -Raw $statePath | ConvertFrom-Json
    Assert ($previous.identifier -eq 'v0.1.0-local.first') 'First install must be recorded'
    Assert (Test-Path (Join-Path $previous.directory 'payload.txt')) 'Installed payload must exist'

    $script:failNextRegistration = $true
    Expect-Failure { Invoke-AmuInstaller $second }
    $state = Get-Content -Raw $statePath | ConvertFrom-Json
    Assert ($state.identifier -eq $previous.identifier) 'Failed update must retain the previous active version'
    Assert ($script:events[$script:events.Count - 1].directory -eq $previous.directory) 'Previous COM registration must be restored'
    Assert (-not $script:events[$script:events.Count - 1].remove) 'Rollback must register the previous version'

    $before = $script:events.Count
    Set-Content (Join-Path $second 'payload.txt') 'tampered'
    Expect-Failure { Invoke-AmuInstaller $second }
    Assert ($script:events.Count -eq $before) 'Tampered packages must not change registration'

    $lock = [IO.File]::Open((Join-Path $env:ProgramFiles 'Amu\install.lock'), 'OpenOrCreate', 'ReadWrite', 'None')
    try { Expect-Failure { Invoke-AmuInstaller $first } } finally { $lock.Dispose() }
    Assert ($script:events.Count -eq $before) 'Concurrent installs must not change registration'

    $Action = 'Uninstall'
    $script:failNextRegistration = $true
    Expect-Failure { Invoke-AmuInstaller $first }
    Assert (Test-Path $statePath) 'Failed uninstall must retain recovery metadata'
    Assert ($script:events[$script:events.Count - 1].directory -eq $previous.directory) 'Failed uninstall must restore registration'

    Invoke-AmuInstaller $first | Out-Null
    Assert (-not (Test-Path $statePath)) 'Successful uninstall must remove active registration metadata'
    Assert (Test-Path (Join-Path $previous.directory 'payload.txt')) 'Uninstall must preserve DLLs for running hosts'
    Write-Output 'PASS: install, rollback, integrity rejection, concurrent install and uninstall recovery'
} finally {
    $env:ProgramFiles = $originalProgramFiles
    Remove-Item -Recurse -Force $script:temporary
}
