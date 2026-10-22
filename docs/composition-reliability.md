# 한글 조합의 신뢰성·복잡도 조사

조사일: 2026-10-22. 조사·설계와 기준 라이브러리 실행까지 완료했다. Amu 입력기나 실제 앱 통합을 구현·검증한 결과는 아니다.

## 결정

**Rust 공통 세션 + 검증된 조합 백엔드 + 운영체제별 입력기 연결**을 우선한다. 첫 실험에서는 libhangul을 사용한다. 조합기를 Rust로 새로 만드는 일보다 조합 수명과 호스트 동기화를 검증하는 일이 먼저다. 자동 한영 판단은 이 위의 별도 책임이다.

한글의 정해진 결합 규칙에는 유한 상태 기계와 작은 테이블이 적합하다. 더 복잡한 모델로 바꿔 해결할 문제가 아니다. 개선해야 할 부분은 검증된 규칙 재사용, 완료 경계 보존, 입력 순서·선택 영역 관리, 제한된 후보 탐색이다. Unicode NFC는 자모 인코딩 정규화이며 키보드 배열·Backspace·한영 의도 판단을 제공하지 않는다.

## Firefox/macOS 문제는 어느 계층에서 생기는가

| 계층 | 맡는 일 | 대표 실패 |
|---|---|---|
| 자모 조합 | 초성·중성·종성 결합과 분리 | 복합 받침 이동, 삭제 후 잘못된 재결합 |
| 조합 세션 | 현재 조합, 완료 음절, 한영 후보, 확정 시점 | 같은 키를 두 번 소비, 외부 확정 후 남은 상태 재사용 |
| 호스트 통합 | 조합 영역·선택 영역·확정 문자열을 앱에 반영 | 문자 중복·유실, 공백 이동, 커서 이동 뒤 잘못된 범위 교체 |

