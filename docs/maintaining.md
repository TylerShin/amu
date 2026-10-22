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
- `검증 (ubuntu-24.04)`와 `검증 (macos-26)` 성공을 필수 검사로 지정.
- 운영자의 긴급 수정 우회 경로는 명시적으로 관리하고 사용 이유를 PR에 기록.

단독 운영 단계에서 본인 PR에 다른 사람의 필수 승인을 요구하면 병합이 막힐 수 있으므로 승인 수는 협업 체계가 생긴 뒤 정한다. 보안 설정에서 Dependabot alerts·secret scanning·push protection의 실제 활성 상태를 확인한다. 취약점 제보 연락처는 [SECURITY.md](../SECURITY.md)다.

## 이슈와 라벨

기본 라벨 `bug`, `enhancement`, `documentation`, `question`, `good first issue`, `help wanted`를 사용한다. Dependabot 변경은 `dependencies`로 분류한다. 한국어 설명과 재현 조건을 기준으로 분류하고, 자동으로 오래된 이슈를 닫는 봇은 두지 않는다.

오변환 제보는 정상 한글·영문을 망가뜨리는 반례와 함께 확인한다. 하나의 제보를 해결하려고 특정 약어를 제품 코드에 등록하지 않는다. 개인정보가 포함된 제보는 필요한 재현 조건만 남기도록 요청한다.

## 의존성과 CI 유지보수

GitHub Actions는 전체 커밋 SHA에 고정하고 Dependabot이 갱신 PR을 연다. 작업 권한은 `contents: read`이며 외부 PR에 비밀정보를 전달하는 `pull_request_target` 흐름을 사용하지 않는다.

Rust·Node.js·actionlint·gitleaks는 `mise.toml`, mise 자체는 CI 설정에 고정한다. Cargo·Actions 이외의 버전은 유지보수자가 변경 PR로 올린다. libhangul·모델 변경에는 원본 리비전, 라이선스, SHA-256, 재현 결과를 함께 갱신한다. CI의 도구 설치가 사용하는 네트워크와 제품의 로컬 입력 경로는 별개다.

CI의 mise 캐시는 비활성화한다. 현재 Rust 백엔드의 실제 toolchain은 mise 설치 경로 밖의 `RUSTUP_HOME`에 있어, mise 캐시만 복원하면 rustfmt·clippy 설치가 누락될 수 있다. 캐시를 도입하려면 toolchain과 구성 요소까지 복원되는지 재실행에서 확인한다.

`secrets:check`는 Git 인덱스의 파일을 임시 디렉터리에 내보내 gitleaks로 검사하고 삭제한다. 결과에는 비밀값을 가려 출력한다. 작업 디렉터리의 미추적 파일이나 전체 과거 이력까지 검사하는 명령은 아니다. 이미 커밋된 과거 유출이 의심되면 별도로 `gitleaks git --redact`로 이력을 확인하고 노출된 인증정보를 폐기한다.

## 릴리스

현재는 소스 빌드만 제공하는 실험 버전이다. 자동으로 태그를 만들거나 공증되지 않은 앱을 안정 버전으로 배포하지 않는다.

1. `CHANGELOG.md`의 미출시 항목을 실제 버전과 날짜로 정리한다.
2. `Cargo.toml`·`Cargo.lock`과 macOS 빌드 스크립트의 번들 버전·ABOUT 문구를 함께 갱신한다.
3. 대상 커밋의 CI와 지원 앱별 수동 검사 결과를 확인한다. 지원 CPU·OS와 남은 제약을 명시한다.
4. 앱 안에 Amu MIT, libhangul 소스·라이선스·교체 안내, 모델 출처·라이선스·재생성 자료가 포함됐는지 검사한다.
5. 배포 방식에 맞게 서명·공증을 검증한 뒤 태그와 GitHub Release를 만든다. 현재 개발 번들은 ad-hoc 서명이다.
6. 한국어 릴리스 설명에 달라진 동작, 설치·업데이트·제거 방법, 검증 환경, SHA-256을 기록한다. `.github/release.yml`의 자동 분류 결과는 초안으로 사용한다.

## 운영 도구 문서

[GitHub의 커뮤니티 운영 문서](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions), [Actions 보안 지침](https://docs.github.com/en/actions/reference/security/secure-use), [공식 runner 목록](https://github.com/actions/runner-images), [mise-action](https://github.com/jdx/mise-action)을 기준으로 구성했다.
