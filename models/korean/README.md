# 한국어 음절 통계

[UD Korean GSD](https://github.com/UniversalDependencies/UD_Korean-GSD)의 리비전 `7628157f2ba54e0580b2c03e22f1a9f5b111d7e7`에서 **train 자료만** 변환했다. 원본과 생성물의 SHA-256은 [manifest.json](manifest.json)에 기록한다. 원본 말뭉치는 Git에서 제외한 연구 캐시에만 보관하며, 앱에는 문장 대신 집계한 통계를 점수로 바꿔 포함한다.

## 출처와 라이선스

기여자: Ryan McDonald, Joakim Nivre, Daniel Zeman, Jinho Choi, Na-Rae Han, Jena D. Hwang, Jayeol Chun 및 Universal Dependencies 기여자.

참고 문헌: Jayeol Chun, Na-Rae Han, Jena D. Hwang, Jinho D. Choi, *Building Universal Dependency Treebanks in Korean*, LREC 2018. Google Universal Dependency Treebank와 원문 자료의 추가 출처 표시는 upstream README를 따른다.

Amu의 변환 데이터인 `syllables.tsv`는 **CC BY-SA 4.0**으로 배포한다. [원본 라이선스 고지](LICENSE.txt)와 [라이선스 전문](https://creativecommons.org/licenses/by-sa/4.0/legalcode)을 참고한다. 데이터와 변환본을 배포할 때 출처·라이선스·변환 사실을 유지한다. 이 고지는 모델 데이터에 적용되며 별개의 애플리케이션 코드에 적용할 라이선스를 지정하지 않는다.

## 변환

정수 ID를 가진 CoNLL-U FORM 토큰 중 현대 한글 음절로만 된 토큰을 남긴다. 토큰 내부의 음절과 인접 음절 쌍을 세고, 평활화를 적용해 결정적인 음의 로그 점수로 기록한다. 같은 토큰의 LEMMA와 XPOS가 같은 수의 형태소로 정렬된 경우, 조사·어미(J/E 태그)의 첫 음절도 집계한다. `A` 행은 `(해당 음절의 조사·어미 시작 횟수 + 0.5) / (FORM 음절 횟수 + 1)`을 1 이하로 제한한 뒤 음의 로그로 변환한 점수다. 형태소 분석의 원형과 실제 표기가 달라질 수 있으므로 확률로 단정하지 않는다. 실행 시 비용을 6 이하로 제한하고, 약어와 경쟁하는 한 음절의 경계 근거로만 사용한다.

영문 문자열, 약어, 전체 단어 조회표, test 자료, 사용자 입력은 모델에 넣지 않는다.

실행 시에는 분절 길이로 점수를 정규화하고 후보 경계에 미치는 영향을 제한한다. 작은 통계적 판독 근거이며 문장 의미 모델은 아니다.

## 재생성

프로젝트 루트에서 실행한다.

```sh
mise run model:korean
```

`scripts/train-korean-model.mjs`가 고정된 원본 SHA-256을 확인한 뒤 처리한다. 일반 빌드는 저장소의 통계를 사용하므로 말뭉치를 다운로드하지 않는다. 앱 번들에는 이 안내, 통계, manifest, 라이선스와 재생성 스크립트를 함께 포함한다. 재생성은 프로젝트의 소스 checkout에서 수행한다.
