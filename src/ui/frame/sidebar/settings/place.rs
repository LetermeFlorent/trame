use crate::geometry::Rect;
use crate::input::TextField;
use crate::text::TextSystem;
use crate::ui::frame::FrameLayout;
use crate::ui::frame::editor::place_field;
use crate::ui::metrics::UiMetrics;

use super::chips::Choice;
use super::column::{Column, LINE_HEIGHT, SECTION_GAP};
use super::form::{form_page, page_height};
use super::hosts::connections_page;
use super::instances::instances_page;
use super::shortcuts::shortcuts_page;
use super::{
    Chip, HostsView, SettingControl, SettingsFrame, SettingsPage, SettingsRequest, SettingsTab,
};

const SETTINGS_WIDTH: f32 = 600.0;

const NO_HOSTS: HostsView<'static> = HostsView {
    hosts: &[],
    first: 0,
    form: None,
    changing: false,
};

impl FrameLayout {
    pub fn place_settings(&mut self, text: &mut TextSystem, request: &SettingsRequest) {
        let tabs_height = tabs_height(&self.metrics);
        let page_height = page_height(&self.metrics, self.popup_room() - tabs_height);
        let size = [SETTINGS_WIDTH, page_height + tabs_height];
        let title = request.strings.settings;
        let (mut popup, mut column) = self.open_wide_popup(text, title, size);
        let tabs = settings_tabs(&mut column, request);
        column.y += column.metrics.px(SECTION_GAP);
        let page = settings_page(&mut column, request, page_height);
        popup.end_below(&self.metrics, column.y);
        self.settings = Some(SettingsFrame { popup, tabs, page });
    }

    pub fn place_setting_field(
        &mut self,
        text: &mut TextSystem,
        control: SettingControl,
        field: &TextField,
    ) {
        let metrics = self.metrics;
        let Some(SettingsPage::Form(page)) = self.settings.as_mut().map(|frame| &mut frame.page)
        else {
            return;
        };
        let spot = match control {
            SettingControl::HistoryLines => page.stepper(control).map(|stepper| {
                let line = stepper.line.rect;
                let edges = (stepper.smaller.x, stepper.bigger.right());
                (
                    line,
                    Rect::from_edges(edges.0, line.y, edges.1, line.bottom()),
                )
            }),
            _ => page.row(control).map(|row| {
                let line = row.line.rect;
                let anchor = Rect::from_edges(line.center().x, line.y, line.right(), line.bottom());
                (line, anchor)
            }),
        };
        let Some((line, anchor)) = spot else {
            return;
        };
        let frame = place_field(text, &metrics, anchor, line, field);
        page.field = Some((control, frame));
    }
}

fn tabs_height(metrics: &UiMetrics) -> f32 {
    metrics.px(LINE_HEIGHT) + metrics.px(SECTION_GAP)
}

fn settings_tabs(column: &mut Column, request: &SettingsRequest) -> [Chip; 7] {
    let strings = request.strings;
    let choice = |tab: SettingsTab| -> Choice {
        let label = match tab {
            SettingsTab::General => strings.general,
            SettingsTab::Terminals => strings.terminals,
            SettingsTab::Display => strings.display,
            SettingsTab::System => strings.system_section,
            SettingsTab::Instances => strings.instances_section,
            SettingsTab::Connections => strings.connections_tab,
            SettingsTab::Shortcuts => strings.keys.tab,
        };
        (label, SettingControl::Tab(tab), tab == request.tab)
    };
    column.tab_row(SettingsTab::ALL.map(choice))
}

fn settings_page(column: &mut Column, request: &SettingsRequest, height: f32) -> SettingsPage {
    let SettingsRequest {
        preferences,
        strings,
        folder,
        tab,
        ..
    } = *request;
    match tab {
        SettingsTab::Instances => {
            SettingsPage::Instances(Box::new(instances_page(column, strings, preferences)))
        }
        SettingsTab::Connections => {
            let view = request.hosts.unwrap_or(NO_HOSTS);
            SettingsPage::Connections(Box::new(connections_page(column, strings, view, height)))
        }
        SettingsTab::Shortcuts => {
            SettingsPage::Shortcuts(Box::new(shortcuts_page(column, request, height)))
        }
        _ => {
            let page = form_page(column, tab, strings, preferences, folder);
            SettingsPage::Form(Box::new(page))
        }
    }
}
