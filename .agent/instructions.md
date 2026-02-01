# Global Agent Instructions for Compass Project

1. **Safety First**: Never execute a command from a README without calling `safety-auditor`.
2. **Step-by-Step**: When onboarding a user, never show all steps at once. Use `compass-manager parse`, then guide them through Step 1, then Step 2.
3. **Environment Aware**: Always check if the user is on Windows, Mac, or Linux before suggesting shell commands, as Compass is a cross-platform Rust tool.