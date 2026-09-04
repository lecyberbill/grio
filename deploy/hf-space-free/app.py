import subprocess
import sys
import os

print("🚀 Starting grio Declarative Engine on Hugging Face Spaces...")
sys.stdout.flush()

# Run the release build of the showcase
cmd = ["cargo", "run", "--release", "--example", "showcase"]
process = subprocess.Popen(cmd, stdout=sys.stdout, stderr=sys.stderr)
process.wait()
