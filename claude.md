# Claude Code Instructions

> **IMPORTANT**: 모든 세션 시작 시 이 파일 전체를 읽고 워크플로우를 따를 것.
> 특히 `bd ready` 확인 없이 작업을 시작하지 말 것.

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
  - 작업 전 항상 spec 문서를 참고할 것
  - spec에 존재하지 않는 요구사항이 있다면 추가
- **TDD(Test-Driven Development)** 방식으로 작업을 진행할 것
  1. 먼저 실패하는 테스트를 작성
  2. 테스트를 통과하는 최소한의 코드 작성
  3. 리팩토링
- 테스트 실행: `cargo test`
- 새로운 기능 추가 시 반드시 테스트 코드부터 작성

---

## 작업 관리 전략

### 문서 계층 구조

- **저장 위치**: 모든 문서는 현재 프로젝트 내에 저장한다

```
docs/specs/           # 기능별 spec 문서 (영구 보관)
├── feature-a.md
├── feature-b.md
└── integration.md
```

### 도구별 역할

#### 1. spec.md - 설계 문서
- **용도**: 요구사항, 아키텍처 결정, 기술 선택 이유
- **생성 시점**: 새 기능 시작 전
- **업데이트**: 설계 변경 시
- **위치**: `docs/specs/{feature-name}.md`

#### 2. GitHub Issues - 외부 추적
- **용도**: 마일스톤, PR 연결, 팀 공유, 이력 보존
- **생성 시점**: spec 확정 후
- **라벨 규칙**:
  - `epic`: 큰 기능 단위
  - `task`: 구현 단위
  - `bug`: 버그 수정
  - `docs`: 문서 작업
- **닫는 시점**: PR 머지 또는 기능 완료

#### 3. Beads - 세션 실행
- **용도**: 일일 작업 추적, 컨텍스트 보존, 의존성 관리
- **생성 시점**: GitHub Issue 작업 시작 시
- **닫는 시점**: 작업 단위 완료 시 (세션 내)
- **GitHub 연결**: notes에 issue 번호 기록

---

## 워크플로우

### 새 기능 시작

```bash
# 1. Spec 작성
# "docs/specs/{feature}.md 작성해줘"

# 2. GitHub Issue 생성
gh issue create --title "feat: {기능명}" \
  --label "epic" \
  --body "spec: docs/specs/{feature}.md"

# 3. 하위 Issue 생성
gh issue create --title "{기능명}: {세부 작업}" \
  --label "task" \
  --body "parent: #{epic-number}"
```

### 작업 세션 시작

```bash
# 1. GitHub Issue 확인
gh issue view 1 --json title,body,labels,state
# 또는
gh issue list --label "task" --state open

# 2. Beads에 세션 작업 생성 (GitHub Issue 연결)
bd create "{작업명}" -p 1 --notes "gh:#{issue-number}"

# 3. 세부 작업 분해
bd create "{세부 작업 1}" -p 1 --parent <task-id>
bd create "{세부 작업 2}" -p 2 --parent <task-id>
```

### 작업 완료

```bash
# 1. Beads task 완료
bd close <task-id> --reason "구현 완료, PR #{pr-number}"

# 2. 모든 하위 beads 완료 시 GitHub Issue 닫기
gh issue close {issue-number} --reason "completed"

# 3. Epic의 모든 task 완료 시 Epic도 닫기
gh issue close {epic-number} --reason "completed"
```

---

## ID 연결 규칙

- Beads notes에 GitHub Issue 번호 기록: `--notes "gh:#{number}"`
- GitHub Issue body에 spec 경로 기록: `spec: docs/specs/xxx.md`
- PR에 Issue 연결: `closes #{number}`

---

## 컨텍스트 복원 순서

세션 시작 시:
1. `bd ready` - 진행 중인 작업 확인
2. `bd show <task-id>` - notes에서 GitHub Issue 번호 확인
3. `gh issue view <number>` - 전체 맥락 파악
4. spec 파일 읽기 - 설계 의도 확인

---

## 상태 동기화

| Beads 상태 | GitHub Issue 상태 |
|-----------|------------------|
| open | open |
| in_progress | open (assignee 지정) |
| blocked | open + label:blocked |
| closed | open (하위 작업일 때) 또는 closed |

---

## 자율 작업 루프

1. `bd ready`로 다음 태스크 확인
2. 태스크 작업 수행
3. 테스트 실행하여 검증
4. `bd close <id>`로 태스크 완료 처리
5. `bd ready`가 빈 결과를 반환할 때까지 반복

---

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

---

## 프롬프트 템플릿

### 새 기능 시작
```
{feature} 기능을 시작할게.
1. docs/specs/{feature}.md 작성
2. GitHub epic issue 생성
3. 하위 task issue 생성
4. 첫 번째 task를 beads로 분해
```

### 세션 재개
```
bd ready 확인하고,
연결된 GitHub issue와 spec을 읽은 뒤
다음 작업 진행해줘.
```

### 작업 완료
```
현재 beads task 완료 처리하고,
관련 GitHub issue 상태도 업데이트해줘.
PR 필요하면 생성해줘.
```

---

## 요약

| 언제 | 무엇을 |
|------|--------|
| 기능 기획 시 | spec.md 작성 |
| 구현 시작 시 | GitHub Issue 생성 (spec 링크) |
| 매 세션 | Beads로 분해 (Issue 번호 연결) |
| 작업 완료 | Beads close → Issue close → PR |
| 나중에 "왜?" | spec.md 참조 |
| 나중에 "언제?" | GitHub Issue 히스토리 |

---

## DO NOT

- `.env` 파일을 절대 커밋하지 말 것
- `unsafe` 블록은 반드시 필요한 경우에만 최소한으로 사용
- 인증 검사를 우회하지 말 것
- API 키를 클라이언트 코드에 노출하지 말 것
