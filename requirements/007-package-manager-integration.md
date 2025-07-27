# Requirement 007: Package Manager Integration

## Basic Information
- **ID**: 7
- **Title**: Package Manager Integration
- **Type**: Functional
- **Priority**: Medium
- **Status**: Approved
- **Phase**: Elaboration

## Description
Detect and integrate with different package managers (npm, pnpm, yarn, bun) to provide appropriate default configurations.

## Source
Reverse engineered from `husky/bin.js:18` - package manager detection

## Rationale
Different package managers have different conventions for test commands and script execution.

## Acceptance Criteria
- [ ] Detect package manager from `npm_config_user_agent` environment variable
- [ ] Generate appropriate test command in sample hooks:
  - [ ] `npm test` for npm
  - [ ] `pnpm test` for pnpm
  - [ ] `yarn test` for yarn
  - [ ] `bun test` for bun
- [ ] Default to `npm test` if detection fails
- [ ] Support manual override of package manager

## Dependencies
- Environment variable access
- String parsing utilities

## Effort
3 story points

## Planned For Iteration
Sprint 2

## Labels
- `package-managers`
- `integration`
- `compatibility`

## Traceability

### Use Cases
- Project uses pnpm and needs pnpm-specific commands
- Yarn project requires yarn test command
- Bun project needs bun-specific integration

### Test Cases
- Test detection of each package manager
- Test fallback to npm default
- Test generated hook commands
- Test manual override capability

### Design Elements
- Package manager detection utilities
- Command generation system