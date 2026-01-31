# Multiple Languages Test

This file tests the ability of Compass to execute code in different programming languages.

## 1. Shell (Default)

Running a standard shell command.

```bash
echo "Hello from Shell!"
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

## 4. JavaScript (Node.js)

Executing JavaScript using Node.js.

```javascript
console.log(`Hello from Node.js ${process.version}`);
console.log("Current directory:", process.cwd());
```

## 5. TypeScript

Executing TypeScript (requires `ts-node`).

```typescript
const greeting: string = "Hello from TypeScript";
const version: string = "v1.0";

console.log(`${greeting} ${version}`);
```

## 6. Go (Golang)

Executing Go code (compiles and runs).

```go
package main

import (
	"fmt"
	"runtime"
)

func main() {
	fmt.Printf("Hello from Go running on %s\n", runtime.GOOS)
}
```

## 7. Rust

Executing Rust code (compiles and runs via `rustc`).

```rust
fn main() {
    println!("Hello from Rust!");
    
    let x = 40;
    let y = 2;
    println!("40 + 2 = {}", x + y);
}
```

## 8. PHP

Executing PHP script.

```php
<?php
echo "Hello from PHP " . phpversion() . "\n";
echo "Operating System: " . PHP_OS . "\n";
?>
```

## 9. Ruby

Executing Ruby script.

```ruby
puts "Hello from Ruby #{RUBY_VERSION}"
puts "Platform: #{RUBY_PLATFORM}"
```

## 10. C# (.NET)

Executing C# code (creates a temporary .NET project).

```csharp
Console.WriteLine($"Hello from .NET {Environment.Version}");
Console.WriteLine($"OS: {Environment.OSVersion}");
```
