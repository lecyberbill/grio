use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use grio::*;

static SECRET_VISIBLE: AtomicBool = AtomicBool::new(true);

/// Reads a Text field as a float (`a`, `b` are strings).
fn num(ctx: &Context, id: &str) -> grio::Result<f64> {
    ctx.get_str(id)
        .unwrap_or("")
        .trim()
        .parse()
        .map_err(|_| format!("`{id}` is not a number").into())
}

fn main() -> grio::Result<()> {
    App::new("Blocks · grio demo")
        .subtitle("Phase 1 — declared flows, .then/.success/.failure chaining, ctx.event, skip/set_prop, interactivity, load, multi-triggers, tabs & accordion.")
        .run_label("Submit")

        // 1.6 — App::on_load: triggered when page mounts (WS connection).
        .on_load(|ctx| {
            ctx.set("load_note", "Page mounted — `load` event received by server.");
            Ok(())
        })
        .panel("1.6 · on_load (page lifecycle)", |p| {
            p.item(Output::new("load_note").label("Status"));
        })

        // 1.1 — Declared flows: handlers only read `a`,`b` and write `cmp_out` (.flow).
        .row(|r| {
            r.item(Text::new("a").label("a").value("1"));
            r.item(Text::new("b").label("b").value("2"));
        })
        .row(|r| {
            r.item(Button::new("cmp_gt").label("a > b ?"));
            r.item(Button::new("cmp_lt").label("b > a ?"));
            r.item(Output::new("cmp_out").label("Result"));
        })
        .on_click("cmp_gt", |ctx| {
            let a: f64 = num(ctx, "a")?;
            let b: f64 = num(ctx, "b")?;
            ctx.set("cmp_out", if a > b { "a > b ✓" } else { "a ≤ b" });
            Ok(())
        })
        .flow(&["a", "b"], &["cmp_out"])
        .on_click("cmp_lt", |ctx| {
            // 1.3 — ctx.event() exposes target (`c`), action (`e`) and data (`d`).
            let target = ctx.event().map(|e| e.c.as_str()).unwrap_or("?");
            let a: f64 = num(ctx, "a")?;
            let b: f64 = num(ctx, "b")?;
            ctx.set("cmp_out", format!("(from `{target}`) {}", if b > a { "b > a ✓" } else { "b ≤ a" }));
            Ok(())
        })
        .flow(&["a", "b"], &["cmp_out"])

        // 1.7 — Multi-triggers: attach same function across multiple buttons.
        .row(|r| {
            r.item(Button::new("opt_a").label("Option A"));
            r.item(Button::new("opt_b").label("Option B"));
            r.item(Output::new("multi_out").label("Last Click"));
        })
        .on("click", ["opt_a", "opt_b"], |ctx| {
            let t = ctx.event().map(|e| e.c.as_str()).unwrap_or("?");
            ctx.set("multi_out", format!("Last click: `{t}`"));
            Ok(())
        })

        // 1.5 — Explicit interactivity: non-editable disabled fields.
        .row(|r| {
            r.item(Text::new("ro").label("Read Only").value("9").interactive(false));
            r.item(Output::new("ro_note").label("Note").value("disabled (interactive = false)"));
            r.item(Slider::new("ro_slider").label("Locked Slider").min(0.0).max(10.0).step(1.0).value(5.0).interactive(false));
        })

        // 1.4 — skip + set_prop: ignore an output, toggle visibility.
        .row(|r| {
            r.item(Button::new("skip_btn").label("set + skip ⇒ out_b frozen"));
            r.item(Output::new("out_a").label("out_a"));
            r.item(Output::new("out_b").label("out_b").value("initial value"));
        })
        .on_click("skip_btn", |ctx| {
            ctx.skip("out_b");
            ctx.set("out_a", "updated A ✓");
            ctx.set("out_b", "MUST NOT be displayed");
            ctx.alert(AlertLevel::Info, "out_b skipped → keeps its initial value");
            Ok(())
        })
        .row(|r| {
            r.item(Button::new("toggle_btn").label("Hide Secret"));
            r.item(Output::new("secret").label("Secret Content").value("*** CONFIDENTIAL PAYLOAD ***"));
        })
        .on_click("toggle_btn", |ctx| {
            let prev = SECRET_VISIBLE.fetch_xor(true, Ordering::SeqCst);
            ctx.set_prop("secret", "visible", !prev);
            ctx.set_prop(
                "toggle_btn",
                "label",
                if !prev { "Show Secret" } else { "Hide Secret" },
            );
            ctx.alert(AlertLevel::Success, if prev { "content hidden" } else { "content revealed" });
            Ok(())
        })

        // 1.2 — Termination chaining: .success / .failure on fallible handlers.
        .add(Output::new("err_out").label("err_out"))
        .add(Output::new("err_log").label("err_log"))
        .add(Button::new("err_btn").label("handler: error if `a` is empty"))
        .on_click("err_btn", |ctx| {
            let a = ctx.get_str("a").unwrap_or("").trim();
            if a.is_empty() {
                return Err("field `a` is empty".into());
            }
            ctx.set("err_out", format!("ok, a = {a}"));
            Ok(())
        })
        .success(|ctx| {
            ctx.set("err_log", "handler succeeded → .success executed");
            Ok(())
        })
        .failure(|ctx| {
            ctx.alert(AlertLevel::Warn, "Failure caught by .failure (recovery)");
            ctx.set("err_log", "failure caught → .failure executed and handled");
            Ok(())
        })

        // 1.8 — Containers: Tabs + Accordion.
        .item(
            Tabs::new("tabs")
                .tab("Compare & Multi", |t| {
                    t.item(Markdown::new("txt_1").text("### Tab 1\n\nDeclared **flows** (1.1), `ctx.event` (1.3), and **multi-triggers** (1.7) are active above."));
                })
                .tab("Chatbot (1.2)", |t| {
                    t.item(Text::new("chat_in").label("Message").placeholder("Type and click Submit…"));
                    t.item(Output::new("chat_out").label("Conversation"));
                    t.item(Markdown::new("txt_2").text("#### Chaining\n`on_submit(user).then(bot)`: the bot answers **after** the user, word by word (streaming)."));
                }),
        )
        .item(
            Accordion::new("acc")
                .open(true)
                .section("Chatbot Overview", |s| {
                    s.item(Markdown::new("acc_1").text("**Streaming** responses via `ctx.append`, progress and alerts — see `examples/greet.rs`."));
                })
                .section("Dynamic Props (1.4)", |s| {
                    s.item(Markdown::new("acc_2").text("`ctx.set_prop(\"id\", \"visible\"|\"label\"|…)` updates widget configuration dynamically without losing its state."));
                }),
        )

        // 1.2 — Chaining: Chatbot (user_fn.then(bot_fn)).
        .on_submit(|ctx| {
            let msg: String = ctx.get("chat_in")?;
            if msg.trim().is_empty() {
                return Err("message is empty".into());
            }
            ctx.append("chat_out", format!("You: {msg}\n"));
            Ok(())
        })
        .then(|ctx| {
            ctx.append("chat_out", "Bot: ");
            for _i in 1..=6 {
                if ctx.cancelled() {
                    ctx.append("chat_out", "\n[cancelled]\n");
                    return Ok(());
                }
                ctx.append("chat_out", "*");
                thread::sleep(Duration::from_millis(120));
            }
            ctx.append("chat_out", " (simulated response)\n");
            ctx.alert(AlertLevel::Success, "chatbot complete");
            Ok(())
        })

        .launch("127.0.0.1:7860")?;
    Ok(())
}
