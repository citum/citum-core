---
name: jj-workflows
description: Jujutsu (jj) version control workflows as a modern alternative to Git. Covers change management, bookmarks (branches), conflict resolution, and GitHub integration without staging areas. Use when managing commits, creating branches, or performing advanced history operations with jj.
---

# Jujutsu (jj) Workflows

Master Jujutsu workflows for modern, flexible version control. jj streamlines commit management and history operations while maintaining full Git compatibility.

## When to Use This Skill

- Creating and managing changes (commits)
- Working with bookmarks (jj's equivalent of branches)
- Handling conflicts with automatic resolution
- Applying changes across bookmarks
- Working with multiple changes simultaneously
- Recovering from mistakes with the undo system
- Preparing changes for GitHub via Git remote

## Key Concepts vs Git

jj fundamentally differs from Git in workflow philosophy:

| Concept | Git | jj |
|---------|-----|-----|
| Basic unit | Commit (immutable after creation) | Change (mutable, auto-tracked) |
| Staging | Explicit `git add` (two-phase) | Automatic (changes tracked in real-time) |
| Branches | Persistent named pointers | Temporary bookmarks + change tracking |
| Rebasing | Explicit operation | Automatic when updating from parent |
| History | Linear or merge commits | DAG with automatic conflict handling |
| Undo | Reflog + manual recovery | Built-in `jj undo` (powerful recovery) |

## Core Workflow: The Change Model

jj tracks **changes**, not commits. The working directory is always "uncommitted" and tracked as the current anonymous change.

```
┌─────────────────────────────────────────┐
│ Working Directory (Current Change)      │
│ - Automatically tracked                 │
│ - No staging area needed                │
└─────────────────────────────────────────┘
         ↓ (jj new)
┌─────────────────────────────────────────┐
│ Previous Changes (Immutable)            │
│ - Form the project history              │
│ - Can be rebased, reordered, squashed   │
└─────────────────────────────────────────┘
```

## Essential Commands

### 1. Creating and Describing Changes

```bash
# Check current status (no staging needed)
jj status

# Show diff of current change
jj diff

# Describe current change (set commit message)
jj describe -m "feat: add user authentication

Support OAuth2 and LDAP providers.
Refs: #42"

# Update existing description
jj describe  # Opens editor
jj describe --message "new message"

# Create new change (move current work to a new change)
jj new -m "feat: next feature
description here"

# Squash current change into parent (combine commits)
jj squash
```

### 2. Managing Bookmarks (Branches)

Bookmarks are temporary named pointers to changes. Unlike Git branches, they're lightweight and don't track ancestry.

```bash
# List bookmarks
jj bookmark list

# Create bookmark at current change
jj bookmark set my-feature

# Move bookmark to different change
jj bookmark set -r abc123 my-feature

# Delete bookmark
jj bookmark delete my-feature

# Rename bookmark
jj bookmark set -r abc123 new-name
jj bookmark delete old-name
```

### 3. Viewing and Navigating History

```bash
# View commit graph (replace git log)
jj log

# View compact graph
jj log --oneline

# Show specific change details
jj show abc123

# Show changes in range
jj log -r "main..HEAD"

# Find changes by author
jj log -r "author(name)"
```

### 4. Working with Changes

```bash
# Move to specific change (checkout equivalent)
jj edit abc123

# Update current change from parent
jj rebase -d main

# Rebase change onto different parent
jj rebase -d other-bookmark

# Duplicate current change (create copy)
jj duplicate

# Abandon current change (delete it)
jj abandon

# Split current change into multiple changes
jj split

# Move files from current change to parent
jj move --to-parent path/to/file

# Move files from parent to current change
jj move --from-parent path/to/file
```

### 5. Git Integration

jj works seamlessly with Git remotes. No git commands needed for push/pull.

```bash
# Push current bookmark to Git remote
jj git push

# Push specific bookmark
jj git push -b my-feature

# Push all bookmarks
jj git push --all

# Import changes from Git remote
jj git fetch

# Sync with remote (fetch + fast-forward)
jj git fetch && jj rebase -d origin/main

# Clone from Git
jj git clone https://github.com/user/repo.git
```

### 6. Conflict Resolution

jj automatically handles many conflicts. For conflicts requiring manual resolution:

```bash
# Check conflict status
jj status  # Shows conflicted files

# Resolve conflicts in your editor
# Edit conflicted files manually
# jj shows conflict markers like Git

# Mark as resolved (stage all changes)
jj resolve

# Abandon conflict resolution and revert
jj abandon
```

### 7. Advanced Operations

```bash
# Rebase range of changes
jj rebase -r "main..my-feature" -d other-base

# Cherry-pick equivalent (duplicate + rebase)
jj duplicate -r abc123
jj rebase -d target-bookmark

# Squash series of changes
jj squash -r "main..HEAD"

# Reorder changes (implicit rebase on reorder)
jj log -r "main.."  # Shows order
jj rebase -r change1 -d change2  # Reorder

# Create worktree-equivalent workspace
jj workspace list
jj workspace add my-workspace
cd my-workspace
jj edit other-change
```

### 8. Undo and Recovery

jj's undo system is more powerful than Git's reflog.

```bash
# Undo last operation
jj undo

# Undo multiple steps
jj undo --revision @-3  # Undo 3 operations back

# View undo history
jj log --reversed

# Restore abandoned change
# Find in jj log --all-branches --all-bookmarks
jj edit abandoned-change

# Restore deleted bookmark
# Bookmarks are stored locally; create new bookmark at desired change
jj bookmark set -r abc123 recovered-bookmark
```

## Practical Workflows

### Workflow 1: Create Feature Branch and Push PR

```bash
# Start on main
jj edit main

# Make changes (automatic, no staging)
# ... edit files ...

# Describe work
jj describe -m "feat: add new feature

Implement feature X with full test coverage.
Refs: #123"

# Create local bookmark
jj bookmark set feat/new-feature

# Push to Git remote
jj git push -b feat/new-feature

# Create PR on GitHub (use gh command)
gh pr create --title "feat: add new feature" \
  --body "Implement feature X with full test coverage. Refs: #123"
```

### Workflow 2: Update Feature Branch from Main

```bash
# Fetch latest from remote
jj git fetch

# Rebase current change onto origin/main
jj rebase -d origin/main

# Push updated branch
jj git push -b feat/my-feature --force-with-lease
# (jj git push handles force-with-lease safely by default)
```

### Workflow 3: Squash Work-in-Progress into Clean Commits

```bash
# View current changes since main
jj log -r "main..@"

# Move to first change to squash
jj edit abc123

# Squash current into parent
jj squash

# Continue with next change
jj edit def456
jj squash

# Final result: clean linear history
```

### Workflow 4: Handle Merge Conflicts

```bash
# Rebase onto main (conflict occurs)
jj rebase -d origin/main

# jj shows conflict status
jj status

# Resolve in editor
# - Manually edit conflicted files
# - Remove conflict markers
# - Save

# Mark as resolved
jj resolve

# Continue with next conflicted change (if any)
jj resolve
```

### Workflow 5: Multi-Change Development

```bash
# Create first change
jj describe -m "feat: part 1"

# Move to main to start next change
jj new main -m "feat: part 2"

# Third feature
jj new main -m "fix: bug"

# View stack
jj log -r "main.."

# Reorder if needed
jj rebase -r feat-part2 -d feat-part1

# Push all
jj git push --all
```

### Workflow 6: Split Large Change

```bash
# Current change is too large
jj status
jj diff  # Shows all changes

# Split at decision point
jj split

# jj prompts interactively to choose which files go where
# Creates two changes from one

# Update descriptions
jj describe -m "First part of work"
jj edit @-1  # Go to first change
jj describe -m "Second part of work"
```

## Comparison: Git vs jj Operations

### Common Task: Create feature branch and commit work

**Git:**
```bash
git checkout -b feat/new-feature
# edit files
git add .
git commit -m "feat: add feature"
git push -u origin feat/new-feature
```

**jj:**
```bash
jj describe -m "feat: add feature"
jj bookmark set feat/new-feature
jj git push -b feat/new-feature
```

### Common Task: Amend last commit

**Git:**
```bash
# edit files
git add .
git commit --amend
git push --force-with-lease
```

**jj:**
```bash
# edit files
# (automatic tracking)
jj describe --message "updated message"  # Optional
jj git push --force-with-lease
```

### Common Task: Rebase onto main

**Git:**
```bash
git fetch origin
git rebase origin/main
git push --force-with-lease
```

**jj:**
```bash
jj git fetch
jj rebase -d origin/main
jj git push
```

### Common Task: Cherry-pick commit to another branch

**Git:**
```bash
git cherry-pick abc123
```

**jj:**
```bash
jj duplicate -r abc123
jj rebase -d target-bookmark
```

## Best Practices for CSLN Project

### 1. Conventional Commits with jj describe

Always use Conventional Commits format in jj descriptions:

```bash
jj describe -m "type(scope): subject

Detailed explanation of changes.
Keep body wrapped at 72 chars.

Refs: #123, csl26-abcd"
```

### 2. Working with Submodules

For the `styles-legacy/` submodule, continue using Git commands:

```bash
# Update submodule (still use git)
git submodule update --remote styles-legacy

# jj for everything else
jj describe -m "chore: update styles-legacy submodule"
jj git push
```

### 3. Pre-Flight Checks Before Push

Even with jj's safeguards, verify before pushing:

```bash
# Review what you're about to push
jj log -r "origin/main..@"

# Check conflicts
jj status

# Then push
jj git push
```

### 4. Bookmarks for Ongoing Work

Use bookmarks to track feature branches:

```bash
# Each feature gets a bookmark
jj bookmark set feat/parser-improvements
jj bookmark set fix/citation-rendering

# Push multiple
jj git push --all

# Later, return to feature
jj edit feat/parser-improvements
```

## Troubleshooting

### Conflict During Rebase

```bash
# jj shows conflicts in status
jj status

# Resolve manually (edit files)
# Then mark resolved
jj resolve
```

### Accidentally Abandoned Change

```bash
# View all changes (including abandoned)
jj log --all

# Find the change hash
# Create bookmark at it
jj bookmark set -r abc123 recovered-feature

# Continue work
jj edit recovered-feature
```

### Lost Work or Wrong Rebase

```bash
# Use undo (most powerful tool)
jj undo

# Can undo multiple steps
jj undo --revision @-5
```

### Sync with Remote After Force Push

```bash
# If remote was force-pushed (e.g., by CI)
jj git fetch
jj rebase -d origin/main
```

## Migration from Git

If transitioning an existing Git project to jj:

```bash
# Initialize jj in existing Git repo
jj init --git-repo .

# jj automatically reads Git history
jj log  # Shows all Git commits as jj changes

# Work normally with jj
# jj pushes to Git remote without issues
```

## Resources

- **Official jj Docs**: https://martinvonz.github.io/jj/
- **jj Comparisons**: https://martinvonz.github.io/jj/latest/git-comparison/
- **jj Configuration**: `.jj/config.toml` for custom aliases and settings
