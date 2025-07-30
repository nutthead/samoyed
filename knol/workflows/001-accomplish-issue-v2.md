---
GITHUB_ISSUE:  https://github.com/nutthead/samoid/issues/7
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
void writeOneUnitTest(toCover: Code) throws WorkflowException
void writeGitHubComment(about: Progress) throws WorkflowException

--- Built on objects
object gh =
  void stageAllChanges() throws WorkflowException
  void commitChanges(messageFormat: String) throws WorkflowException

object git =
  void createConventionalBranch(forIssue: GITHUB_ISSUE) throws WorkflowException
  void stageAllChanges() throws WorkflowException
  void commitChanges(messageFormat: String) throws WorkflowException

let acceptanceCriteria = ["AC7.1", "AC7.2", "AC7.3", "AC7.4", "AC7.5", "AC7.6"]

git.createConventionalBranch(forIssue: GITHUB_ISSUE)

forEach ac in acceptanceCriteria
  let result = isAlreadySatisfied(ac);
  if result == True
    markAcceptanceCriteria(ac, Done)
  else
    tryUntilSuccess
      updateCode(toSatisfy: ac)
      writeOneUnitTest(toCover: new code)
      writeGitHubComment(about: Progress)
      markAcceptanceCriteria(ac, Done)
      gh.stageAllChanges()
      gh.commitChanges(messageFormat: Conventional Commits)
```
