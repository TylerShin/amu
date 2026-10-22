[CmdletBinding()]
param(
    [ValidateSet('Install', 'Uninstall', 'Status')][string]$Action = 'Install',
    [switch]$SkipLanguageSetup
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Invoke-AmuInstaller {
    if (-not [Environment]::Is64BitOperatingSystem -or $env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
        throw '이 패키지는 Windows 11 x64용입니다. ARM64는 아직 지원하지 않습니다.'
    }
    $root = Join-Path $env:ProgramFiles 'Amu'
    $statePath = Join-Path $root 'active.json'
    if ($Action -eq 'Status') {
        if (-not (Test-Path $statePath)) { throw '설치된 Amu가 없습니다.' }
        $state = Get-Content -Raw $statePath | ConvertFrom-Json
        Assert-InstalledPath $state.directory $root
        Confirm-Version $state.directory $state.identifier
        Confirm-Registration $state.directory
        Write-Output ('등록된 버전: ' + $state.identifier)
        Write-Output '이미 실행 중인 앱은 이전 DLL을 유지할 수 있습니다. 업데이트 후 로그아웃·로그인하세요.'
        return
    }
    $principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw '관리자 PowerShell에서 다시 실행하세요. Windows TSF 입력기 등록에 관리자 권한이 필요합니다.'
    }
    New-Item -ItemType Directory -Force $root | Out-Null
    $lock = [IO.File]::Open((Join-Path $root 'install.lock'), 'OpenOrCreate', 'ReadWrite', 'None')
    try {
        $previous = $null
        if (Test-Path $statePath) {
            $previous = Get-Content -Raw $statePath | ConvertFrom-Json
            Assert-InstalledPath $previous.directory $root
        }
        $originalLanguages = $null
        if (-not $SkipLanguageSetup) { $originalLanguages = Get-WinUserLanguageList }
        if ($Action -eq 'Uninstall') {
            if ($null -eq $previous) { throw '설치된 Amu가 없습니다.' }
            try {
                Register-Package $previous.directory -Remove
                if (-not $SkipLanguageSetup) { Update-LanguageList -Remove }
            } catch {
                Register-Package $previous.directory
                if ($null -ne $originalLanguages) { Set-WinUserLanguageList $originalLanguages -Force }
                throw
            }
            Remove-Item $statePath
            Write-Output 'Amu 등록을 제거했습니다. 실행 중인 앱을 보호하기 위해 버전 파일은 보존했습니다.'
            Write-Output ('로그아웃·로그인 후 삭제 가능한 폴더: ' + $root)
            return
        }
        $manifest = Get-Content -Raw (Join-Path $PSScriptRoot 'package.json') | ConvertFrom-Json
        if ($manifest.identifier -notmatch '^v[0-9][A-Za-z0-9.\-]+$') { throw '잘못된 빌드 식별자입니다.' }
        Confirm-Package $PSScriptRoot $manifest
        $destination = Join-Path $root ('versions\' + $manifest.identifier)
        Assert-InstalledPath $destination $root
        if (-not (Test-Path $destination)) {
            $staging = Join-Path $root ('versions\.staging-' + [Guid]::NewGuid().ToString())
            New-Item -ItemType Directory -Force $staging | Out-Null
            try {
                foreach ($file in $manifest.files) {
                    $target = Join-Path $staging $file.path
                    New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
                    Copy-Item -LiteralPath (Join-Path $PSScriptRoot $file.path) -Destination $target
                }
                Copy-Item (Join-Path $PSScriptRoot 'package.json') $staging
                Confirm-Package $staging $manifest
                Confirm-Version $staging $manifest.identifier
                [IO.Directory]::Move($staging, $destination)
            } finally { if (Test-Path $staging) { Remove-Item -Recurse -Force $staging } }
        }
        Confirm-Package $destination $manifest
        Confirm-Version $destination $manifest.identifier
        try {
            Register-Package $destination
            Confirm-Registration $destination
            if (-not $SkipLanguageSetup) { Update-LanguageList }
            $state = @{ identifier = $manifest.identifier; directory = $destination }
            $temporary = Join-Path $root 'active.pending.json'
            $state | ConvertTo-Json | Set-Content -Encoding UTF8 $temporary
            if (Test-Path $statePath) { [IO.File]::Replace($temporary, $statePath, $null) }
            else { [IO.File]::Move($temporary, $statePath) }
        } catch {
            $failure = $_
            if ($null -ne $previous) { Register-Package $previous.directory }
            else { Register-Package $destination -Remove }
            if ($null -ne $originalLanguages) { Set-WinUserLanguageList $originalLanguages -Force }
            throw $failure
        }
        Write-Output ('설치·등록 확인 완료: ' + $manifest.identifier)
        Write-Output '입력 중인 앱은 종료하지 않았습니다. 로그아웃·로그인한 뒤 Win+Space에서 Amu를 선택하세요.'
    } finally { $lock.Dispose() }
}

function Assert-InstalledPath([string]$Directory, [string]$Root) {
    $prefix = [IO.Path]::GetFullPath((Join-Path $Root 'versions')) + '\'
    if (-not [IO.Path]::GetFullPath($Directory).StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Amu 설치 범위 밖의 경로입니다.'
    }
}
function Confirm-Package([string]$Directory, $Manifest) {
    foreach ($file in $Manifest.files) {
        if ($file.path -notmatch '^[A-Za-z0-9_.\-/]+$' -or $file.path -match '(^|/)\.\.(/|$)' -or [IO.Path]::IsPathRooted($file.path)) {
            throw '잘못된 패키지 파일 경로입니다.'
        }
        $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $Directory $file.path)).Hash
        if ($actual -ne $file.sha256) { throw ('패키지 무결성 검사 실패: ' + $file.path) }
    }
}
function Invoke-Native([string]$Executable, [string[]]$Arguments) {
    & $Executable @Arguments
    if ($LASTEXITCODE -ne 0) { throw ($Executable + ' 실패: ' + $LASTEXITCODE) }
}
function Confirm-Version([string]$Directory, [string]$Identifier) {
    foreach ($architecture in @('x64', 'x86')) {
        $executable = Join-Path $Directory ($architecture + '\amu-check.exe')
        $actual = Invoke-Native $executable @('--version') | ConvertFrom-Json
        if ($actual.identifier -ne $Identifier) { throw ('컴파일된 버전 불일치: ' + $architecture) }
    }
}
function Register-Package([string]$Directory, [switch]$Remove) {
    foreach ($architecture in @('x64', 'x86')) {
        $system = 'System32'
        if ($architecture -eq 'x86') { $system = 'SysWOW64' }
        $tool = Join-Path $env:WINDIR ($system + '\regsvr32.exe')
        $library = Join-Path $Directory ($architecture + '\Amu.dll')
        $arguments = @('/s', ('"' + $library + '"'))
        if ($Remove) { $arguments = @('/s', '/u', ('"' + $library + '"')) }
        $process = Start-Process -FilePath $tool -ArgumentList $arguments -Wait -PassThru
        if ($process.ExitCode -ne 0) { throw ('TSF 등록 변경 실패: ' + $architecture + ', ' + $process.ExitCode) }
    }
}
function Confirm-Registration([string]$Directory) {
    foreach ($architecture in @('x64', 'x86')) {
        $library = Join-Path $Directory ($architecture + '\Amu.dll')
        $view = [Microsoft.Win32.RegistryView]::Registry64
        if ($architecture -eq 'x86') { $view = [Microsoft.Win32.RegistryView]::Registry32 }
        $registry = [Microsoft.Win32.RegistryKey]::OpenBaseKey('LocalMachine', $view)
        try {
            $key = $registry.OpenSubKey('SOFTWARE\Classes\CLSID\{991ABF87-12A1-4F5C-A9CC-800724AD3AF3}\InprocServer32')
            if ($null -eq $key) { throw '등록된 COM 서버를 찾지 못했습니다.' }
            try { if ($key.GetValue('') -ne $library) { throw '등록된 DLL 경로가 다릅니다.' } }
            finally { $key.Dispose() }
        } finally { $registry.Dispose() }
        Invoke-Native (Join-Path $Directory ($architecture + '\amu-check.exe')) @('--registered')
    }
}

function Update-LanguageList([switch]$Remove) {
    $tip = '0412:{991ABF87-12A1-4F5C-A9CC-800724AD3AF3}{19CCE201-BAF0-4A9D-A171-F3384BBB4F69}'
    $languages = Get-WinUserLanguageList
    $korean = $languages | Where-Object LanguageTag -eq 'ko-KR' | Select-Object -First 1
    if ($Remove) {
        if ($null -ne $korean) { $korean.InputMethodTips.Remove($tip) | Out-Null }
    } else {
        if ($null -eq $korean) {
            $added = New-WinUserLanguageList 'ko-KR'
            $korean = $added[0]
            $languages.Add($korean)
        }
        if (-not $korean.InputMethodTips.Contains($tip)) { $korean.InputMethodTips.Add($tip) }
    }
    Set-WinUserLanguageList $languages -Force
}

Invoke-AmuInstaller
