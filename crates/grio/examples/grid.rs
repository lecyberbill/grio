use grio::*;

fn main() -> grio::Result<()> {
    App::new("Grid & Layout Containers · grio demo")
        .subtitle("Demonstration of the Grid component, container nesting (Row, Column, Grid), and responsive alignments.")
        .panel("1. 3-Column Responsive Grid (App::grid)", |p| {
            p.grid(3, |g| {
                g.item(Text::new("c1").label("Column 1").value("Text A"));
                g.item(Text::new("c2").label("Column 2").value("Text B"));
                g.item(Text::new("c3").label("Column 3").value("Text C"));
                g.item(Slider::new("s1").label("Slider A").min(0.0).max(100.0).value(25.0));
                g.item(Slider::new("s2").label("Slider B").min(0.0).max(100.0).value(50.0));
                g.item(Slider::new("s3").label("Slider C").min(0.0).max(100.0).value(75.0));
            });
        })

        .panel("2. 2-Column Grid with Custom Gap Spacing", |p| {
            p.grid(2, |g| {
                g.gap(24.0);
                g.item(Output::new("out_left").label("Left Panel").value("Zone 1"));
                g.item(Output::new("out_right").label("Right Panel").value("Zone 2"));
            });
        })

        .panel("3. Nesting: Columns within a Row & Sub-grid", |p| {
            p.row(|r| {
                // Sub-column 1
                r.column(|col| {
                    col.scale(1);
                    col.item(Markdown::new("col1_desc").value("### Left Sub-column\nVertically organized."));
                    col.item(Text::new("user_input").label("Your Message").value("Hello World!"));
                    col.item(Button::new("send_btn").label("Compute"));
                });

                // Sub-column 2 containing a 2x2 nested subgrid
                r.column(|col| {
                    col.scale(2);
                    col.item(Markdown::new("col2_desc").value("### Right Sub-column (Nested 2×2 Grid)"));
                    col.grid(2, |subgrid| {
                        subgrid.item(Output::new("res_len").label("Length").value("0"));
                        subgrid.item(Output::new("res_upper").label("Uppercase").value("-"));
                        subgrid.item(Output::new("res_words").label("Words").value("0"));
                        subgrid.item(Output::new("res_echo").label("Echo").value("-"));
                    });
                });
            });
        })

        .on_click("send_btn", |ctx| {
            let msg: String = ctx.get("user_input").unwrap_or_default();
            let words = msg.split_whitespace().count();
            ctx.set("res_len", msg.chars().count().to_string());
            ctx.set("res_upper", msg.to_uppercase());
            ctx.set("res_words", words.to_string());
            ctx.set("res_echo", format!("Received: {}", msg));
            Ok(())
        })

        .launch("127.0.0.1:7860")
}
