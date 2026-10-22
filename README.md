# Amu · 아무

**아무거나 쓰세요. 한글과 영어 사이의 전환은 Amu가 고민할게요.**

[![CI](https://github.com/TylerShin/amu/actions/workflows/ci.yml/badge.svg)](https://github.com/TylerShin/amu/actions/workflows/ci.yml)
[![라이선스: MIT](https://img.shields.io/badge/코드-MIT-blue)](LICENSE)
![상태: 실험 버전](https://img.shields.io/badge/상태-실험_버전-orange)

Amu는 **“아무거나 쓰세요”의 ‘아무’**에서 이름을 따온 한영 자동 판독 입력기입니다. 한영 전환 키를 누르지 않고 타이핑하면, 같은 키 입력을 한글·영어·혼합 후보로 해석해 표시합니다.

문서·이슈·PR은 한국어를 기본으로 운영합니다.

[직접 써보기](#직접-써보기) · [작동 원리](#작동-원리) · [기여하기](CONTRIBUTING.md) · [개발 계획](ROADMAP.md)

```text
누르는 키                    화면에 보이는 글
hello dkssud              →  hello 안녕
tkatjdsdi tkrhtlvdj        →  삼성sdi 사고싶어
tkatjdqzx                 →  삼성qzx
githubdp                  →  github에
ai gksrmf                 →  ai 한글
```

> [!NOTE]
> 현재는 **macOS에서 직접 빌드해 사용하는 실험 버전**입니다. 자동 판독이 틀릴 수 있으며, 서명·공증된 설치 파일은 아직 배포하지 않습니다. Firefox·Chrome 등 외부 앱의 시스템 입력기 호환성은 검증 중입니다.

## 직접 써보기

macOS, Git, Xcode Command Line Tools의 Swift·clang·SDK, [mise](https://mise.jdx.dev/getting-started.html)가 필요합니다. Rust와 Node.js 버전은 `mise.toml`에서 관리합니다.

```sh
git clone https://github.com/TylerShin/amu.git
cd amu
mise trust
mise install
mise run macos:play
```

`Amu Playground`에서 **자동 판독**을 선택하고 위 예시를 직접 타이핑해보세요. 이 창은 시스템 입력기를 변경하지 않고 실제 공통 코어를 사용합니다. 붙여넣은 텍스트는 소급 변환하지 않습니다. 한글 고정·영문 고정 모드도 선택할 수 있습니다.

이미 열린 테스트 창에는 새 빌드가 자동 적용되지 않습니다. 입력 내용을 복사해두고 앱을 종료한 뒤 다시 실행하세요.

### 다른 앱에서 사용하기

```sh
mise run macos:install
```

현재 사용자의 `~/Library/Input Methods/Amu.app`에 설치·등록합니다. 이후 macOS 입력 메뉴에서 **Amu**를 선택하세요. 메뉴에 없다면 **시스템 설정 → 키보드 → 텍스트 입력 → 편집 → 한국어 → Amu**를 추가합니다. 등록 캐시 때문에 보이지 않을 때는 로그아웃 후 다시 로그인합니다.

설치 명령은 현재 입력 소스를 자동으로 변경하지 않습니다. 이전 설치본은 `~/Library/Application Support/Amu/backups/`에 보관합니다. 기존 한국어·영어 입력 소스로 전환하면 Amu 사용을 멈출 수 있습니다.

제거할 때는 먼저 다른 입력 소스를 선택하고, 키보드 설정에서 Amu를 제거한 다음 설치된 `Amu.app`을 휴지통으로 옮기세요.

## 현재 지원 범위

| 기능 | 상태 |
|---|---|
| 두벌식 조합·받침 이동·Backspace | 구현·자동 검사 |
| 한글·영문·양방향 혼합 판독 | 구현; 한 단어 내부 경계 최대 1개 |
| 미등록 약어·새 영문 문자열 | 형태와 문맥으로 평가; 모든 입력을 구별하지는 못함 |
| macOS 테스트 창·InputMethodKit 입력기 | 로컬 빌드 가능 |
| 외부 앱별 입력·포커스 전환 호환성 | 검증 중 |
| Windows·Linux 입력기 연결 | 미구현 |

키 매핑은 ANSI QWERTY 위치와 Shift 기준입니다. Caps Lock 대문자 처리, 다른 배열·세벌식·옛한글·한자 변환, 확정된 단어의 재판독·자동 변환 취소는 아직 없습니다. 빌드의 최소 OS 대상은 macOS 13이며, 실제 로컬 실행은 Apple Silicon / macOS 26에서 확인했습니다. Intel Mac과 이전 OS의 사용 호환성을 보장하는 의미는 아닙니다.

## 작동 원리

**Rust 공통 코어 + libhangul 조합기 + Swift macOS 연결부**로 구성합니다.

```mermaid
flowchart LR
    Key[키 입력] --> Platform[macOS 연결부]
    Platform --> Session[입력 세션]
    Session --> Hangul[libhangul 조합]
    Hangul --> Decoder[한글·영어·혼합 후보 비교]
    Decoder --> Display[조합 표시·확정]
```

판독기는 한글 조합의 완성도와 음절 연결 통계, 영어의 문자 배열·대소문자·일반 단어 힌트, 최근 네 단어의 언어 경향을 함께 비교합니다. 특정 약어를 등록해야 인식하는 방식은 아닙니다. 문장 의미를 이해하는 모델도 아닙니다.

입력은 로컬에서 처리하며 타이핑 텍스트를 서버로 보내거나 키 입력 로그로 저장하지 않습니다. 입력 중 디스크·네트워크 요청은 하지 않습니다. 개발 도구 설치와 연구 자료 재생성에는 다운로드가 필요합니다.

| 경로 | 책임 |
|---|---|
| `crates/amu-core` | 조합, 후보 평가, 세션·문맥·삭제 정책 |
| `crates/amu-ffi` · `include/amu.h` | Rust ↔ Swift의 C ABI 계약 |
| `macos` | 키 이벤트, marked text, 메뉴, 테스트 창 |
| `models/korean` | 출처와 재생성 정보가 있는 한국어 음절 통계 |
| `third_party/libhangul` | 리비전을 고정한 원본 조합 소스 |
| `research` | 재현 가능한 조합·판독 평가 도구와 결과 |

## 검증과 한계

현대 한글 11,172음절의 조합·삭제, 편집 경계, FFI, 실제 NSTextView와의 연결을 검사합니다. 자동 판독은 고정 회귀 사례와 생성 약어로 별도 평가합니다. 초기 평가 샘플을 버그 수정에서도 재사용하므로 현재 결과는 독립적인 블라인드 평가가 아닙니다.

| 평가 | 기록된 결과 |
|---|---|
| 고정 판독 회귀 사례 | 130 / 130 |
| 짧은 영문·한글 × 5가지 문맥 | 130 / 130 |
| 한글 Shift 시작과 겹치지 않는 대문자 2글자 × 5가지 문맥 | 2,470 / 2,470 |
| 미등록 소문자 약어 + 한글 접두부 | 446 / 600 |
| 미등록 대문자 약어 + 한글 접두부 | 588 / 600 |
| UD GSD test의 한 음절 한글 토큰, 문맥 없음 | 865 / 878 보존 |
| UD GSD test의 두 음절 이상 한글 토큰, 문맥 없음 | 8,747 / 8,747 보존 |

위 수치는 [2026-10-22 평가 기록](research/decoder-evaluation.txt)입니다. 합성 약어와 개별 토큰의 결과이므로 실사용 전체 정확도로 일반화하지 않습니다. 소문자 약어가 온전한 한글로도 조합되는 경우는 여전히 어렵습니다. [평가 조건·성능·알려진 제약](docs/mvp.md)을 함께 읽어주세요.

## 개발과 기여

오변환 재현 사례, 앱별 조합 문제, 문서 개선부터 함께할 수 있습니다. 실제 입력 문장 대신 개인정보를 제거한 재현용 키열을 남겨주세요.

```sh
mise run check           # Rust 포맷·Clippy·코어/FFI 검사
mise run repo:check      # 저장소·모델 무결성·스크립트·문서 링크 검사
mise run workflow:check  # GitHub Actions 문법 검사
mise run macos:smoke     # macOS 빌드 + 실제 NSTextView 연결 검사
mise run core:evaluate  # 판독 평가와 코어 지연 측정
```

[기여 안내](CONTRIBUTING.md) · [행동 규칙](CODE_OF_CONDUCT.md) · [보안 제보](SECURITY.md) · [변경 기록](CHANGELOG.md)

## 더 읽기

- [현재 구현 흐름·검증·제약](docs/mvp.md)
- [전체 구조와 설계 방향](docs/architecture.md) — 향후 설계와 현재 구현을 구분합니다.
- [조합 신뢰성·복잡도 조사](docs/composition-reliability.md)
- [판독·조합 평가 재현 방법](research/README.md)
- [저장소 운영과 배포](docs/maintaining.md)

## 라이선스

Amu 코드·문서는 [MIT](LICENSE)입니다. libhangul은 **LGPL-2.1-or-later**, 한국어 음절 통계는 **CC BY-SA 4.0**을 따릅니다. [외부 코드·데이터 고지](THIRD_PARTY_NOTICES.md)에서 적용 범위와 출처를 확인할 수 있습니다.
