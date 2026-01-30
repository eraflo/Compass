import sys
import os

def generate(case_type):
    output_dir = "tests/fixtures"
    os.makedirs(output_dir, exist_ok=True)
    file_path = os.path.join(output_dir, f"test_{case_type}.md")
    
    templates = {
        "nested": "# Nested Test\n```bash\n# Parent\n```python\nprint('child')\n```\n```",
        "huge": "# Performance Test\n" + "## Step\n```bash\necho ok\n```\n" * 50,
        "malformed": "## No Closing Code Block\n```bash\necho 'oops'"
    }

    content = templates.get(case_type, "# Default\n```bash\nls\n```")
    
    with open(file_path, "w") as f:
        f.write(content)
    return f"Created test fixture at: {file_path}"

if __name__ == "__main__":
    print(generate(sys.argv[1] if len(sys.argv) > 1 else "default"))