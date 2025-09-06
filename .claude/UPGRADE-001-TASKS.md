# UPGRADE-001: File Tree Panel Enhancement Tasks

## Overview
파일 트리 패널 개선을 위한 단계별 작업 목록입니다. 각 작업을 완료할 때마다 체크박스를 업데이트해주세요.

## Task Checklist

### Phase 1: Analysis and Planning
- [x] **Task 1.1**: Analyze current file-tree-panel implementation
  - 현재 `src/ui/phase2_app.rs`의 파일 트리 구현 분석 완료
  - 비활성화된 상태와 하드코딩된 데모 데이터 확인
  - 개선 필요 사항 파악 완료

### Phase 2: Empty State Implementation
- [x] **Task 2.1**: Create empty state guide component with folder selection button
  - ✅ 빈 상태에서 표시할 가이드 텍스트 컴포넌트 생성
  - ✅ 폴더 선택 버튼 UI 구현
  - ✅ 적절한 아이콘과 메시지 추가

- [x] **Task 2.2**: Implement folder selection dialog functionality
  - ✅ 네이티브 폴더 선택 다이얼로그 구현 (`rfd` 크레이트 사용)
  - ✅ 사용자가 폴더를 선택할 수 있는 인터페이스 제공
  - ✅ 선택된 폴더 경로 검증 및 저장

### Phase 3: Core Functionality
- [x] **Task 3.1**: Add folder opening and file listing functionality
  - ✅ 선택된 폴더의 파일과 하위 폴더 로드 완료
  - ✅ 파일 시스템 서비스를 통한 디렉토리 탐색 구현 완료
  - ✅ 에러 핸들링 및 로딩 상태 관리 완료

- [x] **Task 3.2**: Update app state to handle folder selection
  - ✅ AppState에 선택된 폴더 경로 저장 (SettingsState.last_opened_folder 추가)
  - ✅ 폴더 변경 시 상태 업데이트 로직 구현 (handle_folder_change 메서드)
  - ✅ 지속성을 위한 설정 저장 (persistence.rs 통합)

### Phase 4: UI Enhancements
- [ ] **Task 4.1**: Create path display bar above main-content panel
  - 메인 콘텐츠 패널 상단에 경로 표시 바 추가
  - 현재 폴더 경로를 사용자 친화적으로 표시
  - 경로 세그먼트 클릭으로 상위 폴더 탐색 가능

- [ ] **Task 4.2**: Remove margin from file-tree-panel CSS
  - `assets/styles.css`에서 file-tree-panel 마진 제거
  - 레이아웃이 깨지지 않도록 주의
  - 다른 패널과의 일관성 유지

- [ ] **Task 4.3**: Remove unnecessary text from panels
  - "File Tree (Temporarily Disabled)" 텍스트 제거
  - 불필요한 설명 텍스트 정리
  - 더 깔끔한 UI로 개선

### Phase 5: Testing and Validation
- [ ] **Task 5.1**: Test folder selection and navigation functionality
  - 폴더 선택이 제대로 작동하는지 테스트
  - 파일 목록 표시가 올바른지 확인
  - 경로 표시와 네비게이션 테스트
  - 에러 상황 처리 검증

## Implementation Notes

### File Locations
- **Main App Component**: `src/ui/phase2_app.rs` (lines 300-457)
- **App State**: `src/state/app_state.rs`
- **CSS Styles**: `assets/styles.css`
- **File System Service**: `src/services/file_system.rs`

### Key Components to Modify
1. **File Tree Panel** in `phase2_app.rs`
2. **App State** for folder path management
3. **CSS Styles** for margin removal
4. **Path Display Bar** (new component)

### Technical Considerations
- Use existing `FileSystemService` for directory operations
- Leverage current `AppState` structure for state management
- Maintain accessibility features (ARIA labels, keyboard navigation)
- Ensure cross-platform compatibility for folder dialog

## Dependencies
- Native folder selection dialog (platform-specific)
- Existing file system service
- Current app state management
- CSS styling system

## Success Criteria
- [x] Analysis completed and plan created
- [ ] Empty state shows helpful guide text and folder selection button
- [ ] Users can select a folder through native dialog
- [ ] Selected folder contents are displayed in file tree
- [ ] Current folder path is shown above main content
- [ ] File tree panel has no margins
- [ ] All unnecessary text removed
- [ ] All functionality tested and working

---
**Progress**: 5/9 tasks completed (55.6%)

**Next Task**: Task 4.1 - Create path display bar above main-content panel