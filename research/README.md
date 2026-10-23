# 조합과 판독 평가

이 디렉터리는 Amu의 조합·판독 평가 도구와 실행 기록을 보관한다. 내려받은 말뭉치와 라이브러리 checkout은 `.gitignore`로 제외한다.

## 입력 중 표시 안정성

[2026-10-23 수정 전후 비교](typing-stability.md)는 최종 문자열과 매 키의 의도 일치, 이미 보인 키의 언어 재해석, 반복 전환, 코어 지연을 나눠 측정한다. `mise run core:stability`로 고정 의도 자료를 실행하며, 공개 말뭉치·수정 전 소스 복원·전체 비교 수치는 보고서에서 연결한다.

## Amu 판독 평가

```sh
mise run core:evaluate
```

현재 기록은 [decoder-evaluation.txt](decoder-evaluation.txt)다. 개발용·평가용 난수 seed를 구분했다. 초기 평가에 쓴 샘플은 이후 수정에서도 재사용하므로 새 변경에 대한 독립적인 블라인드 평가가 아니다. 합성 약어는 실사용 분포와 다르다. 측정 범위·실패 예·공개 말뭉치 test 재현 방법은 [구현 현황](../docs/mvp.md)을 따른다.

짧은 입력은 영문·한글 26개와 Shift 시작이 겹치지 않는 대문자 두 글자 494조합을 빈 문맥·한글/영문 1·4단어 뒤에서 평가한다. 말뭉치는 한 음절도 포함하고 빈 문맥·한글/영문 4단어 뒤로 나눈다. [decoder-before-short-input.txt](decoder-before-short-input.txt)는 수정 전 `5cfbdf8`의 코어에 당시 확장 평가 코드와 fixture만 복사한 비교 결과다. 영문 힌트와 한국어 모델은 수정 전후 동일하다.

혼합 경계 평가는 회귀 테스트와 [같은 입력 자료](../crates/amu-core/tests/support/mixed_boundaries.rs)를 사용한다. 미등록 접두부 10종 × 한글 9종 × 문맥 5종, 정상 한글·영문 각 14종 × 문맥 5종을 확인한다. [decoder-before-mixed-boundaries.txt](decoder-before-mixed-boundaries.txt)는 `d5b0bf9`의 코어에 현재 평가 코드와 자료를 복사해 실행한 결과다. 이 고정 사례들은 CI에서도 검사하며 말뭉치 전체 평가는 로컬에서 별도로 실행한다.

말뭉치 경로 다음에 출력 경로를 주면 토큰별 결과도 TSV로 저장한다. 수정 전후에 같은 자료를 사용해 행 번호·문맥·정답이 같은지 확인하고, 이전에 정답이던 행이 오답으로 바뀌었는지 대조할 수 있다. 아래 파일은 명시한 공개 말뭉치의 평가 결과이며 실제 타이핑 기록이 아니다.

```sh
mkdir -p .local
mise exec -- cargo run --release -p amu-core --example evaluate_decoder --locked -- research/upstream/korean-gsd/test.conllu .local/decoder-snapshot.tsv
```

직접 지정한 재현용 키열의 자동 판독과 한글 후보는 아래 명령으로 비교한다. 명령 인자만 출력하며 실제 타이핑을 수집하지 않는다.

```sh
mise exec -- cargo run -p amu-core --example inspect_input --locked -- 'ai' 'gksrmf AI' 'ai gksrmf'
```

`decoder-before-korean-statistics.txt`, `decoder-with-korean-statistics.txt`, `decoder-final-development.txt`는 개발 중의 비교 기록이며 현재 정확도의 근거로 사용하지 않는다.

## 한글 조합 기준 검사

```sh
mise run composition:check
```

Git과 clang이 필요하다. 고정 리비전의 libhangul 소스를 연구 캐시에 받아 현대 한글 11,172음절의 조합·삭제와 경계 trace를 검사한다. [기준 결과](composition-evidence.json)는 라이브러리의 검증이며 현재 Amu 전체·OS 호환성 검사를 대신하지 않는다.

## 데이터 출처

제품의 한국어 통계는 [변환 명세](../models/korean/README.md)를 따른다. `mise run model:korean`은 고정된 UD Korean GSD train 자료를 다운로드·검증한 뒤 통계를 재생성한다.
