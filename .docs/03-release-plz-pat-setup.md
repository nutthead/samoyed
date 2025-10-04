# Release-plz PAT Setup

## Problem

GitHub Actions workflows created by `GITHUB_TOKEN` don't trigger CI workflows to prevent infinite loops. This means release PRs created by release-plz won't automatically run tests, causing them to be blocked waiting for the required "CI Success" status check that never runs.

## Solution

Use a Personal Access Token (PAT) instead of `GITHUB_TOKEN` for the release-pr workflow. The workflow now **requires** this token and will fail with a clear error message if it's not configured.

## Setup Instructions

### 1. Create a Fine-Grained Personal Access Token

1. Go to: https://github.com/settings/personal-access-tokens/new
2. Configure the token:
   - **Token name**: `samoyed-release-plz` or similar descriptive name
   - **Description**: Use this token for authenticating release-plz when creating release PRs. Ensures CI workflows trigger properly by bypassing GitHub's restriction that prevents GITHUB_TOKEN from triggering workflows on bot-created PRs. Required permissions: Contents, Pull requests, Workflows (all read+write).
   - **Expiration**: Choose appropriate duration (90 days recommended for security)
   - **Resource owner**: ⚠️ **CRITICAL** - Select **`nutthead`** (the organization), NOT your personal account
   - **Repository access**: Select "Only select repositories"
     - Choose: `nutthead/samoyed`
   - **Repository permissions**: ⚠️ **ALL THREE must be "Read and write"**
     - **Contents**: **Read and write** (required to create branches and commits)
     - **Pull requests**: **Read and write** (required to create/update PRs)
     - **Workflows**: **Read and write** (required to trigger CI workflows)
     - **Metadata**: Read-only (automatically selected)

3. Click "Generate token"
4. **Copy the token immediately** (you won't be able to see it again)

> **⚠️ Common Mistake**: Setting any of the three permissions to "Read-only" will cause a 403 error when trying to create branches. All three must be "Read and write".

### 2. Add Token as Repository Secret

#### Option A: Via GitHub Web Interface (Recommended)

1. Go to: https://github.com/nutthead/samoyed/settings/secrets/actions
2. Click "New repository secret"
3. Configure the secret:
   - **Name**: `NH_RELEASE_PLZ_TOKEN`
   - **Value**: Paste the PAT you copied
4. Click "Add secret"

#### Option B: Via GitHub CLI

```bash
gh secret set NH_RELEASE_PLZ_TOKEN --body "YOUR_PAT_TOKEN_HERE" --repo nutthead/samoyed
```

### 3. Verify Setup

The workflow at `.github/workflows/release-pr.yml` is configured to require this token. The first step validates the token exists:

```yaml
- name: Validate NH_RELEASE_PLZ_TOKEN exists
  run: |
    if [ -z "${{ secrets.NH_RELEASE_PLZ_TOKEN }}" ]; then
      echo "❌ Error: NH_RELEASE_PLZ_TOKEN secret is not configured"
      # ... helpful error message with setup instructions ...
      exit 1
    fi
```

Once the secret is added:
- Release PRs will be created with the PAT ✅
- CI workflows will trigger automatically on the PR ✅
- "CI Success" status check will run and complete ✅
- The release PR can be merged once CI passes ✅

### 4. Test

To test, merge any commit to master that modifies files matching the release-pr workflow paths. The workflow will:
1. Validate `NH_RELEASE_PLZ_TOKEN` exists (fails fast if missing)
2. Create a release PR using the PAT
3. CI workflows will automatically run on the PR
4. PR can be merged once all checks pass

## Error Handling

If `NH_RELEASE_PLZ_TOKEN` is not configured:
- The workflow will **fail immediately** with a descriptive error message
- The error message includes step-by-step setup instructions
- No release PR will be created (prevents creating PRs that can't be merged)

## Security Notes

- **Fine-grained PATs are more secure than classic PATs** - they can be scoped to specific repositories and have granular permissions
- **Limit token scope** to only required permissions (Contents, Pull requests, Workflows)
- **Set reasonable expiration** (90 days recommended) and rotate tokens periodically
- **Never commit tokens** to the repository or share them publicly
- **Store tokens securely** - GitHub repository secrets are encrypted at rest
- **Revoke old tokens** when creating new ones at: https://github.com/settings/personal-access-tokens
- **Monitor token usage** via audit logs if suspicious activity is detected

## Troubleshooting

### Release PR workflow fails with "NH_RELEASE_PLZ_TOKEN secret is not configured"

This means the token hasn't been added as a repository secret. Follow the setup instructions above.

### Error: "Resource not accessible by personal access token" (HTTP 403)

This error occurs when the token has insufficient permissions. Common causes:

1. **Permissions set to "Read-only" instead of "Read and write"**
   - All three permissions (Contents, Pull requests, Workflows) must be "Read and write"
   - Check your token at: https://github.com/settings/personal-access-tokens
   - Click "Edit" and verify all three permissions show "Read and write"

2. **Wrong Resource owner selected**
   - For organization-owned repositories, the Resource owner must be the organization (`nutthead`), not your personal account
   - You'll need to recreate the token with the correct Resource owner

3. **Organization approval pending**
   - If the organization requires PAT approval, check: https://github.com/organizations/nutthead/settings/personal-access-tokens/active
   - An organization admin must approve the token before it can be used

### Release PR is created but CI doesn't run

This indicates the token exists but may not have the **Workflows** permission set to "Read and write". Verify the token has all three required permissions at the "Read and write" level.

### Token expired

Fine-grained PATs expire after the configured duration. Create a new token following the same steps and update the `NH_RELEASE_PLZ_TOKEN` secret with the new value.
