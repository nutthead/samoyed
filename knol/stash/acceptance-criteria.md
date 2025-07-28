## Acceptance Criteria Format Standards

### Required Format for GitHub Issues

Use this format for all acceptance criteria in GitHub issues:

**For 9+ acceptance criteria (use phases):**
```
#### Phase <n>: <Phase Name> (<n> story points)
- [ ] **AC<issueNumber>.<n>** Description of acceptance criterion
- [ ] **AC<issueNumber>.<n>** Description of acceptance criterion
```

**For <9 acceptance criteria (no phases):**
```
- [ ] **AC<issueNumber>.<n>** Description of acceptance criterion
- [ ] **AC<issueNumber>.<n>** Description of acceptance criterion
```

**Examples:**

*Multi-phase format:*
```
#### Phase 1: Core Infrastructure (8 story points)
- [ ] **AC5.1** Implement `IsolatedEnvironment` trait and `MockIsolatedEnvironment`
- [ ] **AC5.2** Implement `IsolatedFileSystem` trait with temp directory management
- [ ] **AC5.3** Create `TestContext` with automatic cleanup

#### Phase 2: Advanced Features (5 story points)
- [ ] **AC5.4** Implement `StatefulMock` system for complex behavior simulation
- [ ] **AC5.5** Add `TimeProvider` abstraction for time-dependent testing
```

*Simple format:*
```
- [ ] **AC5.1** Implement `IsolatedEnvironment` trait and `MockIsolatedEnvironment`
- [ ] **AC5.2** Implement `IsolatedFileSystem` trait with temp directory management
- [ ] **AC5.3** Create `TestContext` with automatic cleanup
```

### Guidelines:
- Always prefix with `AC<issueNumber>.` for traceability
- Use descriptive but concise criterion descriptions
- Group into logical phases for complex features (9+ criteria)
- Assign story points to phases, not individual criteria
- Ensure criteria are testable and unambiguous

## Session Reminders

### TODO: Work on permission issues
The Samoid hook runner is encountering permission issues during commit operations:
- `.samoid/_/prepare-commit-msg: 3: exec: .git/COMMIT_EDITMSG: Permission denied`
- This affects the git commit workflow when hooks are enabled
- Need to investigate and fix the permission handling in the hook runner implementation
