# Windows에서 Amu 사용하기

Windows 11 x64용 실험 입력기입니다. x64·x86 앱용 TSF DLL을 함께 포함합니다. ARM64 Windows, 터치 키보드, 원격 데스크톱과 앱별 호환성은 검증 대상에 포함하지 않습니다.

## Release 패키지 설치

1. GitHub Releases에서 `Amu-…-windows-x64.zip`을 받고 압축을 풉니다. 소스 코드 ZIP과 다릅니다.
2. 입력 중인 글을 저장하고 다른 입력기로 전환합니다.
3. **현재 사용자의 관리자 PowerShell**을 열고 압축을 푼 폴더로 이동합니다.
4. 아래 명령을 실행합니다.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\Install.ps1
```

이 명령에서만 실행 정책을 우회하며 시스템 실행 정책은 변경하지 않습니다. 코드 서명 인증서는 아직 사용하지 않습니다. Windows의 보안 경고가 나올 수 있으므로 이 저장소의 Release 출처와 `SHA256SUMS`를 확인하세요. 관리자가 차단한 실행 정책은 우회하지 마세요.

설치기는 `%ProgramFiles%\Amu\versions\<빌드 식별자>`에 복사하고 파일 해시·컴파일된 버전·32/64비트 COM 등록 경로를 확인합니다. 한국어 입력 목록에 Amu를 추가하며 기존 언어와 입력기는 유지합니다. 관리자 권한이 필요한 이유는 시스템 TSF 등록과 Program Files 쓰기 때문입니다. 다른 관리자 계정의 자격 증명으로 설치하면 입력 목록도 그 계정에 적용되므로 현재 사용자로 실행하세요.

설치 후 **로그아웃·로그인**하고 `Win+Space`에서 **Amu**를 선택하세요. 보이지 않으면 Windows 설정 → 시간 및 언어 → 언어 및 지역 → 한국어 → 언어 옵션 → 키보드에서 Amu를 추가하세요.

## 타이핑

기본은 자동 판독입니다. 한글·영어를 QWERTY 키 위치로 입력합니다.

```text
hello dkssud       → hello 안녕
tkatjdsdi          → 삼성sdi
airh               → ai고
fallback           → fallback
fd                 → ㄹㅇ
```

- `Ctrl+Shift+F1`: 자동 판독
- `Ctrl+Shift+F2`: 한글 두벌식 고정
- `Ctrl+Shift+F3`: 영문 고정
- `Win+Space`: 다른 입력기로 전환

Shift와 Caps Lock 대소문자를 반영합니다. Backspace는 코어의 한글 자모 삭제 규칙을 따릅니다. Space·Enter·Tab·방향키·단축키는 조합을 확정하고 앱으로 전달합니다. 붙여넣은 내용이나 이미 확정한 단어를 소급 변환하지 않습니다. 자동 판독 자체의 모호한 입력 제한은 macOS와 같습니다.

## 업데이트·확인·제거

새 Release의 설치 명령을 다시 실행합니다. 이전 버전은 남겨두며 사용 중인 DLL을 덮어쓰거나 앱·ctfmon을 강제 종료하지 않습니다. **이미 실행 중인 앱은 이전 DLL을 계속 사용할 수 있으므로 업데이트 후 로그아웃·로그인해야 합니다.** 아래 확인 명령은 등록된 설치본의 버전이며 모든 앱에 로드된 버전을 조회하는 기능은 아닙니다.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\Install.ps1 -Action Status
powershell -NoProfile -ExecutionPolicy Bypass -File .\Install.ps1 -Action Uninstall
```

제거는 다른 입력기로 전환하고 관리자 PowerShell에서 실행하세요. 등록과 입력 목록에서 Amu를 제거하며 파일은 보존합니다. 로그아웃·로그인 후 `%ProgramFiles%\Amu`를 삭제하면 됩니다. 등록 중 오류가 나면 이전 설치본의 등록을 복구하고 실패를 표시합니다.

## 소스에서 빌드

Git, mise, **Visual Studio Build Tools의 C++ 데스크톱 개발·MSVC x64/x86·Windows SDK**가 필요합니다. Rust와 Node 버전은 저장소의 `mise.toml`에서 설치합니다.

```powershell
git clone https://github.com/TylerShin/amu.git
cd amu
mise trust
mise install
mise run windows:package
```

MSVC 설치는 `vswhere`로 찾습니다. Rust 대상은 x64·x86을 설치하고 libhangul을 교체 가능한 DLL로 빌드합니다. 결과는 `dist/release-windows/`이며 압축을 풀어 위의 설치 명령을 사용합니다. `mise run windows:install`은 빌드·검사·패키징 후 설치까지 수행합니다(관리자 PowerShell 필요).

`mise run windows:check`는 Rust 회귀 검사와 x64·x86 네이티브 검사를 수행합니다. 네이티브 검사는 실제 Windows TSF context와 테스트용 문서 저장소에서 조합·삭제·확정·포커스 전환·편집 잠금 거부를 확인합니다. 이것만으로 메모장·브라우저·Electron 등 모든 앱의 호환성을 검증한 것은 아닙니다. 첫 사용 시 메모장에서 위 입력, 삭제, 방향키, 선택 후 입력, 앱 전환을 확인해주세요.

## 라이선스와 복구 자료

Amu는 MIT, libhangul은 LGPL-2.1-or-later, 한국어 음절 통계는 CC BY-SA 4.0입니다. ZIP에 라이선스·libhangul 소스·교체 안내·모델 재생성 자료를 포함합니다. Microsoft Visual C++ 런타임 DLL은 Microsoft 재배포 조건을 따릅니다. 패키지의 `package.json`에는 개별 파일 SHA-256이 있습니다.
