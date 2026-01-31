# Multiple Languages Test

This file tests the ability of Compass to execute code in different programming languages.

## 1. Shell (Default)

Running a standard shell command.

```bash
echo "This is running in $(basename $SHELL)"
uname -a || ver
```

## 2. Python Script

This block uses the `python` language identifier. It should be executed by the Python interpreter.

```python
import sys
import platform

print(f"Hello from Python {platform.python_version()}")
print(f"Running on: {sys.platform}")

# Math test
a = 10
b = 32
print(f"The meaning of life is {a + b}")
```

## 3. Python with Loop

Testing indentation handling.

```python
print("Counting:")
for i in range(1, 6):
    print(f"  Item {i}")
print("Done!")
```
