use gooey::value::{Dynamic, Switchable};
use gooey::widget::{MakeWidget, WidgetInstance};
use gooey::Run;

#[derive(Debug)]
enum ActiveContent {
    Intro,
    Success,
}

fn main() -> gooey::Result {
    let active = Dynamic::new(ActiveContent::Intro);

    active
        .switcher(|current, active| match current {
            ActiveContent::Intro => intro(active.clone()),
            ActiveContent::Success => success(active.clone()),
        })
        .contain()
        .centered()
        .expand()
        .run()
}

fn intro(active: Dynamic<ActiveContent>) -> WidgetInstance {
    const INTRO: &str = "This example demonstrates the Switcher<T> widget, which uses a mapping function to convert from a generic type to the widget it uses for its contents.";
    INTRO
        .and(
            "Switch!"
                .into_button()
                .on_click(move |_| active.set(ActiveContent::Success))
                .centered(),
        )
        .into_rows()
        .make_widget()
}

fn success(active: Dynamic<ActiveContent>) -> WidgetInstance {
    "The value changed to `ActiveContent::Success`!"
        .and(
            "Start Over"
                .into_button()
                .on_click(move |_| active.set(ActiveContent::Intro))
                .centered(),
        )
        .into_rows()
        .make_widget()
}