import subprocess
import sys
import re

def run_tests(action, test_name=None):
    base_cmd = ["cargo", "test"]
    
    if action == "run_unit" and test_name:
        base_cmd.append(test_name)
    
    try:
        # We capture output to analyze it
        result = subprocess.run(base_cmd, capture_output=True, text=True)
        
        if result.returncode == 0:
            return f"✅ All tests passed!\n\n{result.stdout}"
        else:
            # Clean the output to keep only relevant failures
            summary = re.findall(r"FAILED\s+(.*)", result.stdout)
            return f"❌ Tests failed!\n\nFailures detected in: {summary}\n\nFull error:\n{result.stderr if result.stderr else result.stdout}"
            
    except Exception as e:
        return f"Error executing cargo test: {str(e)}"

if __name__ == "__main__":
    action_arg = sys.argv[1]
    name_arg = sys.argv[2] if len(sys.argv) > 2 else None
    print(run_tests(action_arg, name_arg))