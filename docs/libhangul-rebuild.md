# 번들에 포함된 libhangul 교체

앱은 LGPL-2.1-or-later의 libhangul을 별도 공유 라이브러리로 사용한다. 수정하지 않은 조합 소스와 COPYING은 `Contents/Resources/libhangul-source`에 있다.

앱을 종료하고 번들의 `Contents` 디렉터리에서 아래 명령을 실행하면 호환되는 대체 라이브러리를 빌드할 수 있다.

```sh
clang -std=c11 -O2 -fPIC -dynamiclib -mmacosx-version-min=13.0 \
  -DENABLE_EXTERNAL_KEYBOARDS=0 -install_name @rpath/libhangul.dylib \
  Resources/libhangul-source/hangulctype.c \
  Resources/libhangul-source/hangulinputcontext.c \
  Resources/libhangul-source/hangulkeyboard.c \
  -o Frameworks/libhangul.dylib
codesign --force --sign - Frameworks/libhangul.dylib
codesign --force --sign - ..
```

앱과 같은 CPU 아키텍처로 빌드해야 한다. 현재 로컬 개발 빌드는 ad-hoc 서명을 사용하며 hardened runtime의 라이브러리 검증을 활성화하지 않는다.

위 명령은 함께 배포한 소스 범위를 위한 안내다. 외부 키보드 로딩 등 기능을 추가하려면 해당 의존성도 준비해야 한다.
