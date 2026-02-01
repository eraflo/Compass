---
name: commit-helper
description: Helps the user write Conventional Commits messages for Semantic Release.
input_schema:
  type: object
  properties:
    changes:
      type: string
      description: "Description of what you just coded."
---

# Commit Helper

## Instructions
- Analyze the user's changes.
- Suggest a commit message following the format: `<type>(<scope>): <description>`.
- Types: feat, fix, docs, style, refactor, perf, test, chore.

## Examples
**User:** "I just finished the parser logic."
**Agent:** "I suggest: `feat(parser): implement markdown structural parsing`"