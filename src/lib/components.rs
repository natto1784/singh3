use serenity::{
    builder::{CreateComponents, CreateSelectMenu},
    model::{channel::ReactionType, interactions::message_component::ButtonStyle},
};

pub fn make_range_select_menu(first: usize, last: usize) -> CreateSelectMenu {
    let mut sm = CreateSelectMenu::default();
    sm.custom_id("range")
        .placeholder("Page No")
        .options(|mut os| {
            for x in first..=last {
                os = os.create_option(|o| o.label(x).value(x));
            }

            os
        });
    sm
}

pub fn make_terminal_components(terminal: &str, pages: usize) -> CreateComponents {
    let mut c = CreateComponents::default();
    c.create_action_row(|ar| {
        ar.create_button(|b| {
            b.style(ButtonStyle::Primary)
                .label("First")
                .emoji(ReactionType::Unicode("\u{23EA}".to_string()))
                .custom_id("first")
                .disabled(terminal == "first")
        })
        .create_button(|b| {
            b.style(ButtonStyle::Primary)
                .label("Prev")
                .emoji(ReactionType::Unicode("\u{25C0}".to_string()))
                .custom_id("prev")
                .disabled(terminal == "first")
        })
        .create_button(|b| {
            b.style(ButtonStyle::Primary)
                .label("Next")
                .emoji(ReactionType::Unicode("\u{25B6}".to_string()))
                .custom_id("next")
                .disabled(terminal == "last")
        })
        .create_button(|b| {
            b.style(ButtonStyle::Primary)
                .label("Last")
                .emoji(ReactionType::Unicode("\u{23E9}".to_string()))
                .custom_id("last")
                .disabled(terminal == "last")
        })
        .create_button(|b| {
            b.style(ButtonStyle::Danger)
                .label("Delete")
                .emoji(ReactionType::Unicode("\u{1F5D1}".to_string()))
                .custom_id("delete")
        })
    })
    .create_action_row(|ar| ar.add_select_menu(make_range_select_menu(1, pages)));
    c
}
