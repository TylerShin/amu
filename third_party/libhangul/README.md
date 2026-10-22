# libhangul 조합 소스

- 출처: https://github.com/libhangul/libhangul
- 리비전: `23d2cd9d10d516d523d60c562dbcc7ea08139a09`
- 라이선스: LGPL-2.1-or-later. COPYING과 원본 파일의 저작권 표시를 따른다.

조합에 필요한 소스·헤더 7개를 수정 없이 가져왔다. 한자, 외부 XML 키보드 로딩, gettext 및 upstream 빌드 도구는 제외했다.

Amu는 교체 가능한 공유 라이브러리로 빌드한다. Rust 코어 안에 정적으로 복사하지 않는다. 앱 번들에는 이 소스, 라이선스와 교체 빌드 안내를 함께 넣는다.
