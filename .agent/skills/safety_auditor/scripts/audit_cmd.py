import sys

def audit(command):
    danger_zones = {
        "rm -rf": "HIGH: Potential recursive file deletion.",
        "sudo": "MEDIUM: Execution with root privileges.",
        "> /dev/": "CRITICAL: Direct hardware/device write.",
        "curl | bash": "HIGH: Unsafe remote script execution."
    }
    
    found = [msg for pattern, msg in danger_zones.items() if pattern in command]
    
    if not found:
        return "Risk Level: LOW"
    return " | ".join(found)

if __name__ == "__main__":
    cmd = " ".join(sys.argv[1:])
    print(audit(cmd))