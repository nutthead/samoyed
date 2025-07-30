---
GITHUB_ISSUE:  https://github.com/nutthead/samoid/issues/8
GITHUB_PROJECT_BOARDS:
  - Feature Release: (https://github.com/orgs/nutthead/projects/5)
  - Kanban: https://github.com/orgs/nutthead/projects/6)
  - Product Launch: https://github.com/orgs/nutthead/projects/7
  - Roadmap: ttps://github.com/orgs/nutthead/projects/8
CLAUDE_PERMISSIONS: ALL GRANTED
---

# Workflow: Accomplish GitHub Issue

This workflow provides a systematic approach to accomplishing/implementing GitHub issues with precision and without getting overwhelmed.

__Execute it **WITH PRECISION** for {GITHUB_ISSUE}!__

```workflow accomplish acceptance criteria in {GITHUB_ISSUE}
--- Uses the GitHub CLI (`gh`) for all GitHub operations
--- Uses the git commands for all git operations

--- Reads the detailed acceptance criterion and thinks about it
--- Reads the source code and thinks about it
--- Thinks hard and determines if the source code is already accomplishing `ac`
--- Returns True if the source code is already accomplishing `criterion`, and False otherwise
Boolean isAlreadySatisfied(ac: AcceptanceCriterion) throws WorkflowException

--- Marks the acceptance criteria as accomplished on GitHub
void def markAcceptanceCriteria(ac: AcceptanceCriterion, as: Done | Todo) throws WorkflowException

--- Built ins
void tryUntilSuccess(List<Doable>) throws WorkflowException
void updateCode(toSatisfy: AcceptanceCriterion) throws WorkflowException
void updateAndOrWriteCodeAndPerfTests(toSatisfy: AcceptanceCriterion) throws WorkflowException
void writeOneUnitTest(toCover: Code) throws WorkflowException

--- Built on objects

--- The GitHub CLI
object gh
  void readWorkItem(GITHUB_ISSUE: Issue) throws WorkflowException
  void writeGitHubComment(about: Progress) throws WorkflowException
  void markAcceptanceCriteria(ac: AcceptanceCriterion, as: Done | Todo) throws WorkflowException
  void readAcceptanceCriterion(ac: AcceptanceCriterion) throws WorkflowException


--- The git CLI
object git =
  void stageAllChanges() throws WorkflowException
  void commitChanges(messageFormat: String) throws WorkflowException
  void createConventionalBranch(forIssue: GITHUB_ISSUE) throws WorkflowException
  void stageAllChanges() throws WorkflowException
  void commitChanges(messageFormat: String) throws WorkflowException


WORKFLOW:
  git.stageAllChanges()
  git.commitChanges(messageFormat: Conventional Commits)

  let acceptanceCriteria = ["AC8.1", "AC8.2", "AC8.3", "AC8.4", "AC8.5", "AC8.6", "AC8.7", "AC8.8"]

  gh.readWorkItem(GITHUB_ISSUE) and think hard about it
  forEach ac in acceptanceCriteria
      tryUntilSuccess
        ghreadAcceptanceCriterion(ac) and think very hard about it
        ULTRATHINK and updateAndOrWriteCodeAndPerfTests(toSatisfy: ac)
        gh.writeGitHubComment(about: Progress)
        gh.markAcceptanceCriteria(ac, Done)
        git.stageAllChanges()
        git.commitChanges(messageFormat: Conventional Commits)
```
