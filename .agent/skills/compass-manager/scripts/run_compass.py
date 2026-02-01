import subprocess
import sys

def main():
    if len(sys.argv) < 3:
        print("Usage: run_compass.py <action> <file_path>")
        sys.exit(1)

    action = sys.argv[1] # parse, check, or execute
    file_path = sys.argv[2]
    
    # Run the compiled rust binary via cargo
    cmd = ["cargo", "run", "--", action, file_path]
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        print(result.stdout)
    except subprocess.CalledProcessError as e:
        print(f"Compass Error: {e.stderr}")

if __name__ == "__main__":
    main()