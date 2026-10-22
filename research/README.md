# 조합과 판독 평가

이 디렉터리는 Amu의 조합·판독 평가 도구와 실행 기록을 보관한다. 내려받은 말뭉치와 라이브러리 checkout은 `.gitignore`로 제외한다.

## Amu 판독 평가

```sh
mise run core:evaluate
```

현재 기록은 [decoder-evaluation.txt](decoder-evaluation.txt)다. 개발용·평가용 난수 seed를 구분했다. 초기 평가에 쓴 샘플은 이후 수정에서도 재사용하므로 새 변경에 대한 독립적인 블라인드 평가가 아니다. 합성 약어는 실사용 분포와 다르다. 측정 범위·실패 예·공개 말뭉치 test 재현 방법은 [구현 현황](../docs/mvp.md)을 따른다.

`decoder-before-korean-statistics.txt`, `decoder-with-korean-statistics.txt`, `decoder-final-development.txt`는 개발 중의 비교 기록이며 현재 정확도의 근거로 사용하지 않는다.

## 한글 조합 기준 검사

```sh
mise run composition:check
```

Git과 clang이 필요하다. 고정 리비전의 libhangul 소스를 연구 캐시에 받아 현대 한글 11,172음절의 조합·삭제와 경계 trace를 검사한다. [기준 결과](composition-evidence.json)는 라이브러리의 검증이며 현재 Amu 전체·OS 호환성 검사를 대신하지 않는다.

## 데이터 출처

제품의 한국어 통계는 [변환 명세](../models/korean/README.md)를 따른다. `mise run model:korean`은 고정된 UD Korean GSD train 자료를 다운로드·검증한 뒤 통계를 재생성한다.
