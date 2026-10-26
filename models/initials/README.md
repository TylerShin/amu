# 초성·영문 자음 쌍 통계

이 모델은 단어·약어 조회표가 아니다. 26×26 문자 쌍마다 한국어 초성과 영문 자음의 음의 로그 빈도를 저장한다. `vs`, `fd` 등의 개발 회귀 사례는 학습 스크립트가 읽지 않는다. 세션 생성 시 676개 log odds를 준비하며 키 처리 중 파일·네트워크를 읽지 않는다.

## 자료와 변환

- [UD Korean GSD](https://github.com/UniversalDependencies/UD_Korean-GSD/tree/7628157f2ba54e0580b2c03e22f1a9f5b111d7e7)의 train FORM 중 두 음절 이상 한글 어형을 중복 제거한다. 인접 음절의 초성을 두벌식 키로 투영하고 소문자 키 쌍을 집계한다. 쌍자음의 Shift 입력은 이 모델에서 제외한다.
- [UD English EWT](https://github.com/UniversalDependencies/UD_English-EWT/tree/4a4d77f599ea53cc405f85d0cec4b2f14f81d42b)의 train FORM 중 2~6자 ASCII 영문이며 a/e/i/o/u/y가 없는 토큰을 소문자화하고 중복 제거한다. 인접 문자 쌍을 집계한다. 모음이 있는 단어는 별도 철자 근거로 다룬다.
- 반복 이름·상용구가 작은 자료를 지배하지 않도록 어형별로 한 번만 센다. 각 쌍에 0.5의 평활화 빈도를 더한다. 양쪽의 전체 쌍 빈도로 정규화한 음의 로그 값을 소수점 여섯 자리로 저장한다.
- 원문·어형 집합은 생성물에 포함하지 않는다. test 자료, 수동 초성 정답 목록, 사용자 입력도 학습하지 않는다.

GSD의 일반 단어 초성은 실제 채팅 초성의 **간접 근거**다. EWT의 해당 영문 어형도 199개뿐이다. 미등록 조합에 점수를 낼 수 있다는 사실은 모든 약어나 초성을 정확히 구별한다는 뜻이 아니다. 이 한계와 모호할 때의 동작은 [판독 정책](../../docs/decoder-policy.md)에 명시한다.

## 출처·라이선스

`pairs.tsv`는 두 원본 데이터의 변환물로 **CC BY-SA 4.0**을 유지한다. Amu 애플리케이션 코드의 MIT 라이선스와 구분한다.

- Korean GSD 기여자: Ryan McDonald, Joakim Nivre, Daniel Zeman, Jinho Choi, Na-Rae Han, Jena D. Hwang, Jayeol Chun 및 Universal Dependencies 기여자. 참고 문헌: Chun 외, *Building Universal Dependency Treebanks in Korean*, LREC 2018. [원본 고지](LICENSE-korean.txt).
- English EWT annotations © 2013–2021 The Board of Trustees of The Leland Stanford Junior University. 원문은 LDC English Web Treebank의 여러 출처에서 왔으며 일부는 public domain, 일부는 Google, Yahoo!, University of Pennsylvania 및 원저작자 권리가 있다. [원본 고지](LICENSE-english.txt)와 upstream README의 출처 표시를 유지한다.

원본 리비전, SHA-256, 입력 규모와 생성물 체크섬은 [manifest.json](manifest.json)에 기록했다. 일반 빌드는 내려받지 않으며 양쪽 OS 패키지에 모델·출처·라이선스·재생성 스크립트를 포함한다.

## 재생성

소스 checkout에서 `mise run model:initials`를 실행한다. [생성 스크립트](../../scripts/train-initial-model.mjs)는 고정된 train 파일의 체크섬을 검증한다. 원본 다운로드 캐시는 Git에서 제외한다. `mise run repo:check`는 생성물과 manifest의 체크섬을 비교한다.
