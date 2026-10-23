# 외부 코드와 데이터

Amu에서 작성한 코드·문서는 [MIT 라이선스](LICENSE)를 따른다. 아래 외부 코드와 데이터에는 각각의 라이선스가 적용된다. 원본 파일의 저작권 표시를 유지한다.

| 구성 요소 | 용도 | 출처와 고정 리비전 | 라이선스 |
|---|---|---|---|
| libhangul 조합 소스 7개 | 두벌식 조합·삭제 | [libhangul/libhangul](https://github.com/libhangul/libhangul/tree/23d2cd9d10d516d523d60c562dbcc7ea08139a09) | [LGPL-2.1-or-later](third_party/libhangul/COPYING) |
| 한국어 음절 통계 | 혼합 후보의 한글 경계 점수 | [UD Korean GSD](https://github.com/UniversalDependencies/UD_Korean-GSD/tree/7628157f2ba54e0580b2c03e22f1a9f5b111d7e7)의 train 자료를 변환 | [CC BY-SA 4.0](models/korean/LICENSE.txt) |
| 초성·영문 자음 쌍 통계 | 짧은 자음 입력 후보 비교 | [UD Korean GSD](models/korean/README.md)와 [UD English EWT](models/initials/README.md)의 고정 train 자료를 변환 | [CC BY-SA 4.0](models/initials/LICENSE-english.txt) |

## 초성·영문 자음 쌍 통계

일반 한국어 어형의 초성과 짧은 영문 자음 어형에서 인접 문자 쌍을 집계했다. 기여자·원문 권리·고정 리비전은 [모델 안내](models/initials/README.md), [Korean 고지](models/initials/LICENSE-korean.txt), [English 고지](models/initials/LICENSE-english.txt)에 유지한다. 생성물 `models/initials/pairs.tsv`는 CC BY-SA 4.0이며 애플리케이션 코드와 별개다. 원문·단어 조회표·수동 약어 목록을 배포하지 않는다.

## libhangul

원본 조합 소스·헤더를 수정 없이 가져왔고, 앱에서 교체할 수 있는 공유 라이브러리로 빌드한다. 앱 번들에는 해당 소스, COPYING, [교체 빌드 안내](docs/libhangul-rebuild.md)를 함께 넣는다. 자세한 범위는 [libhangul 안내](third_party/libhangul/README.md)를 따른다.

## 한국어 음절 통계

기여자: Ryan McDonald, Joakim Nivre, Daniel Zeman, Jinho Choi, Na-Rae Han, Jena D. Hwang, Jayeol Chun 및 Universal Dependencies 기여자.

참고 문헌: Jayeol Chun, Na-Rae Han, Jena D. Hwang, Jinho D. Choi, *Building Universal Dependency Treebanks in Korean*, LREC 2018. 원본 자료의 추가 출처 표시는 upstream README를 따른다.

Amu는 한글로만 이루어진 FORM 토큰에서 음절·인접 음절 쌍을 세고 평활화한 점수로 변환했다. 이 변환 데이터인 `models/korean/syllables.tsv`에도 CC BY-SA 4.0을 적용한다. 원문 문장이나 영문 약어 목록은 모델에 포함하지 않는다. [변환 설명](models/korean/README.md), [리비전·SHA-256](models/korean/manifest.json), [재생성 스크립트](scripts/train-korean-model.mjs)를 제공한다.

앱에는 통계·출처·라이선스·재생성 스크립트를 함께 넣는다. 원본 말뭉치는 다운로드 캐시에만 저장한다. 빌드에 포함된 모델 데이터의 조건은 Amu 코드의 MIT 표기와 별도로 유지한다.

## Windows 런타임

Windows 패키지에는 빌드에 사용한 MSVC의 재배포용 C/C++ 런타임 DLL을 포함한다. 이 파일은 Amu의 MIT 라이선스가 아닌 Microsoft Visual Studio 재배포 조건을 따른다. Windows SDK와 MSVC는 사용자 입력 처리 중 내려받지 않는다. [Microsoft 런타임 배포 안내](https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files)와 [Windows libhangul 교체 안내](docs/libhangul-rebuild-windows.md)를 참고한다.
