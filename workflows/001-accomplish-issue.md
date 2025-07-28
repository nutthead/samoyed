---
GITHUB_ISSUE:  https://github.com/nutthead/samoid/issues/5
GITHUB_PROJECT_BOARDS:
  - Feature Release: (https://github.com/orgs/nutthead/projects/5)
  - Kanban: https://github.com/orgs/nutthead/projects/6)
  - Product Launch: https://github.com/orgs/nutthead/projects/7
  - Roadmap: ttps://github.com/orgs/nutthead/projects/8
CLAUDE_PERMISSIONS: ALL GRANTED
---

# Workflow: Accomplish GitHub Issue

This workflow provides a systematic approach to implementing GitHub issues with precision and without getting overwhelmed.

**__Execute it, <em>WITH PRECISION</em> for {GITHUB_ISSUE}!__**

## Type Definitions

```typescript
type MultilineString = Str | List<Str>
type Issue = URI | Num | Str
type Result = { details: MultilineString }
type Task = {
  codePart: List<Task>
  testPart: List<Task>
  checkboxItems: List<Str>
}
type NaturalLanguage = MultilineString | Freeform
```

## Core Functions

### Label Management
```typescript
// Enhanced label management with validation
def validateAndUpdateLabels(issue: Issue, labels: List<Str>): Result {
  availableLabels = getLabels(repo)
  for label in labels {
    if (!availableLabels.contains(label)) {
      // Try to find similar existing label
      similarLabel = findSimilarLabel(label, availableLabels)
      if (similarLabel) {
        Log.info(`Using existing label "${similarLabel}" instead of "${label}"`)
        addLabel(issue, similarLabel)
      } else {
        // Fallback to comment-based status tracking
        Log.info(`Label "${label}" not found, using comment fallback`)
        addComment(issue, `Status: ${extractStatusFromLabel(label)}`)
      }
    } else {
      addLabel(issue, label)
    }
  }
}

def updateIssueStatus(issue: Issue, forTask: Task | Str): Result {
  // Check available labels first
  availableLabels = getLabels(repo)


  // Try to find appropriate status label
  statusLabels = availableLabels.filter(label => label.contains("status:"))


  if (statusLabels.length > 0) {
    // Use existing status label pattern
    targetLabel = findBestStatusLabel(forTask, statusLabels)
    if (targetLabel) {
      addLabel(issue, targetLabel)
    }
  }


  // Always add comment as primary communication method
  addComment(issue, generateStatusComment(forTask))

  return Result("Status updated via labels and comments")
}
```

### Core Workflow Functions
```typescript
def tryUntilSuccess(anything: Anything): Result
def accomplish(task: Task | Array<Task>): Result
def updateIssueDetails(details: List<Comment | Status | Label | NaturalLanguage>): Result
def updateProjectBoard(issue: Issue, board: ProjectBoard | List<ProjectBoard>): Void
def divideIssue(issue: Issue): List<Task>
def stop(): Void
```

### Enhanced Git Operations
```typescript
object Git = {
  checkoutNewBranch(): Result {
    branchName = generateBranchName(issue)
    return executeCommand(`git checkout -b ${branchName}`)
  }

  stageChanges(): Result {
    // Stage only source files, exclude build artifacts
    return executeCommand("git add src/ *.toml *.lock *.md")
  }

  commit(messageFormat: Str): Result {
    // Use conventional commits with issue reference
    message = formatCommitMessage(messageFormat, issue)
    return executeCommand(`git commit -m "${message}"`)
  }

  push(): Result {
    currentBranch = getCurrentBranch()
    return executeCommand(`git push -u origin ${currentBranch}`)
  }
}
```

### Logging with Enhanced Error Handling
```typescript
object Log = {
  info(rawMessage: Str): MultilineString {
    return `🔍 Log.info: ${rawMessage}`
  }

  success(rawMessage: Str): MultilineString {
    return `✅ Log.success: ${rawMessage}`
  }

  error(rawMessage: Str): MultilineString {
    return `❌ Log.error: ${rawMessage}`
  }

  warning(rawMessage: Str): MultilineString {
    return `⚠️ Log.warning: ${rawMessage}`
  }
}
```

## Main Workflow

```typescript
BEGIN:
  try {
    // 1. Initialize and validate environment
    let validateResult = MUST tryUntilSuccess(validateEnvironment())
    Log.info(validateResult)

    // 2. Update issue status with enhanced label handling
    let updateResult = MUST tryUntilSuccess(
      updateIssueStatus(GITHUB_ISSUE, forTask: NaturalLanguage("Working on {GITHUB_ISSUE}"))
    )
    Log.info(updateResult)

    // 3. Create feature branch
    let checkoutResult = MUST tryUntilSuccess(Git.checkoutNewBranch())
    Log.info(checkoutResult)

    // 4. Analyze and divide issue into manageable tasks
    let smallWorkUnitList = MUST tryUntilSuccess(divideIssue(GITHUB_ISSUE))
    Log.info(`Divided into ${smallWorkUnitList.length} tasks`)

    // 5. Execute each task systematically
    forEach workUnit in smallWorkUnitList {
      Log.info(NaturalLanguage(`Starting work unit: ${workUnit.description}`))

      // Implement the code part
      let accomplishmentResult = MUST tryUntilSuccess(accomplish(workUnit.codePart))
      Log.success(NaturalLanguage(`Accomplished code implementation: ${accomplishmentResult}`))

      // Implement and run tests
      if (workUnit.testPart.length > 0) {
        let testResult = MUST tryUntilSuccess(accomplish(workUnit.testPart))
        Log.success(NaturalLanguage(`Tests implemented and passing: ${testResult}`))
      }

      // Commit changes with proper staging
      let gitResult = MUST tryUntilSuccess([
        Git.stageChanges(),
        Git.commit(messageFormat: "conventional"),
        Git.push()
      ])
      Log.info(NaturalLanguage(`Changes committed and pushed: ${gitResult}`))

      // Update issue progress with fallback handling
      let statusResult = MUST tryUntilSuccess(
        updateIssueDetails([
          Comment(`Completed: ${workUnit.description}`),
          NaturalLanguage("Progress update with task completion")
        ])
      )
      Log.info(statusResult)
    }

    // 6. Final status update and project board sync
    let completionResult = MUST tryUntilSuccess(
      updateIssueDetails([
        Comment("✅ All tasks completed successfully"),
        NaturalLanguage("Implementation ready for review")
      ])
    )
    Log.success(completionResult)

    let boardResult = MUST tryUntilSuccess(
      updateProjectBoard(issue: GITHUB_ISSUE, boards: GITHUB_PROJECT_BOARDS)
    )
    Log.info(boardResult)

  } catch (error) {
    Log.error(NaturalLanguage(`Workflow failed: ${error.message}`))

    // Enhanced error recovery
    let recoveryActions = [
      "Save current progress",
      "Document error details",
      "Create recovery issue if needed",
      "Notify stakeholders"
    ]

    forEach action in recoveryActions {
      tryUntilSuccess(executeRecoveryAction(action))
    }

    stop()
  }
```

## Validation Functions

```typescript
def validateEnvironment(): Result {
  checks = [
    validateGitRepo(),
    validateGHCLI(),
    validatePermissions(),
    validateLabels()
  ]

  return aggregateResults(checks)
}

def validateLabels(): Result {
  availableLabels = getLabels(repo)
  requiredPatterns = ["status:", "phase:", "priority:"]

  missingPatterns = []
  for pattern in requiredPatterns {
    if (!availableLabels.any(label => label.contains(pattern))) {
      missingPatterns.add(pattern)
    }
  }

  if (missingPatterns.length > 0) {
    Log.warning(`Missing label patterns: ${missingPatterns}`)
    return Result("Labels validated with warnings")
  }

  return Result("All label patterns available")
}
```

## Error Recovery Strategies

1. **Label Failures**: Fall back to comments for status tracking
2. **Git Failures**: Attempt to recover working directory state
3. **Network Failures**: Implement retry with exponential backoff
4. **Permission Failures**: Document and escalate to user
5. **Build Failures**: Capture logs and create debugging context

## Success Criteria

- ✅ All acceptance criteria implemented
- ✅ Tests passing
- ✅ Code committed and pushed
- ✅ Issue updated with progress
- ✅ Project boards synchronized
- ✅ Documentation updated
- ✅ Ready for code review


This workflow ensures robust issue accomplishment with comprehensive error handling and graceful degradation when GitHub API operations fail.
