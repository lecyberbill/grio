import subprocess
import os
import sys
import threading
import time

# Satisfy Hugging Face ZeroGPU watchdog if ZeroGPU is assigned
try:
    import spaces
    @spaces.GPU(duration=10)
    def init_gpu():
        return "GPU initialized"
    init_gpu()
except Exception as e:
    pass

print("🚀 Starting grio Declarative Engine on Hugging Face Spaces...")
sys.stdout.flush()

cargo_bin = os.path.expanduser("~/.cargo/bin/cargo")

# Install modern Rust if not present
if not os.path.exists(cargo_bin):
    print("📦 Installing latest stable Rust toolchain via rustup...")
    sys.stdout.flush()
    subprocess.run("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y", shell=True, check=True)

# Add cargo to PATH
os.environ["PATH"] = f"{os.path.expanduser('~/.cargo/bin')}:{os.environ.get('PATH', '')}"

print("⚡ Running grio showcase...")
sys.stdout.flush()

# Run the showcase with modern cargo
process = subprocess.Popen([cargo_bin, "run", "--release", "--example", "showcase"])
process.wait()
