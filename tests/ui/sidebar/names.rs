use trame::locale::{Language, Strings};
use trame::session::SessionId;
use trame::text::TextSystem;
use trame::ui::{FrameLayout, UiMetrics};
use trame::workspace::Workspace;

use crate::{input, text_system};

const PHRASE: &str = "Terminal {number}";
const SHOWN: &str = "Terminal 1";

fn phrased() -> &'static Strings {
    Box::leak(Box::new(Strings {
        instance_name: PHRASE,
        ..*Language::English.strings()
    }))
}

fn unnamed(sidebar_open: bool) -> Workspace {
    let mut workspace = Workspace::new();
    workspace.add_instance(None, SessionId(1));
    workspace.sidebar_open = sidebar_open;
    workspace
}

fn laid_out(text: &mut TextSystem, workspace: &Workspace) -> FrameLayout {
    let mut input = input(workspace, UiMetrics::new(1.0));
    input.strings = phrased();
    FrameLayout::compute(text, &input)
}

#[test]
fn an_unnamed_instance_is_named_from_the_phrase_of_the_language() {
    let mut text = text_system();
    let open = laid_out(&mut text, &unnamed(true));
    let expected = text.layout(SHOWN, open.metrics.strong());
    let row = &open.sidebar.as_ref().expect("the sidebar").rows[0];
    assert_eq!(row.label.width, expected.width);
    let (title, _) = open.title_bar.title.as_ref().expect("the instance name");
    assert_eq!(title.width, expected.width);
    let closed = laid_out(&mut text, &unnamed(false));
    assert_eq!(closed.rail.as_ref().expect("the rail").slots[0].name, SHOWN);
}
