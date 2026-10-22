# Amu 0.1.0 로컬 프로토타입

구현일: 2026-10-22. 개발자가 직접 타이핑하며 코어와 macOS 통합을 평가하는 최소 버전이다.

## 실행 흐름과 책임

```text
NSEvent
  → KeyTranslator: ANSI QWERTY + Shift를 공통 입력으로 변환
  → InputCoordinator: 코어 호출 → 결과를 순서대로 호스트에 제출
  → CoreSession / C ABI: 메모리 소유권·버퍼 길이·오류 코드 변환
  → Rust Session: 토큰 수명, 후보 선택·적용, 삭제·확정 정책
  → KoreanBuffer: 완료 음절과 현재 조합 음절을 분리해 보유
  → HangulComposer: 소유한 libhangul context로 자모 결합·삭제
  → Decoder: 전체 한글·전체 영문·양방향 혼합 후보 비교
  → 한국어 조합·음절 통계 / 영어 철자 특성 / 최근 언어 문맥
  → Update: consumed + commit + preedit
  → CompositionClient: 호스트의 조합 영역 갱신 또는 텍스트 확정
```

| 파일 | 책임 |
|---|---|
| `crates/amu-core/src/session.rs` | 키 처리의 진입점, 토큰 수명, 완료 경계와 삭제 정책 |
| `crates/amu-core/src/korean_buffer.rs` | 한글 후보의 완료 음절·현재 음절 및 조합 상태 소유 |
| `crates/amu-core/src/composer.rs` | libhangul context의 생성·호출·해제와 UCS-4 복사 |
| `crates/amu-core/src/language.rs` | 후보 생성·비교·분절 선택의 실행 흐름 |
| `crates/amu-core/src/korean_evidence.rs` | 조합 완성도·겹받침·드문 종성 조합의 구조적 특징 |
| `crates/amu-core/src/korean_statistics.rs` | 일반 한국어 자료의 음절 연결 통계와 분절 경계 점수 |
| `crates/amu-core/src/english.rs` | 문자 bigram, 모음·자음 배열, 대문자, 단어 힌트의 영문 점수 |
| `crates/amu-core/src/context.rs` | 최근 네 단어의 언어 경향; 원문은 보관하지 않음 |
| `crates/amu-ffi/src/lib.rs`, `include/amu.h` | Rust와 Swift 사이의 고정 C ABI |
| `macos/Sources/InputCoordinator.swift` | 코어 결과의 commit→preedit 반영, 재진입 확정 요청 처리 |
| `macos/Sources/AmuInputController.swift` | InputMethodKit 콜백·입력 메뉴·클라이언트 연결 |
| `macos/Sources/Playground.swift` | 설치 없이 같은 코어를 사용하는 AppKit 타이핑 창 |

조합기·언어 선택·세션·OS 효과를 분리하되 함수마다 trait이나 패키지를 만들지 않았다. 코어는 직접 호출하고 실제 호스트 경계에만 작은 `CompositionClient` 프로토콜을 둔다. FFI의 숫자 코드는 Swift의 `CoreOperation`에서 한 번만 변환한다.

## 현재 동작

