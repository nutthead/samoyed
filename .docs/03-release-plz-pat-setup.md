# Release-plz PAT Setup

## Problem

GitHub Actions workflows created by `GITHUB_TOKEN` don't trigger CI workflows to prevent infinite loops. This means release PRs created by release-plz won't automatically run tests.

## Solution

Use a Personal Access Token (PAT) instead of `GITHUB_TOKEN` for the release-pr workflow.

## Setup Instructions

### 1. Create a Fine-Grained Personal Access Token

1. Go to: https://github.com/settings/tokens?type=beta
2. Click "Generate new token"
3. Configure the token:
   - **Name**: `release-plz-token` (or similar)
   - **Expiration**: Choose appropriate duration (90 days recommended)
   - **Repository access**: Select "Only select repositories"
     - Choose: `nutthead/samoyed`
   - **Permissions**:
     - Repository permissions:
       - **Contents**: Read and write (to create branches)
       - **Pull requests**: Read and write (to create/update PRs)
       - **Metadata**: Read-only (automatically selected)

4. Click "Generate token"
5. **Copy the token immediately** (you won't be able to see it again)

### 2. Add Token as Repository Secret

1. Go to: https://github.com/nutthead/samoyed/settings/secrets/actions
2. Click "New repository secret"
3. Configure the secret:
   - **Name**: `RELEASE_PLZ_TOKEN`
   - **Value**: Paste the PAT you copied
4. Click "Add secret"

### 3. Verify Setup

The workflow at `.github/workflows/release-pr.yml` is already configured to use this token:

```yaml
token: ${{ secrets.RELEASE_PLZ_TOKEN || secrets.GITHUB_TOKEN }}
```

Once the secret is added:
- Release PRs will trigger CI workflows ✅
- Tests will run automatically ✅
- The release PR can be merged once CI passes ✅

### 4. Test

To test, merge any commit to master that changes version in `Cargo.toml`. The release-pr workflow will:
1. Create a release PR using the PAT
2. CI workflows will automatically run on the PR
3. PR can be merged once all checks pass

## Fallback Behavior

If `RELEASE_PLZ_TOKEN` is not configured:
- Workflow falls back to `GITHUB_TOKEN`
- Release PR will be created successfully
- **But CI won't run automatically** (must be triggered manually)

## Security Notes

- Fine-grained PATs are more secure than classic PATs
- Limit token scope to only required permissions
- Set reasonable expiration (rotate tokens periodically)
- Never commit tokens to the repository
