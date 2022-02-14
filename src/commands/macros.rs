macro_rules! make_range_select_menu {
    ($sm:expr, $f: expr, $l: expr) => {{
        $sm.custom_id("range")
            .placeholder("Page No")
            .options(|mut os| {
                for x in $f..=$l {
                    os = os.create_option(|o| o.label(x).value(x));
                }

                os
            })
    }};
}

macro_rules! make_terminal_components {
    ($c: expr, $terminal: expr, $pages: expr ) => {{
        $c.create_action_row(|ar| {
            ar.create_button(|b| {
                b.style(ButtonStyle::Primary)
                    .label("First")
                    .emoji(ReactionType::Unicode("\u{23EA}".to_string()))
                    .custom_id("first")
                    .disabled($terminal == "first")
            })
            .create_button(|b| {
                b.style(ButtonStyle::Primary)
                    .label("Prev")
                    .emoji(ReactionType::Unicode("\u{25C0}".to_string()))
                    .custom_id("prev")
                    .disabled($terminal == "first")
            })
            .create_button(|b| {
                b.style(ButtonStyle::Primary)
                    .label("Next")
                    .emoji(ReactionType::Unicode("\u{25B6}".to_string()))
                    .custom_id("next")
                    .disabled($terminal == "last")
            })
            .create_button(|b| {
                b.style(ButtonStyle::Primary)
                    .label("Last")
                    .emoji(ReactionType::Unicode("\u{23E9}".to_string()))
                    .custom_id("last")
                    .disabled($terminal == "last")
            })
            .create_button(|b| {
                b.style(ButtonStyle::Danger)
                    .label("Delete")
                    .emoji(ReactionType::Unicode("\u{1F5D1}".to_string()))
                    .custom_id("delete")
            })
        })
        .create_action_row(|ar| ar.create_select_menu(|sm| make_range_select_menu!(sm, 1, $pages)))
    }};
}
pub(crate) use make_range_select_menu;
pub(crate) use make_terminal_components;
