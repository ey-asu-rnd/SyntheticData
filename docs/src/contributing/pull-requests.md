# Pull Requests

Guide to submitting and reviewing pull requests.

## Before You Start

### 1. Check for Existing Work

- Search open issues for related discussions
- Check open PRs for similar changes
- Review the roadmap for planned features

### 2. Open an Issue First

For significant changes, open an issue to discuss:

- New features or major changes
- Breaking changes to public API
- Architectural changes
- Performance improvements

### 3. Create a Branch

```bash
# Sync with upstream
git checkout main
git pull origin main

# Create feature branch
git checkout -b feature/my-feature

# Or for bug fixes
git checkout -b fix/issue-123
```

## Making Changes

### 1. Write Code

Follow the [Code Style](code-style.md) guidelines:

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy

# Run tests
cargo test
```

### 2. Write Tests

Add tests for new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_feature_works() {
        // Test implementation
    }
}
```

### 3. Update Documentation

- Update relevant docs in `docs/src/`
- Add/update rustdoc comments
- Update CHANGELOG.md if applicable

### 4. Commit Changes

Write clear commit messages:

```bash
# Good commit messages
git commit -m "Add Benford's Law validation to amount generator"
git commit -m "Fix off-by-one error in batch generation"
git commit -m "Improve memory efficiency in large volume generation"

# Avoid vague messages
git commit -m "Fix bug"
git commit -m "Update code"
git commit -m "WIP"
```

### Commit Message Format

```
<type>: <short summary>

<optional detailed description>

<optional footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `refactor`: Code change without feature/fix
- `test`: Adding/updating tests
- `perf`: Performance improvement
- `chore`: Maintenance tasks

## Submitting a PR

### 1. Push Your Branch

```bash
git push -u origin feature/my-feature
```

### 2. Create Pull Request

Use the PR template:

```markdown
## Summary

Brief description of changes.

## Changes

- Added X feature
- Fixed Y bug
- Updated Z documentation

## Testing

- [ ] Added unit tests
- [ ] Added integration tests
- [ ] Ran full test suite
- [ ] Tested manually

## Checklist

- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests pass locally
- [ ] No new warnings from clippy
```

### 3. PR Title Format

```
<type>: <short description>
```

Examples:
- `feat: Add OCEL 2.0 export format`
- `fix: Correct decimal serialization in JSON output`
- `docs: Add process mining use case guide`

## Review Process

### Automated Checks

All PRs must pass:

| Check | Requirement |
|-------|-------------|
| Build | Compiles on all platforms |
| Tests | All tests pass |
| Formatting | `cargo fmt --check` passes |
| Linting | `cargo clippy` has no warnings |
| Documentation | Builds without errors |

### Code Review

Reviewers will check:

1. **Correctness**: Does the code do what it claims?
2. **Tests**: Are changes adequately tested?
3. **Style**: Does code follow conventions?
4. **Documentation**: Are changes documented?
5. **Performance**: Any performance implications?

### Responding to Feedback

- Address all comments
- Push fixes as new commits (don't force-push during review)
- Mark resolved conversations
- Ask for clarification if needed

## Merging

### Requirements

Before merging:

- All CI checks pass
- At least one approving review
- No unresolved conversations
- Branch is up to date with main

### Merge Strategy

We use **squash and merge** for most PRs:

- Combines all commits into one
- Keeps main history clean
- Preserves full history in PR

### After Merge

- Delete your feature branch
- Update local main:

```bash
git checkout main
git pull origin main
git branch -d feature/my-feature
```

## Special Cases

### Breaking Changes

For breaking changes:

1. Open an issue for discussion first
2. Document migration path
3. Update CHANGELOG with breaking change notice
4. Use `BREAKING CHANGE:` in commit footer

### Large PRs

For large changes:

1. Consider splitting into smaller PRs
2. Create a tracking issue
3. Use feature flags if needed
4. Provide detailed documentation

### Security Issues

For security vulnerabilities:

1. **Do not** open a public issue
2. Contact maintainers directly
3. Follow responsible disclosure

## PR Templates

### Feature PR

```markdown
## Summary

Adds [feature] to support [use case].

## Motivation

[Why is this needed?]

## Changes

- Added `NewType` struct in `synth-core`
- Implemented `NewGenerator` in `synth-generators`
- Added configuration options in `synth-config`
- Updated CLI to support new feature

## Testing

- Added unit tests for `NewType`
- Added integration tests for generation flow
- Manual testing with sample configs

## Documentation

- Added user guide section
- Updated configuration reference
- Added example configuration
```

### Bug Fix PR

```markdown
## Summary

Fixes #123 - [brief description]

## Root Cause

[What caused the bug?]

## Solution

[How does this fix it?]

## Testing

- Added regression test
- Verified fix with reproduction steps from issue
- Ran full test suite

## Checklist

- [ ] Regression test added
- [ ] Root cause documented
- [ ] Related issues linked
```

## See Also

- [Development Setup](development-setup.md) - Environment setup
- [Code Style](code-style.md) - Coding standards
- [Testing](testing.md) - Testing guidelines