- 현대 한글 두벌식, Shift 자모, 복합 중성·종성, 받침 이동과 삭제.
- 자동 판독·한글 고정·영문 고정. `hello dkssud` → `hello 안녕`, `rust gksrmf` → `rust 한글`.
- 자동 판독에서 `tkatjdsdi tkrhtlvdj` → `삼성sdi 사고싶어`. 약어 등록 없이 `삼성qzx`, `설정florp`, `모델ZKD` 같은 후보도 판독하며 입력한 대소문자를 유지한다. `githubdp → github에`처럼 영문 뒤 한글도 지원한다.
- 한 단어를 marked text로 보유하고 공백·기호·Return·Tab·방향키·단축키 앞에서 확정한 뒤 원래 키를 호스트에 전달한다.
- `rkqtk` → `갑사`에서 Backspace → `갑ㅅ`. 일반 삭제로 이전 음절 경계를 되감지 않는다.
- 삭제 후에는 그 토큰의 언어 해석을 고정한다. 토큰이 끝나거나 모두 지워지면 자동 판단을 다시 시작한다.
- 한글→영문 혼합 후보에서는 영문 끝부터 삭제한다. `삼성sdi → 삼성sd → 삼성s → 삼성` 뒤에는 보관한 한글 후보의 실제 조합 상태로 돌아간다. `갑사api`에서 영문 세 글자를 지운 뒤 한 번 더 지우면 `갑ㅅ`이 된다.
- 영문→한글 후보에서도 삭제 시 실제 한글 조합 상태를 사용한다. `github갑사 → github갑ㅅ → github갑 → github → githu`를 유지한다.
- 공백은 단어를 확정하고 최근 언어 경향을 갱신한다. 외부 Finish, 포커스 이동, 모드 전환, reset, 소유 영역 밖 삭제는 문맥을 지운다. 한글 단어 뒤의 `go`는 `해`, 영어 흐름에서는 `go`로 판단한다.
- 원시 키 64개 또는 한글 버퍼 약 192바이트에서 표시 중 토큰을 확정하고 이어 입력한다. 한 음절의 변화 때문에 한글 버퍼는 검사 경계보다 소수 바이트 더 커질 수 있으나 FFI 1,024바이트 버퍼 이내로 제한된다.
- 포커스 종료·외부 commit 요청은 현재 표시를 한 번 확정한다. 재진입 요청은 현재 반영이 끝난 뒤 처리한다.
- 한 입력 대상에 하나의 Rust 세션을 소유한다. C의 문자열 포인터는 호출 내부에서 복사하고 밖으로 노출하지 않는다. 키별 디스크·네트워크 요청이나 입력 텍스트 로그는 없다.

## 판독 근거와 비용

별도 약어 목록 `english-suffixes.txt`는 삭제했다. `sdi`나 특정 회사명에 대한 조건문이 없다. 전체 한글, 전체 영문, 한글→영문, 영문→한글 후보를 같은 진입점에서 비교한다. 후보 한 개에는 내부 경계를 최대 하나 둔다. 한글 부분은 실제 libhangul로 조합하고, 고른 후보만 편집 가능한 별도 context에 복원한다.

영문 점수는 기존 일반 영어 seed의 문자 bigram, 모음·자음 배열, 짧은 initialism 형태, 대문자 경계, 일반 단어 힌트를 함께 사용한다. 단어 목록에 없다는 이유로 후보를 버리지 않는다. 한국어는 조합 완성도와 자모·받침 특징을 보고, 혼합 후보끼리는 음절 연결 통계도 비교한다. 점수와 신뢰 차이는 수동 설계한 휴리스틱이며, 학습된 의미 모델의 확률이나 신뢰도로 해석하면 안 된다.

