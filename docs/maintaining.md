# 저장소 운영과 배포

운영 계정은 [TylerShin](https://github.com/TylerShin), 저장소 이름은 `amu`, 기본 브랜치는 `main`이다. 문서·이슈·PR·릴리스 설명을 한국어로 관리한다. 기능의 완료 여부는 [개발 계획](../ROADMAP.md), 사용자에게 보이는 변화는 [변경 기록](../CHANGELOG.md)에 남긴다.

## 첫 공개

아래는 공개 준비가 끝난 로컬 저장소에서 **실제로 GitHub에 생성·push할 때** 실행하는 명령이다. 저장소 생성이나 공개 설정은 문서를 추가하는 것만으로 적용되지 않는다. `origin`이 이미 있거나 같은 이름의 저장소가 존재하면 먼저 대상을 확인하고, 다른 저장소를 덮어쓰지 않는다.

```sh
git status --short
git log -1 --format='%h %an <%ae> %s'
mise run repo:check
mise run secrets:check
mise run workflow:check
mise run check
mise run macos:smoke
```

공개할 커밋을 확인한 다음 개인 계정의 인증을 현재 서브셸에만 적용한다. 토큰을 파일이나 Git 설정에 저장하지 않는다. GitHub HTTPS 인증에는 `gh auth git-credential` helper가 필요하다.

```sh
(
  export GH_TOKEN="$(gh auth token --user TylerShin)"
  test "$(gh api user --jq .login)" = TylerShin || exit 1
  gh repo create TylerShin/amu \
    --public --source=. --remote=origin --push \
    --description '아무거나 쓰세요. 한글과 영어를 자연스럽게 이어 쓰는 로컬 입력기'
)
```

생성과 첫 push를 확인한 뒤 기본 설정을 적용한다.

```sh
(
  export GH_TOKEN="$(gh auth token --user TylerShin)"
  test "$(gh api user --jq .login)" = TylerShin || exit 1
  gh repo edit TylerShin/amu \
    --enable-issues --enable-wiki=false \
    --enable-squash-merge --enable-merge-commit=false --enable-rebase-merge=false \
    --delete-branch-on-merge \
    --add-topic korean,hangul,input-method,macos,rust,swift,ime
)
```

README의 CI 배지는 첫 실행 결과를 반영한다. GitHub Actions에서 **해당 커밋**의 두 OS 검사가 통과하는지 확인한다. 로컬 검사를 통과했다는 사실만으로 원격 CI 성공을 기록하지 않는다.

## 브랜치와 리뷰

기능·수정은 작업 브랜치와 PR을 사용하고 squash merge를 기본으로 한다. PR 제목은 한국어 변경 의도가 드러나게 작성한다. `.github/CODEOWNERS`는 기본 담당자를 지정할 뿐 필수 승인이나 브랜치 보호를 켜지 않는다.

첫 CI 실행 후 GitHub의 `main` Ruleset에 다음을 설정한다.

- 삭제와 force push 제한.
- PR을 통한 변경, 미해결 리뷰 대화의 해결.
- `검증 (ubuntu-24.04)`, `검증 (macos-26)`, `검증 (windows-2025)` 성공을 필수 검사로 지정.
- 운영자의 긴급 수정 우회 경로는 명시적으로 관리하고 사용 이유를 PR에 기록.

단독 운영 단계에서 본인 PR에 다른 사람의 필수 승인을 요구하면 병합이 막힐 수 있으므로 승인 수는 협업 체계가 생긴 뒤 정한다. 보안 설정에서 Dependabot alerts·secret scanning·push protection의 실제 활성 상태를 확인한다. 취약점 제보 연락처는 [SECURITY.md](../SECURITY.md)다.

## 이슈와 라벨

기본 라벨 `bug`, `enhancement`, `documentation`, `question`, `good first issue`, `help wanted`를 사용한다. Dependabot 변경은 `dependencies`로 분류한다. 한국어 설명과 재현 조건을 기준으로 분류하고, 자동으로 오래된 이슈를 닫는 봇은 두지 않는다.

오변환 제보는 정상 한글·영문을 망가뜨리는 반례와 함께 확인한다. 하나의 제보를 해결하려고 특정 약어를 제품 코드에 등록하지 않는다. 개인정보가 포함된 제보는 필요한 재현 조건만 남기도록 요청한다.

## 의존성과 CI 유지보수

GitHub Actions는 전체 커밋 SHA에 고정하고 Dependabot이 갱신 PR을 연다. 검증 작업 권한은 `contents: read`이며 Release 공개 작업에만 `contents: write`를 부여한다. 외부 PR에 비밀정보를 전달하는 `pull_request_target` 흐름을 사용하지 않는다.

Rust·Node.js·actionlint·gitleaks는 `mise.toml`, mise 자체는 CI 설정에 고정한다. Cargo·Actions 이외의 버전은 유지보수자가 변경 PR로 올린다. libhangul·모델 변경에는 원본 리비전, 라이선스, SHA-256, 재현 결과를 함께 갱신한다. CI의 도구 설치가 사용하는 네트워크와 제품의 로컬 입력 경로는 별개다.

CI의 mise 캐시는 비활성화한다. 현재 Rust 백엔드의 실제 toolchain은 mise 설치 경로 밖의 `RUSTUP_HOME`에 있어, mise 캐시만 복원하면 rustfmt·clippy 설치가 누락될 수 있다. 캐시를 도입하려면 toolchain과 구성 요소까지 복원되는지 재실행에서 확인한다.

`secrets:check`는 Git 인덱스의 파일을 임시 디렉터리에 내보내 gitleaks로 검사하고 삭제한다. 결과에는 비밀값을 가려 출력한다. 작업 디렉터리의 미추적 파일이나 전체 과거 이력까지 검사하는 명령은 아니다. 이미 커밋된 과거 유출이 의심되면 별도로 `gitleaks git --redact`로 이력을 확인하고 노출된 인증정보를 폐기한다.

## 릴리스

main에 push하면 같은 CI 실행에서 Linux·macOS·Windows 검사 → macOS·Windows ZIP 패키징 → GitHub 개발 릴리스 공개를 수행한다. `workflow_dispatch`로 main에서 재시도할 수도 있다. PR·fork·다른 브랜치는 공개하지 않는다.

- `verify` 행렬의 모든 검사가 성공해야 `release` 작업이 시작된다. 패키지는 각 OS 검사에서 만든 바이너리를 그대로 사용하며 공개 작업에서 다시 빌드하지 않는다.
- 태그는 `v0.1.0-dev.<Actions 실행 번호>.<커밋 앞 12자리>` 형식이다. 태그 대상은 이동하는 main 대신 검사한 전체 커밋 SHA로 고정한다.
- Windows는 x64·x86 DLL을 함께 묶으며 CI에서 실제 TSF context 검사와 설치·등록·제거를 확인한다. 설치는 버전별 디렉터리에 보관하며 실행 중인 호스트 앱을 종료하지 않는다. [Windows 안내](install-windows.md)를 따른다.
- 현재 공개 macOS CPU는 `macos-26` runner의 Apple Silicon이다. 압축 전후 서명, 실행 파일의 CPU, 버전, 필수 라이선스 자료와 SHA-256을 검사한다.
- Release에는 macOS·Windows ZIP, `SHA256SUMS`, `release.json`이 올라간다. 모든 파일을 draft에 올린 후 prerelease로 공개한다. 업로드 실패 시 draft로 남으며 같은 실행을 재시도하면 이어서 공개한다. 이미 공개된 같은 태그는 덮어쓰지 않는다.
- main의 연속 push는 서로 취소하지 않는다. PR은 새 커밋이 이전 검증을 취소한다. main 실행을 수동 취소하면 해당 실행의 공개는 완료되지 않을 수 있다.
- 인증서·공증·별도 관리 앱·백그라운드 업데이트는 없다. 일반 다운로드 앱에 적용되는 Gatekeeper 제한과 수동 설치 절차는 [설치 안내](install-release.md)에 명시한다.

앱 버전의 기준은 Cargo workspace 버전이다. 빌드 스크립트가 번들 정보·ABOUT·`build.json`·실행 파일 안의 버전 응답에 같은 값을 넣는다. 로컬 빌드는 커밋·dirty 상태·고유 빌드 ID를 포함한다. 릴리스 기본 버전을 올릴 때는 `Cargo.toml`·`Cargo.lock`과 변경 기록을 갱신한다.

### 로컬 적용

```sh
mise run macos:install  # 빌드부터 실제 실행 버전 확인까지
mise run macos:version  # 실행 중인 프로세스에 다시 확인
mise run macos:package  # 로컬 ZIP을 dist/release에 생성
```

설치 전에 Amu에서 한국어 또는 ABC로 전환한다. 설치기는 staging 복사를 끝낸 뒤 지정 경로의 Amu만 정상 종료를 요청한다. 앱도 선택 상태·남은 조합을 검사해 종료를 거부할 수 있다. SIGKILL이나 다른 입력 앱 종료는 사용하지 않는다.

동시 설치는 잠금으로 막는다. 설치·등록·시작·버전 확인이 실패하면 새 프로세스를 안전하게 종료할 수 있을 때 이전 파일로 복구하고 실패를 보고한다. 복구 중 입력기가 다시 사용되어 종료할 수 없다면 현재 앱과 백업을 모두 보존한다. 백업 경로는 오류에 표시한다. 정상 복구 후에도 적용 성공으로 보고하지 않으며 다른 입력기를 유지한 상태에서 명령을 다시 실행한다.

비정상적인 전원 종료 등으로 `~/Library/Input Methods/.amu-install.lock`이 남으면 다른 설치가 실행 중이지 않은지 확인한 뒤 잠금 디렉터리를 제거한다. 자동으로 다른 설치의 잠금을 지우지 않는다.

원격 push만으로 로컬 Mac에서 코드를 실행하지 않는다. 로컬 적용은 사용자가 위 명령을 실행할 때 수행한다. 실패 로그를 확인할 때는 CI의 head SHA가 공개하려는 커밋과 같은지 먼저 대조한다.

## 운영 도구 문서

[GitHub의 커뮤니티 운영 문서](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions), [Actions 보안 지침](https://docs.github.com/en/actions/reference/security/secure-use), [공식 runner 목록](https://github.com/actions/runner-images), [mise-action](https://github.com/jdx/mise-action)을 기준으로 구성했다.
