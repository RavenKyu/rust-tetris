# Claude Code Instructions

## 프로젝트 개요

- **프로젝트명**: Tetris
- **언어**: Rust (Edition 2024)
- **설명**: 테트리스 게임 구현

## 파일 탐색 규칙

- 항상 Serena를 사용하여 파일을 탐색할 것
- 코드베이스를 탐색할 때 Serena의 심볼릭 도구들을 우선적으로 활용할 것
  - `get_symbols_overview`: 파일의 심볼 개요 파악
  - `find_symbol`: 심볼 검색 및 정보 조회
  - `find_referencing_symbols`: 심볼 참조 관계 파악
  - `search_for_pattern`: 패턴 기반 검색

## Rust 코딩 가이드라인

- `cargo fmt`로 코드 포맷팅 유지
- `cargo clippy`의 경고를 준수할 것
- 불필요한 `unwrap()` 사용 자제, 적절한 에러 처리 구현
- 타입 추론이 명확하지 않은 경우 명시적 타입 어노테이션 사용
- **함수형 프로그래밍 지향**
  - 불변성(immutability) 우선
  - 순수 함수(pure function) 작성
  - Iterator, map, filter, fold 등 함수형 메서드 활용
  - 사이드 이펙트 최소화

## 개발 방법론

- **스펙주도 개발(Spec-Driven Development)**
  - 작업 전 항상 `spec.md`와 `plan.md`를 참고할 것
  - `spec.md`에 존재하지 않는 요구사항이 있다면 추가
  - 요구사항 변경 시 `plan.md`도 함께 업데이트
- **TDD(Test-Driven Development)** 방식으로 작업을 진행할 것
  1. 먼저 실패하는 테스트를 작성
  2. 테스트를 통과하는 최소한의 코드 작성
  3. 리팩토링
- 테스트 실행: `cargo test`
- 새로운 기능 추가 시 반드시 테스트 코드부터 작성

## 태스크 관리

Beads (bd 명령어)를 사용하여 태스크를 관리한다.

작업 시작 전:
1. `bd ready`로 작업 가능한 태스크 확인
2. `bd update <id> --status in_progress`로 태스크 상태 변경
3. 태스크 작업 수행
4. 완료 시 `bd close <id>` 실행
5. 필요 시 `bd update <id> --notes "..."`로 메모 추가

`bd epic status <epic-id>`로 전체 진행 상황 확인 가능.

## 자율 작업 루프

1. `bd ready`로 다음 태스크 확인
2. 태스크 작업 수행
3. 테스트 실행하여 검증
4. `bd close <id>`로 태스크 완료 처리
5. `bd ready`가 빈 결과를 반환할 때까지 반복

## Git 워크플로우

### Git Worktree 활용

- 상위 디렉토리에 worktree 생성
- 이름 형식: `이슈코드-설명`
- 예시: `git worktree add ../DE-123-hotfix-bad-connect feature/DE-123-hotfix-bad-connect`

### Git Commit Message 템플릿

```
<type>(<issue-code>): <제목>

## 변경 사항
- 변경 내용 1
- 변경 내용 2

## 관련 이슈
- #<issue-code>
```

**규칙:**
- 브랜치명에서 이슈코드 추출하여 항상 포함 (예: `DE-123`)
- type은 수정 내용에 따라 선택:
  - `feat`: 새로운 기능
  - `fix`: 버그 수정
  - `refactor`: 리팩토링
  - `chore`: 빌드, 설정 등 기타 작업
  - `docs`: 문서 수정
  - `test`: 테스트 추가/수정
- 첫 줄은 제목 (50자 이내 권장)
- 본문은 Markdown 형식으로 작성

**예시:**
```
feat(DE-123): 연결 실패 시 재시도 로직 추가

## 변경 사항
- 최대 3회 재시도 로직 구현
- 지수 백오프 적용

## 관련 이슈
- #DE-123
```

## DO NOT

- `.env` 파일을 절대 커밋하지 말 것
- `unsafe` 블록은 반드시 필요한 경우에만 최소한으로 사용
- 인증 검사를 우회하지 말 것
- API 키를 클라이언트 코드에 노출하지 말 것
