# Commit automaticatty

코드 변경사항을 감지하면 다음 단계로 git commit을 실행해줍니다. : $ARGUMENTS

## 커밋 메시지 생성 단계

1. git status로 변경사항 확인
2. 변경된 파일들의 내용을 분석
3. 변경사항의 성격 파악 (기능 추가, 버그 수정, 리팩토링 등)
4. Conventional Commits 규칙에 따라 commit message 생성
5. git add . && git commit -m "commit message" 실행
6. 커밋 완료 알림

## 커밋 에시지 예시

- feat: implement user authentication system
- fix: resolve memory leak in data processing
- docs: update API documentation
- style: format code according tO eslint rules
