---
pre_run: "echo '[HOOK] Pre-run hook executed: Setting up environment...' && mkdir -p hook_demo_output"
post_run: "echo '[HOOK] Post-run hook executed: Cleaning up...' && rmdir hook_demo_output"
on_failure: "echo '[HOOK] Failure detected! Check the logs.'"
---

# 🎣 Hooks & Automation Demo

This runbook demonstrates how Compass handles lifecycle hooks defined in the frontmatter.

## 1. Verify Pre-run
The `pre_run` hook should have created a directory named `hook_demo_output`. Let's check if it exists.

```bash
ls -d hook_demo_output
```

## 2. Simulate Work
We will create a file inside that directory.

```bash
echo "Work in progress" > hook_demo_output/status.txt
cat hook_demo_output/status.txt
```

## 3. Simulate Failure (Optional)
If you want to test the `on_failure` hook, run the command below. It is guaranteed to fail.

```bash
exit 1
```

## 4. Finish
If you finish successfully, the `post_run` hook will cleanup the directory.
