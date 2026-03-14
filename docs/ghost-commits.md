# Ghost Commits

## Summary

Ghost commits are **real detached Git commit objects** created with `git write-tree` and `git commit-tree`, then paired with app-level metadata that makes undo safe.

They are not:

- patches
- stashes
- temporary branches

They are:

1. a snapshot built from a **temporary Git index**
2. a **tree object** written from that temporary index
3. a **commit object** created from that tree
4. a commit created **without updating any refs**
5. a metadata record storing the commit SHA plus untracked-path preservation data

The architecture is simple:

- **Git stores the snapshot contents**
- **the application stores the pointer and cleanup instructions**

---

## Core Idea

The mechanism splits snapshotting into two layers.

### Layer 1: Git snapshot storage

The system creates a real Git commit object from the current repository state.

It does this by:

1. creating a temporary index with `GIT_INDEX_FILE`
2. optionally seeding that index from `HEAD` with `git read-tree`
3. staging snapshot content into that temporary index with `git add --all`
4. writing a tree object with `git write-tree`
5. creating a detached commit object with `git commit-tree`

Because `git commit-tree` is used directly, the result is a commit object in Git's object database, but no branch, `HEAD`, or named ref is updated.

### Layer 2: App-level undo metadata

The commit alone is not enough for safe undo.

The system also stores metadata containing:

- the ghost commit SHA
- the parent commit SHA, if one existed
- a list of untracked files that already existed before the snapshot
- a list of untracked directories that already existed before the snapshot

This metadata allows the system to restore tracked files from the commit while deleting only newly-created untracked junk.

---

## Data Model

The minimal metadata shape is:

```json
{
  "id": "<ghost commit sha>",
  "parent": "<optional parent sha>",
  "preexisting_untracked_files": ["path/to/file"],
  "preexisting_untracked_dirs": ["path/to/dir"]
}
```

### Field meanings

- `id`
  - the SHA of the detached commit object created for the snapshot

- `parent`
  - the current `HEAD` commit when the snapshot was created
  - may be `null` if the repository has no commit yet

- `preexisting_untracked_files`
  - untracked files that existed before snapshot creation
  - must not be deleted during undo cleanup

- `preexisting_untracked_dirs`
  - untracked directories that existed before snapshot creation
  - also used to preserve ignored or excluded directories from cleanup

---

## Creation Flow

### Step 1: Inspect repository state

Before creating the snapshot, the system captures repository status, including tracked changes and untracked content.

A typical command shape is:

```sh
git status --porcelain=2 -z --untracked-files=all
```

This status scan is used to determine:

- the current tracked state
- which untracked files and directories already exist
- which paths should be preserved during undo cleanup
- which large or ignored paths should be excluded from the snapshot payload

---

### Step 2: Create a temporary index

The snapshot is built in an isolated index, not the real live index.

That is done by creating a temp file and setting:

```sh
GIT_INDEX_FILE=/path/to/temp/index
```

This is a crucial design choice.

Using a temporary index means:

- the live staged state is not modified
- the snapshot process is isolated
- users do not get surprise index mutations

If you skip this, the implementation becomes invasive and error-prone.

---

### Step 3: Seed from `HEAD`

If the repository already has a current commit, the temporary index is initialized from it:

```sh
git read-tree <head-sha>
```

This provides the tracked baseline.

If there is no existing `HEAD`, the system can still create a ghost commit. In that case, the snapshot starts from an empty baseline.

---

### Step 4: Stage snapshot content into the temporary index

The working tree state is staged into the temporary index:

```sh
git add --all
```

Because `GIT_INDEX_FILE` is set, this affects only the temporary index.

---

### Step 5: Write the tree object

Once the temporary index contains the snapshot state, a tree object is created:

```sh
git write-tree
```

This returns a tree SHA representing the repository snapshot.

---

### Step 6: Create the detached commit object

The tree is then turned into a commit object:

```sh
git commit-tree <tree-sha> -p <parent-sha> -m "ghost snapshot"
```

If there is no parent, the `-p <parent-sha>` part is omitted.

This creates a real commit object, but does **not** update any refs.

That means:

- no branch moves
- `HEAD` does not move
- no stash entry is created
- no hidden branch is created

The result is just an object SHA.

---

## Where the Data Lives

## Git-level storage

The snapshot commit is stored in Git's object database.

In practice, that means it exists as a loose or packed object under `.git/objects`.

It is not pinned by a named ref.

That is a very important detail.

The object exists because Git created it, but unless some ref later points to it, it remains an **unreferenced commit object**.

---

## App-level storage

The application stores the metadata record separately in session or conversation history.

That persisted record contains:

- the commit SHA
- the parent SHA
- the preexisting untracked files
- the preexisting untracked directories

This history record is what makes later undo possible.

Without it, you would have a commit SHA but no safe way to decide which untracked files should be deleted.

---

## Why Both Storage Layers Are Required

Git is very good at restoring tracked content from a commit.

Git is not good at answering this question:

> Which untracked paths appeared after the snapshot and should now be deleted?

That is why the metadata layer exists.

It stores the state of untracked content at snapshot time so undo can distinguish:

- old untracked content that must be preserved
- new untracked content that should be deleted

This is the difference between a clean undo and a destructive one.

---

## Undo Flow

Undo works by combining the detached commit SHA with the stored metadata.

### Step 1: Find the latest ghost snapshot metadata

The application walks its stored history and finds the latest ghost snapshot record.

That gives it:

- the commit SHA to restore from
- the untracked preservation lists

---

### Step 2: Capture the current untracked state

Before restoring, the system scans current untracked files and directories.

This is necessary so it can compare the current state against the snapshot-time preservation metadata.

---

### Step 3: Restore tracked files from the detached commit

Tracked files are restored from the ghost commit with a command shape like:

```sh
git restore --source <ghost-commit-sha> --worktree -- .
```

The important choice here is that only the **working tree** is restored.

The staged/index state is intentionally left alone.

That means undo restores file contents while preserving unrelated staged changes.

---

### Step 4: Remove only newly-created untracked paths

After tracked files are restored, the system compares current untracked paths against the stored preservation metadata.

The rule is:

- if an untracked file existed before the snapshot, keep it
- if an untracked file lives inside a preserved directory, keep it
- if an untracked path appeared after the snapshot and is not preserved, delete it

This cleanup produces the effect users expect from undo without trashing unrelated preexisting files.

---

### Step 5: Consume the snapshot record

After a successful restore, the application removes the consumed ghost snapshot entry from history.

This makes the undo model act like a stack of snapshot points.

---

## Handling Large or Ignored Untracked Content

Not every untracked path should be included in the commit payload.

The implementation intentionally excludes certain content, especially:

- very large untracked files
- very large untracked directories
- default-ignored directories such as dependency folders

That content is typically expensive, noisy, or pointless to snapshot as hidden commit payload.

### Important nuance

Excluded paths are still preserved in metadata.

So the behavior is:

- **do not commit them**
- **do remember them for cleanup safety**

That means undo will not recreate their contents from the ghost commit, but it also will not accidentally delete them.

This is a deliberate compromise:

- smaller snapshot payloads
- safer cleanup behavior
- no attempt to secretly commit giant junk directories

---

## Why This Design Works

### 1. Temporary index isolation

Using `GIT_INDEX_FILE` avoids mutating the user's real staging area.

This keeps snapshot creation side-effect free from the user's point of view.

### 2. Detached commit fidelity

Using `git write-tree` plus `git commit-tree` gives a real Git snapshot with exact tree semantics.

That is cleaner and more reliable than inventing a custom patch format.

### 3. Metadata-enabled cleanup

The metadata layer solves the hard part: distinguishing old untracked files from new ones.

Without this, undo becomes either incomplete or destructive.

### 4. No visible branch pollution

Because no refs are updated, the mechanism leaves normal branch history untouched.

It gets snapshot behavior without littering the commit graph users see every day.

### 5. Staged changes are preserved

Restoring only the worktree means unrelated staged changes survive undo.

That is a very practical choice.

---

## Caveats

### Detached commits are not strongly pinned

Because no named ref points to the ghost commit, it is theoretically vulnerable to Git garbage collection.

If unreachable objects are pruned, the stored SHA may become invalid.

So this design is elegant, but not maximally durable.

If stronger retention is required, a hidden ref namespace could be added. But doing that would change the design.

### Excluded untracked content is preserved, not reconstructed

If a large or ignored untracked path was excluded from the commit payload, undo can preserve it from deletion, but it cannot rebuild it from the commit.

That is intentional.

### This is snapshot-based undo, not history rewriting

The mechanism restores a prior filesystem state. It is not trying to reverse individual edits semantically.

That simplicity is part of why it works.

---

## Minimal Reimplementation Spec

If you want to build something that behaves the same way, implement these rules:

1. Create snapshots as detached Git commits using `git commit-tree`
2. Build snapshots from a temporary index using `GIT_INDEX_FILE`
3. Do not update refs, branches, or `HEAD`
4. Persist the detached commit SHA in app/session history
5. Persist preexisting untracked file and directory lists alongside it
6. Restore tracked files using `git restore --source <sha> --worktree`
7. Preserve staged changes during undo
8. Delete only untracked paths created after the snapshot
9. Exclude giant or ignored untracked content from the commit payload when needed
10. Still preserve excluded paths in metadata so undo cleanup does not delete them

---

## Reference Pseudocode

### Create snapshot

```text
function createGhostSnapshot(repo):
    parent = resolveHeadIfPresent(repo)
    existingUntracked = captureUntracked(repo)

    tempIndex = createTempIndexFile()
    env.GIT_INDEX_FILE = tempIndex

    if parent exists:
        run("git read-tree <parent>")

    run("git add --all")
    tree = run("git write-tree")

    if parent exists:
        commit = run("git commit-tree <tree> -p <parent> -m 'ghost snapshot'")
    else:
        commit = run("git commit-tree <tree> -m 'ghost snapshot'")

    return {
        id: commit,
        parent: parent,
        preexisting_untracked_files: existingUntracked.files,
        preexisting_untracked_dirs: existingUntracked.dirs
    }
```

### Restore snapshot

```text
function restoreGhostSnapshot(repo, ghost):
    currentUntracked = captureUntracked(repo)

    run("git restore --source <ghost.id> --worktree -- .")

    for each path in currentUntracked:
        if path in ghost.preexisting_untracked_files:
            keep
        else if path is inside any ghost.preexisting_untracked_dirs:
            keep
        else:
            delete(path)

    removeGhostSnapshotRecordFromHistory()
```

---

## Final Takeaway

The correct mental model is:

> A ghost commit is a detached Git commit object created from a temporary index, paired with metadata that records which untracked paths already existed so undo can restore tracked files and safely remove only newly-created untracked content.

That combination is the whole mechanism.