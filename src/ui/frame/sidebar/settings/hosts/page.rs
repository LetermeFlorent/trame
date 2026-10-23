use crate::geometry::{Point, Rect};
use crate::locale::Strings;
use crate::preferences::ConnectForm;
use crate::ui::frame::editor::{
    FormAction, FormFrame, FormSpec, form_height, form_title, place_form,
};
use crate::ui::frame::sidebar::settings::column::{Column, LINE_HEIGHT};
use crate::ui::frame::sidebar::settings::{ChoiceRow, SettingControl, SettingLine};

use super::{HostList, HostsView, ListSpec, host_labels};

const FORM_ACTIONS: [FormAction; 2] = [FormAction::Cancel, FormAction::Save];

#[derive(Clone, Debug)]
pub struct ConnectionsPage {
    pub header: ChoiceRow,
    pub list: Option<HostList>,
    pub hint: Option<SettingLine>,
    pub form: Option<FormFrame>,
}

impl ConnectionsPage {
    pub fn control_at(&self, point: Point) -> Option<SettingControl> {
        self.header
            .controls()
            .find(|(rect, _)| rect.contains(point))
            .map(|(_, control)| control)
    }
}

pub(in crate::ui::frame::sidebar::settings) fn connections_page(
    column: &mut Column,
    strings: &'static Strings,
    view: HostsView,
    height: f32,
) -> ConnectionsPage {
    let bottom = column.y + height;
    match view.form {
        Some(form) => form_page(column, strings, form, view.changing),
        None => list_page(column, strings, view, bottom),
    }
}

fn form_page(
    column: &mut Column,
    strings: &'static Strings,
    form: &ConnectForm,
    changing: bool,
) -> ConnectionsPage {
    let caption = column.caption(form_title(strings, changing));
    let header = column.choices_on(caption, &[], None);
    let height = form_height(&column.metrics);
    let area = Rect::new(column.inner.x, column.y, column.inner.width, height);
    let spec = FormSpec {
        area,
        form,
        actions: &FORM_ACTIONS,
        strings,
    };
    let form = place_form(column.text, &column.metrics, &spec);
    column.y = area.bottom();
    ConnectionsPage {
        header,
        list: None,
        hint: None,
        form: Some(form),
    }
}

fn list_page(
    column: &mut Column,
    strings: &'static Strings,
    view: HostsView,
    bottom: f32,
) -> ConnectionsPage {
    let caption = column.caption(strings.connections);
    let add = (strings.add, SettingControl::AddHost, false);
    let header = column.choices_on(caption, &[add], None);
    let mut page = ConnectionsPage {
        header,
        list: None,
        hint: None,
        form: None,
    };
    if view.hosts.is_empty() {
        let hint = column.line(strings.no_connections, LINE_HEIGHT, column.metrics.label());
        page.hint = Some(hint);
        return page;
    }
    let room = (bottom - column.y) / column.metrics.px(LINE_HEIGHT);
    let labels = host_labels(view.hosts);
    let spec = ListSpec {
        labels: &labels,
        first: view.first,
        shown: room.floor().max(1.0) as usize,
        pickable: false,
        forgets: true,
    };
    page.list = Some(column.host_list(&spec));
    page
}