Firefox 139에는 macOS 기본 두벌식으로 `안녕, 세상아`를 입력했을 때 공백이 뒤로 이동하는 회귀가 있었다. 수정 대상은 자모 결합식이 아니라 조합 확정 시 편집기의 선택 영역 처리였다. 이 문제는 현재 VERIFIED FIXED이며 140/141 검증 기록이 있다. 사용자가 겪었던 문제의 원인까지 같다고 단정하지 않는다. [Mozilla bug 1968843](https://bugzilla.mozilla.org/show_bug.cgi?id=1968843)

이전 Cocoa 통합 버그에서는 한 타건이 기존 조합 확정과 다음 ASCII 문자 삽입을 함께 일으킬 때 후속 입력을 잃었다. 이 역시 수정된 사례다. 한 번의 키 이벤트를 한 번의 텍스트 삽입과 동일시하면 안 된다. [Mozilla bug 1317906](https://bugzilla.mozilla.org/show_bug.cgi?id=1317906)

Gecko는 native IME와 웹 편집기가 다른 프로세스에 있을 수 있어 텍스트·선택 영역 캐시와 비동기 통지를 관리한다. 따라서 조합기 단위 테스트 성공만으로 화면 반영의 정확성을 증명할 수 없다. [Firefox IME handling guide](https://firefox-source-docs.mozilla.org/editor/IMEHandlingGuide.html)

## 기존 구현 조사

| 후보 | 확인한 구현 | 적용 판단 |
|---|---|---|
| libhangul | C 조합기. `process`, `backspace`, `flush`, preedit/commit 분리. 기본 두벌식 실행 가능 | 초기 기준·실험 백엔드로 사용. LGPL-2.1-or-later 조건과 배포·링크 방식을 제품 채택 전에 결정 |
| 구름 입력기 | Swift에서 libhangul을 호출하고, 입력 수신·조합 완료·호스트 삽입을 분리 | macOS 통합의 참고. 현재 Amu에서 구름 자체를 실행한 것은 아님 |
| kime | Rust `CharacterState`와 조합 백엔드. libhangul과 결과를 비교하는 differential fuzz harness | 테스트 방식 참고. 조합 crate도 GPL-3.0-or-later이므로 코드 복사를 기본안으로 잡지 않음 |

읽은 소스는 다음 revision으로 고정했다.

- [libhangul `23d2cd9`](https://github.com/libhangul/libhangul/blob/23d2cd9d10d516d523d60c562dbcc7ea08139a09/hangul/hangulinputcontext.c): 고정 크기 조합 상태, 12칸 stack, preedit/commit/flush 각각 64개 UCS 코드 단위 버퍼.
- [구름 `46c62e5`](https://github.com/gureum/gureum/blob/46c62e51a311c89ee084ce14eb8071b6d81f765d/OSXCore/HangulComposer.swift): libhangul 삭제 처리와 완료 문자열 큐. [InputReceiver](https://github.com/gureum/gureum/blob/46c62e51a311c89ee084ce14eb8071b6d81f765d/OSXCore/InputReceiver.swift)에서 외부 확정 요청과 앱 삽입을 처리한다.
- [kime `4a9ca83`](https://github.com/Riey/kime/blob/4a9ca83ff8029fc048f964c3a28f13b0ade235b6/fuzz/src/diff.rs): 양쪽 엔진의 `누적 완료 문자열 + 현재 조합`을 매 연산 후 비교한다. 단독 복합 자모의 표현 차이는 명시적으로 정규화한다. 이 harness의 존재가 모든 입력 동등성을 보장하지는 않는다.

Rust→C 직접 함수 호출에는 프로세스·JSON 왕복이 없다. FFI 비용만으로 Rust 재작성이 필요하다고 판단할 근거는 아직 없다. 세션마다 조합 context를 소유하고, C가 반환한 버퍼의 유효 기간과 소유권을 Rust 경계에서 제한한다. 이 경계 밖에 포인터를 노출하지 않는다. C 의존성이 있으므로 WASM·Windows 빌드는 별도 컴파일/링크 검증이 필요하다.

## 실행으로 바로잡은 Backspace 계약

기본 두벌식으로 `r k q t k`를 입력했다.

| 동작 | 조합기에서 새로 완료된 문자열 | 현재 조합(preedit) | 누적 표시 |
|---|---|---|---|
| `r` | 없음 | ㄱ | ㄱ |
| `k` | 없음 | 가 | 가 |
| `q` | 없음 | 갑 | 갑 |
| `t` | 없음 | 값 | 값 |
| `k` | 갑 | 사 | 갑사 |
| Backspace | 없음 | ㅅ | 갑ㅅ |
| Backspace | 없음 | 없음 | 갑 |
| Backspace | 조합기가 키를 소비하지 않음 | 없음 | 호스트가 갑 삭제 |

초기 설계의 `갑사 → 값`은 **일반 Backspace로 채택하지 않는다**. 원시 키를 하나 줄여 전체를 다시 조합하면 `rkqt → 값`이 되지만, 실제 입력기의 음절 완료 경계는 이미 달라졌다. 원시 키열만 보관해서는 삭제 후 상태를 복원할 수 없다.

따라서 세 계약을 분리한다.

1. **조합 삭제:** 현재 음절에서 한 입력 요소 삭제. 완료 경계를 역으로 넘어가 다시 결합하지 않는다.
2. **토큰/호스트 삭제:** 현재 음절이 없으면 세션이 아직 보유한 토큰 접두부의 마지막 글자 삭제. 세션 소유 영역도 없으면 호스트에 위임한다.
3. **자동 변환 취소:** 적용 전후 텍스트와 유효한 범위를 가진 별도의 undo 기록으로 처리한다. 일반 Backspace의 의미를 몰래 바꾸지 않는다.

한영 판별을 위해 한 단어를 조합 중으로 유지할 때, libhangul의 `commit`은 세션 내부의 완료 음절이다. 이를 즉시 호스트 commit과 동일하게 취급하지 않는다. `CompositionSession`은 완료 음절 접두부와 현재 음절을 묶어 marked text를 만들고, 단어 경계에서 `CommitText`를 만든다. 후보 재생에도 이 음절 경계를 포함한다.

## 운영체제 통합 원칙

macOS InputMethodKit을 첫 통합 경로로 삼는다. Amu 입력 소스를 한 번 선택하면 그 안에서 한영을 자동 판별하는 형태다. 다른 입력 소스가 선택돼 있어도 전역에서 가로채는 제품 형태와는 차이가 있다. Windows에서는 같은 코어를 TSF 조합 범위에 연결한다. UNIX 공통 API만으로 이 부분을 통합할 수는 없다. [Apple InputMethodKit](https://developer.apple.com/documentation/inputmethodkit), [Microsoft TSF compositions](https://learn.microsoft.com/en-us/windows/win32/tsf/compositions)

- 키 소비 여부, 현재 조합 표시, 완료 문자열, 원래 키의 통과 여부를 한 연산 결과로 전달한다. 확정 문자열과 뒤따르는 공백/기호의 순서를 보존한다.
- 조합 중인 범위를 소유하고 그 범위 안에서 우선 변환한다. 이미 앱에 확정한 이전 단어는 문맥 점수에 쓰더라도 기본적으로 다시 쓰지 않는다.
- 포커스·선택 영역·입력 소스 변경과 호스트의 외부 확정 요청을 명시적인 세션 이벤트로 다룬다. 종료 이유에 맞는 commit/cancel 정책을 정하고 같은 종료 요청을 중복 적용하지 않는다.
- 자체 operationId와 호스트에서 확인한 상태를 구별한다. API 호출 완료를 실제 화면 반영 확인으로 가장하지 않는다. 늦은 콜백이 새 입력칸의 상태를 확정하지 못하도록 generation을 확인한다.
- raw-key 범위, UTF-8 바이트, UTF-16 단위, grapheme 경계를 구별한다. 좌표 변환은 플랫폼 어댑터의 책임이다.
- 한 단어 전체를 marked text로 유지하면 자동완성·Enter 제출 동작에 영향을 줄 수 있다. 기본 입력창뿐 아니라 rich editor와 터미널에서 검증한 뒤 확정한다.

웹 데모가 호스트 IME 입력을 받아 동작하는 모드에서는 `compositionstart/update/end`와 `beforeinput/input`을 관찰하고 같은 조합을 `keydown`에서 중복 처리하지 않는다. 원시 키를 받아 Amu 조합기를 시험하는 모드는 별도로 둔다. W3C Input Events Level 2는 조사일 현재 Working Draft이며 실제 브라우저 호환성을 보증하는 문서가 아니다. [Input Events Level 2](https://www.w3.org/TR/2026/WD-input-events-2-20261015/)

## 시간·공간 복잡도

입력 길이 `n`, 현재 보유 토큰 길이 상한 `L`, 문맥 단어 수 `W`, 후보 수 `K`, 후보당 확장 수 `B`, 조합 테이블 크기 `C`, 현재 다시 표시할 문자 수 `R`, undo 보관 건수 `U`, 불변 사전 크기 `D`를 구별한다. 아래는 **구현 계획의 분석**이며 아직 없는 Amu 엔진의 실측값이 아니다.

| 작업 | 시간 | 추가 공간 | 조건·주의 |
|---|---|---|---|
| 고정 두벌식 조합/삭제 한 번 | 입력 길이 기준 O(1) | O(1) | 현재 음절과 작은 테이블만 참조. libhangul의 일반 조합 검색은 `bsearch`이므로 C를 변수로 보면 O(log C) |
| n개 키의 순수 조합 | O(n) | 상태 O(1), 결과 보관 시 O(n) | 결과 문자열 저장 비용을 숨기지 않음 |
| 토큰/음절 경계 이력 | 보통 append/pop O(1) 상각 | O(L) | 사전 할당으로 확장 비용 제한. 매 키 전체 snapshot 복사는 금지 |
| 모든 부분 문자열을 후보로 검토 | 최소 후보 개수 O(n²) | 보관 방식에 따라 O(n²) | 각 부분을 매번 길이만큼 재조합/평가하면 O(n³). 모든 분절 경로 직접 열거는 지수 증가 |
| 상한 있는 증분 beam 후보 탐색 | 키당 O(KB), 토큰당 O(LKB) | 경로·확장 버퍼 O(LK+KB) | 후보 확장·특징 갱신이 상수일 때. 상위 K 선택은 선형 selection 기준; 정렬하면 O(KB log(KB)) |
| W단어·각 K후보 문맥 DP | O(WK²) | 경로 복원 O(WK) | 인접 후보 전이 점수를 O(1)로 계산하는 모델. 후보 생성 비용 별도 |
| 전체 marked text 인코딩/전달 | O(R) | O(R) | 전체를 받는 OS API면 작은 내부 delta만으로 이 비용을 없앨 수 없음 |
| 사전과 통계 | 시작 시 로딩/검증 O(D) | 공유 O(D) | 키마다 로딩하지 않음. 사전 검색은 사용 자료구조 비용을 따름 |
| 읽기 문맥·undo 원문 보관 | 문자열 복사량에 비례 | O(WL+UR) | 조합 상태와 별도 비용. undo 건수·총 바이트도 제한 |

libhangul의 조합 검색 근거는 [hangulkeyboard.c](https://github.com/libhangul/libhangul/blob/23d2cd9d10d516d523d60c562dbcc7ea08139a09/hangul/hangulkeyboard.c)다. 고정 현대 두벌식에서 C는 입력 n과 함께 늘지 않는다. 조합 상태 O(1)은 문장 전체와 undo를 포함한 제품 메모리가 O(1)이라는 뜻이 아니다.

특히 **조합기는 O(1)이어도 매 키마다 길어지는 marked text 전체를 복사하면 토큰 전체 비용은 O(n²)**다. 현재 토큰을 제한하고 이미 확정된 긴 문장을 매번 전달하지 않아야 한다. OS IPC·렌더링·외부 앱 지연에는 우리 알고리즘의 상한만으로 실시간 보장을 붙일 수 없다.

혼합 한영 판독은 접두어 trie 상태와 소수 후보를 증분 갱신하는 beam 탐색부터 시작한다. 이는 탐색량을 제한하는 대신 최적 후보를 놓칠 수 있는 근사 방식이다. 모든 부분 문자열을 다시 점수화하는 구현은 피한다. 삭제/분절 변경 시 영향을 받은 상태만 복구하되, 전체 토큰 재평가가 필요한 경우의 비용 O(LKB)를 별도로 측정한다. 후보 문자열을 매번 통째로 복제하면 위 상한에 추가 L이 붙으므로 경로 참조를 저장한다.

초기 실험값은 `L=64키, W=8단어, K=4후보`를 제안한다. 확정 수치가 아니며 말뭉치와 p99로 조정한다. 상한에 도달하면 자동 재판단 범위를 줄이고 표시 중 텍스트를 정상 확정한다. 입력을 버리거나 기본 한글 조합을 중단하지 않는다. 이전 W단어는 읽기 문맥이며 자동 재편집 권한이 아니다.

## 이번에 실행한 검증과 다음 검증

`mise run composition:check`가 libhangul revision을 검증하고 조합에 필요한 C 소스만 clang으로 빌드한다. 한자 사전·외부 XML 키보드·입력기 설치는 사용하지 않는다. 기존 checkout의 revision이나 추적 소스가 다르면 덮어쓰지 않고 실패한다.

- 현대 한글 **11,172자**의 표준 두벌식 키열을 입력·flush해 Unicode 기대 음절과 비교: 통과.
- 같은 11,172자 각각에 입력 키 수만큼 Backspace 후 빈 상태 확인: 통과.
- 복합 받침 이동, 복합 중성 삭제, 단순 받침 이동, 확정 뒤 삭제, Shift, 기호/공백의 **6개 trace**: 매 단계 표시 비교 통과.
- [실행 근거](../research/composition-evidence.json)에 revision·컴파일러·preedit/commit/consumed 결과를 저장했다.

이 검사는 **libhangul 기준 동작의 재현**이다. Amu 조합기를 아직 구현하지 않았으므로 차등 테스트를 통과했다고 표현하지 않는다. 검사 스크립트의 간단한 호스트 모델도 실제 앱·grapheme 삭제를 대신하지 않는다.

후속 검증은 세 층으로 분리한다.

1. **조합 차등 검증:** 새 백엔드를 도입하면 libhangul과 같은 연산열을 실행한다. 표시 결과뿐 아니라 음절 완료 경계·키 소비 여부도 비교한다. 의도적인 정책 차이는 개별 fixture로 기록한다. 무작위 키·Backspace·flush·reset 및 짧은 연산열 탐색으로 발견한 실패를 최소 trace로 줄여 보관한다.
2. **세션 계약 검증:** 중복 확정, 재진입, 오래된 콜백, 포커스 이동, 입력 소스 전환, 선택 영역 교체, 붙여넣기, Return/Tab/방향키와 빠른 반복 입력을 모의 호스트에서 실행한다. 원문 유실·중복과 잘못된 입력칸 변경을 허용하지 않는다.
3. **실제 호스트 검증:** macOS TextEdit, Firefox/Chrome/Safari의 input·textarea·contenteditable, VS Code/Electron, 터미널에서 같은 시나리오를 실행하고 앱/OS 버전을 기록한다. Windows TSF도 별도 호스트 행렬로 검증한다. 아직 실행하지 않았다.

성능은 순수 조합, 언어 판독, marked text 생성, OS 반영을 따로 측정한다. Rust/C 언어 비교보다 조합 경계 정확성, 오변환율, p95/p99 지연, 원문 유실·중복 여부를 먼저 본다.

## 개발 도구 관리

[mise.toml](../mise.toml)에 Rust 1.99.1, rustfmt/clippy, 기존 연구 재현용 Node.js 22.22.1을 고정했다. 설치된 mise 2025.12.9에서 설정을 검증하고 `mise install`로 Rust를 설치했다. Rust 버전은 [공식 배포 manifest](https://static.rust-lang.org/dist/channel-rust-stable.toml)의 2026-10-15 stable을 확인해 선택했다.

Rust 버전의 단일 관리 위치는 mise.toml이다. 같은 버전을 별도 rust-toolchain.toml에 중복 수기 관리하지 않는다. 나중에 Cargo 프로젝트가 생기면 앱의 Cargo.lock도 커밋하고 `--locked`로 의존성을 재현한다. 도구 버전 갱신은 별도 변경으로 올려 조합/세션 검증을 함께 실행한다. [mise Rust 공식 문서](https://mise.jdx.dev/lang/rust.html)

현재 연구용 재현 스크립트는 macOS/Linux의 git·clang을 추가로 사용한다. macOS clang/SDK는 Xcode Command Line Tools 영역이며 mise의 Rust 설치와 별개다. 연구용 C 스크립트는 Windows에서 검증하지 않았다. Windows 입력기 자체의 MSVC 빌드와 실제 TSF 검사는 별도 CI로 실행하며 [설치·검증 범위](install-windows.md)에 기록한다.
