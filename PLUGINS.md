# 🛡️ grio WebAssembly Plugin Engine & Extensible ABI Guide

This guide details the **WebAssembly Plugin ABI specification** for `grio`, resource sandboxing, and how to write, compile, and integrate third-party plugins without recompiling the host `grio` server.

---

## 1. Philosophy & Security Model

Third-party plugins execute in a **strictly isolated sandbox**:
- **Linear Memory Isolation**: Dedicated linear memory space with strict runtime bounds checking.
- **Resource Limits (`SandboxLimits`)**:
  - `max_memory_pages`: Configurable upper memory bound (e.g. 128 pages = 8 MB; 1 page = 64 KB).
  - `max_fuel`: Instruction count / fuel metering to protect against infinite loops and CPU starvation.
  - `timeout_ms`: Automatic task cancellation on execution deadline exceeded.
- **Zero Host Leakage**: No direct access to host filesystem, raw sockets, or host environment variables.

---

## 2. Universal Extensible ABI Protocol

To allow third-party plugins to expose **arbitrary and unforeseen features** at runtime, `grio` relies on a message-passing ABI over WebAssembly memory:

### Exported Plugin Entry Points:
1. `alloc(len: u32) -> u32`: Allocates a buffer in the sandbox to receive input payloads from the host.
2. `dealloc(ptr: u32, len: u32)`: Frees the allocated memory block.
3. `grio_describe() -> (ptr, len)`: Returns the plugin manifest as JSON (name, version, capabilities).
4. `grio_invoke(method_ptr, method_len, payload_ptr, payload_len) -> (out_ptr, out_len)`:
   - Dispatches any arbitrary method name (e.g., `"filter"`, `"tokenize"`, `"predict"`, `"custom_transform"`).
   - `payload` supports universal JSON (`serde_json::Value`) or raw binary buffers (Apache Arrow, f32 arrays).

---

## 3. Example: Writing a Plugin in Rust (`wasm32-unknown-unknown`)

Here is the source code for an autonomous third-party plugin crate (e.g. `my_wasm_filter/src/lib.rs`):

```rust
// Compile with: cargo build --target wasm32-unknown-unknown --release

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Deserialize)]
struct FilterInput {
    text: String,
    level: Option<String>,
}

#[derive(Serialize)]
struct FilterOutput {
    filtered_text: String,
    flagged: bool,
    tokens: usize,
}

#[no_mangle]
pub extern "C" fn grio_invoke(
    method_ptr: *const u8,
    method_len: usize,
    data_ptr: *const u8,
    data_len: usize,
) -> u64 {
    let method = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(method_ptr, method_len)) };
    let input_bytes = unsafe { std::slice::from_raw_parts(data_ptr, data_len) };

    let output_bytes = match method {
        "censor" => {
            let input: FilterInput = serde_json::from_slice(input_bytes).unwrap_or(FilterInput {
                text: String::new(),
                level: None,
            });
            let is_bad = input.text.contains("bad");
            let clean = input.text.replace("bad", "***");
            let out = FilterOutput {
                filtered_text: clean,
                flagged: is_bad,
                tokens: input.text.split_whitespace().count(),
            };
            serde_json::to_vec(&out).unwrap()
        }
        "custom_method" => {
            // Any arbitrary, future, unforeseen method
            serde_json::to_vec(&json!({ "status": "executed", "dynamic_feature": 42 })).unwrap()
        }
        _ => Vec::new(),
    };

    // Pack pointer and length into a u64
    let ptr = output_bytes.as_ptr() as u32;
    let len = output_bytes.len() as u32;
    std::mem::forget(output_bytes); // Keep memory alive until host reads it
    ((ptr as u64) << 32) | (len as u64)
}
```

---

## 4. Usage in a `grio` Host Application

```rust
use grio::*;
use serde_json::json;

fn main() -> Result<()> {
    // 1. Instantiate the plugin with sandboxing limits
    let plugin = WasmPlugin::from_file("./plugins/moderator.wasm")?
        .limits(SandboxLimits {
            max_memory_pages: 128, // 8 MB
            max_fuel: 10_000_000,
            timeout_ms: 1000,
        });

    // 2. Register the plugin in the App builder
    App::new("Sandboxed Plugin App")
        .wasm_plugin("moderator", plugin)
        .item(Text::new("input_msg").label("User Text"))
        .item(Output::new("clean_msg").label("Filtered Output"))
        .on_submit(|ctx| {
            let text: String = ctx.get("input_msg")?;
            
            // 3. Universal sandboxed execution
            let result = ctx.call_wasm(
                "moderator",
                "censor",
                &json!({ "text": text, "level": "strict" })
            )?;

            ctx.set("clean_msg", result["filtered_text"].as_str().unwrap_or(""));
            Ok(())
        })
        .launch("127.0.0.1:7860")
}
```