한국어 통계는 [UD Korean GSD](https://github.com/UniversalDependencies/UD_Korean-GSD)의 고정 리비전 train 자료에서 **한글만으로 된 FORM 토큰**을 추출해 만들었다. 45,807토큰에서 음절 1,280종·음절 쌍 21,559종의 통계를 얻었다. 영문 약어나 전체 단어 조회표는 생성하지 않는다. [출처·라이선스·변환 과정](../models/korean/README.md)과 [SHA-256 명세](../models/korean/manifest.json)를 포함하고, `mise run model:korean`으로 재생성한다. 일반 빌드·타이핑에는 네트워크가 필요 없다.

최근 문맥은 같은 입력 대상에서 공백으로 확정한 네 단어의 언어 경향이다. 입력 원문, 주변 문서, 문장 의미를 읽는 방식은 아니다. `분야`와 `부sdi`처럼 동일한 키열의 의도가 모호하면 온전한 한글을 우선한다. 대문자라도 한글 Shift 자모 키로만 구성된 `PTP` 같은 경우는 여전히 모호하다. 이미 확정한 단어를 후행 문맥으로 다시 쓰지 않는다.

세션은 libhangul context 세 개(전체 후보·탐색용·선택한 혼합 후보)를 소유한다. 영어·한국어 통계는 최초 세션 생성 때 한 번 준비하고 읽기 전용으로 공유한다. 후보마다 context를 할당하지 않는다. 최대 64키 토큰에 대해 최대 128개 후보를 비교하며, 키당 **O(n²)**, 세션 가변 공간 **O(n)**이다. 토큰 전체 최악은 O(n³), 전체 N키 입력은 K=64 상한에서 O(NK²)다. 고정 모델 공간은 별도 O(M)이며 해시 조회는 평균 O(1)을 전제한다. 이는 아직 prefix 상태를 모두 캐시하는 증분 탐색은 아니다.

2026-10-22 이 Mac의 release 코어 측정에서 일반 fixture 키 p99 **4.3µs**, 64키 토큰 p99 **106.2µs**, 최초 세션·모델 준비 **3.49ms**였다. 문자열 출력 생성까지 포함하고 macOS 호스트 반영은 제외한다. 비교 제품보다 빠르다는 근거는 아니며 호스트 부하에 따라 달라진다.

## 검증

`mise run check`:

- Cargo fmt와 Clippy 경고 없음.
- Rust 기능 테스트 17개, C ABI 테스트 5개 통과.
- 기능 테스트 하나에서 현대 한글 **11,172개 음절**의 전체 키열·조합 결과·삭제 후 빈 상태를 모두 검사한다.
- 완료 음절 뒤 삭제, 자동 판독 기본 예제, 영어 삭제 후 새 한글 조합, 모드 전환, 세션 격리, 4,096키 입력의 유실 없는 분할 확정을 검사한다.
- C ABI의 UTF-8 길이, 메모리 레이아웃, 잘못된 입력의 상태 보존을 검사한다.
- `삼성sdi 사고싶어`, 대문자 약어, 한글 뒤 다른 약어, 혼합 구간 삭제·재입력, `분야/연아/인아`의 한글 보존, 모드 전환 및 64키 경계의 혼합 후보 확정을 검사한다. C ABI의 혼합 언어 코드는 2, 공백 확정 이벤트는 kind 4다.
- 미등록 약어 검사는 대상 영문이 일반 영어 힌트에 없음을 함께 검증한다. 문맥의 세션 격리·초기화 및 10,000회 편집 이벤트의 버퍼 상한도 검사한다.

`mise run macos:smoke`:

- 합성 NSEvent를 **실제 NSTextView**의 키 처리와 marked text API에 전달한다.
- 조합 표시·삭제, 반복 commit의 중복 방지, 혼합 단어, `삼성sdi 사고싶어`의 약어 삭제·재입력·공백 확정, 미등록 `qzx`, 공백을 통한 문맥 전달·초기화, 선택 영역 교체, 동기 재진입 commit을 확인했다.
- Swift → C ABI → Rust → libhangul → AppKit 경로를 검사하지만 OS가 외부 앱에 전달하는 전체 IME 경로를 대신하지 않는다.

앱 번들의 서명·plist·상대 dylib 경로를 검사했고 현재 사용자 입력기로 등록했다. Playground 창 실행도 확인했다. Firefox·Chrome·Safari·TextEdit의 실제 시스템 입력기 타이핑 및 앱 간 전환 회귀 검사는 아직 완료하지 않았다.

InputMethodKit 서버 생성과 Objective-C 런타임의 `AmuInputController` 등록도 별도 시작 검사에서 통과했다. 설치 후 기존 시스템 입력 소스가 유지되는 것을 확인했다. 테스트 창에서 사용자가 입력 중인 내용은 후속 자동화로 덮어쓰지 않았다.

## 사전에 없는 입력의 평가

[재현 가능한 평가 코드](../crates/amu-core/examples/evaluate_decoder.rs)와 [전체 결과](../research/decoder-evaluation.txt)를 남겼다. `mise run core:evaluate`로 합성 평가와 지연을 측정할 수 있다. 개발에 사용한 샘플과 최종 평가용 난수 seed를 구분했다. 최종 샘플 결과를 본 뒤 판독 가중치는 바꾸지 않았다.

| 평가 | 결과 | 해석 범위 |
|---|---|---|
| 고정 회귀 사례 | 84/84 | 한국어 42, 영어 16, 미등록 영어 10, 혼합 16 |
| 최종 생성 소문자 약어 | 447/600 (74.5%) | 미등록 3~5글자 자음 약어 × 한글 접두부 6종 |
| 최종 생성 대문자 약어 | 594/600 (99.0%) | 별도 생성 약어 × 같은 접두부 6종 |
| UD GSD test 한국어 | 8,747/8,747 | 두 음절 이상 한글 FORM 토큰, 문맥 없는 개별 입력; 중복 포함 |

합성 약어 평가는 의도한 영문 끝부분과 한글 경계가 모두 맞아야 성공이다. 소문자 `kcwy`, `hdyr`, 대문자 `PTP` 등 실패 예도 출력한다. 생성 샘플은 실사용 분포가 아니므로 제품 전체 정확도로 일반화하지 않는다. 한국어 검증은 train과 다른 test 파일을 썼으며 한 음절·자모·문장 내 오변환까지 100%라는 뜻이 아니다.

한국어 test 재현 파일: 고정 리비전의 `ko_gsd-ud-test.conllu`, SHA-256 `3d1df99bda4800235e14bcfd915baf706eafa1a3935a75ffd32420a51e57f5aa`. 이 파일을 `research/upstream/korean-gsd/test.conllu`에 두고 `mise exec -- cargo run --release -p amu-core --example evaluate_decoder --locked -- research/upstream/korean-gsd/test.conllu`로 검사한다.

## 현재 제약

임의의 소문자 약어를 모두 구별하지는 못한다. 일반 영어 목록은 여전히 보조 근거이며, 새 판독기는 규칙·통계 기반의 실험 버전이다. 문장 의미 이해, 여러 번 교차하는 한영 분절, 후행 단어를 이용한 재판독, 개인 사전, 자동 변환 취소 기록은 아직 없다. 한글/영문 고정 모드로 조합 연결부를 따로 평가할 수 있다.

키 매핑은 ANSI QWERTY 위치와 Shift를 대상으로 한다. Caps Lock에 따른 대문자 처리, 다른 배열·세벌식·옛한글·한자 변환은 아직 제공하지 않는다. 붙여넣기는 호스트에 맡기며 붙여넣은 텍스트를 소급 변환하지 않는다.

호스트의 비동기 변경·선택 영역 이동을 일반화한 revision 확인과 부분 적용 복구는 아직 없다. `InputCoordinator`의 반영 완료는 API 호출이 끝났다는 뜻이며 화면 반영 확인을 의미하지 않는다. 정상 IME 콜백 계약에서 벗어나는 앱의 호환성은 별도 검증해야 한다. 이미 확정한 이전 단어를 자동으로 다시 쓰지 않는다.

Windows TSF·Linux 입력기 연결과 cross compilation은 아직 없다. Rust 도메인 로직은 OS와 분리되어 있지만 현재 빌드 스크립트는 호스트 clang으로 native 빌드한다. macOS 번들은 개발용 ad-hoc 서명이며 공증·자동 업데이트를 포함하지 않는다.

## 빌드·설치 유지보수

- mise로 Rust 1.99.1, rustfmt/clippy, Node 22.22.1을 고정한다. Swift/SDK는 Xcode에서 공급한다.
- Cargo.lock을 포함하고 빌드·검사에 `--locked`를 사용한다. 현재 Rust 외부 crate 의존성은 없다.
- libhangul 소스는 지정 revision의 조합 관련 7개 파일을 수정 없이 포함한다. `COPYING`과 출처를 유지하고 별도 dylib로 배포한다.
- 빌드는 새 디렉터리에서 서명·검증한 뒤 기존 번들과 교체한다. 실행 중인 앱의 바이너리 파일을 덮어쓰지 않는다. 기존 빌드는 `dist/.previous/`에 남는다.
- 설치 시 동일 식별자의 기존 앱은 `~/Library/Application Support/Amu/backups/`에 보존한다. 다른 식별자의 동명 앱은 덮어쓰지 않는다.
- 입력기 재설치 후 기존 프로세스가 남아 있으면 다른 입력 소스로 전환하고 다시 선택한다. macOS가 이전 프로세스를 계속 사용하면 재로그인이 필요하다. 설치 프로그램이 다른 입력기나 시스템 서비스를 종료하지 않는다.

다음 구현 순서는 실제 호스트 호환성 trace 확보, 혼합 분절·보존 규칙 확장, 실사용 문맥 평가와 OS 반영까지 포함한 지연 측정이다. 조사 단계의 전체 설계가 이미 구현됐다는 뜻은 아니다.
