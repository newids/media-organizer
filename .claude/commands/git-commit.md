# Git Commit Automatically

Analyze code changes and execute git commit with the following steps: $ARGUMENTS

## Commit Message Generation Steps

1. Check changes with `git status`
2. Analyze the content of modified files
3. Identify the nature of changes (feature addition, bug fix, refactoring, etc.)
4. Generate commit message following Conventional Commits rules
5. Execute `git add . && git commit -m "commit message"`
6. Notify completion

## Commit Message Examples

- feat: implement user authentication system
- fix: resolve memory leak in data processing
- docs: update API documentation  
- style: format code according to eslint rules

## Instructions for Claude Code

When this command is invoked:

1. **Analyze Changes**: Use `git status` and `git diff` to understand what files have been modified
2. **Review Recent Commits**: Check `git log --oneline -n 3` to understand the commit message style
3. **Generate Appropriate Message**: Create a commit message that:
   - Follows Conventional Commits format (type: description)
   - Accurately describes the changes made
   - Includes technical details when relevant
   - Ends with Claude Code attribution
4. **Execute Commit**: Run `git add .` followed by `git commit` with the generated message
5. **Confirm Success**: Verify the commit completed successfully

The commit message should follow this format:
```
type: brief description

- Detailed bullet points of changes
- Technical implementation details  
- Any important notes or context

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>
```