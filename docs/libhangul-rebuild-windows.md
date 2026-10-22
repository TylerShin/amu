# Windows libhangul DLL 교체

Amu는 Rust 코어에 libhangul을 정적으로 합치지 않고 옆 폴더의 `hangul.dll`을 불러옵니다. 패키지의 `libhangul-source/`에 수정하지 않은 원본과 LGPL-2.1-or-later 라이선스가 있습니다.

다른 입력기로 전환하고 Amu를 사용하는 앱을 모두 닫으세요. 버전 설치 폴더의 `x64` 또는 `x86`에 있는 DLL을 같은 아키텍처로 교체할 수 있습니다. 관리자 권한이 필요하며 불확실하면 로그아웃·로그인 후 교체합니다. DLL 서명 검증이나 독점 체크섬으로 대체 라이브러리의 로딩을 제한하지 않습니다. 설치기의 무결성 검사는 배포 패키지를 설치할 때 수행합니다.

Visual Studio의 대상 아키텍처 Native Tools Command Prompt에서 별도 작업 폴더를 만들고 다음 `hangul.def`를 저장합니다.

```text
LIBRARY hangul
EXPORTS
hangul_ic_new
hangul_ic_delete
hangul_ic_process
hangul_ic_backspace
hangul_ic_reset
hangul_ic_get_commit_string
hangul_ic_get_preedit_string
```

`SOURCE`를 패키지의 `libhangul-source` 경로로 바꿔 실행합니다.

```bat
cl /nologo /std:c11 /O2 /LD /MD /DENABLE_EXTERNAL_KEYBOARDS=0 /D_CRT_SECURE_NO_WARNINGS SOURCE\hangulctype.c SOURCE\hangulinputcontext.c SOURCE\hangulkeyboard.c /Fe:hangul.dll /link /DEF:hangul.def /IMPLIB:hangul.lib
```

생성된 DLL을 설치 폴더의 해당 아키텍처 디렉터리로 복사하고 앱을 다시 여세요. 함께 배포한 `amu-check.exe`를 실행해 조합·TSF 검사를 확인할 수 있습니다. 외부 키보드 XML 기능 등을 추가한다면 그에 필요한 의존성도 준비해야 합니다.
