# Smart Guide Test

This file tests the OS-specific filtering and conditional logic of Compass.

<!-- compass:if os="windows" -->
## Windows Only Section
This section should ONLY appear on Windows.
```powershell
echo "Hello from Windows!"
```
<!-- compass:endif -->

<!-- compass:if os="macos" -->
## MacOS Only Section
This section should ONLY appear on macOS.
```bash
echo "Hello from macOS!"
```
<!-- compass:endif -->

<!-- compass:if os="linux" -->
## Linux Only Section
This section should ONLY appear on Linux.
```bash
echo "Hello from Linux!"
```
<!-- compass:endif -->

## Common Section
This section appears everywhere.
```bash
echo "I am universal!"
```

## Error Recovery Test
Running a command that will prompt for a fix if it fails.

```bash
# Simulating a "bind: address already in use" error to trigger Smart Recovery
echo "Simulating failure..."
>&2 echo "bind: address already in use"
exit 1
```
