mod color;
mod components;
mod state;

use egui::{Color32, Response, TextStyle, TextWrapMode, Widget};

use self::color::ToColor32;
use self::components::level_menu_button::LevelMenuButton;
use self::components::target_menu_button::TargetMenuButton;
use self::state::LogsState;
use crate::time::DateTimeFormatExt;
use crate::tracing::collector::EventCollector;

#[derive(Debug, Clone)]
pub struct Logs {
    collector: EventCollector,
    glob: Option<globset::GlobSet>,
    logs_state: LogsState,
}

impl Logs {
    #[must_use]
    pub const fn new(collector: EventCollector) -> Self {
        Self { collector, glob: None, logs_state: LogsState::DEFAULT }
    }
}

impl Widget for Logs {
    fn ui(mut self, ui: &mut egui::Ui) -> Response {
        Widget::ui(&mut self, ui)
    }
}
impl Widget for &mut Logs {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let glob = self.glob.get_or_insert_with(||{
            let mut glob = globset::GlobSetBuilder::new();
            for target in self.logs_state.target_filter.targets.clone() {
                glob.add(target);
            }
            glob.build().unwrap()
        });

        let events = self.collector.events();
        let filtered_events = events
            .iter()
            .rev()
            .filter(|event| self.logs_state.level_filter.get(event.level) && !glob.is_match(&event.target))
            .collect::<Vec<_>>();

        let row_height = ui.style().text_styles.get(&TextStyle::Small).unwrap().size;

        ui.allocate_ui(egui::Vec2::new(100.+80.+100.+120.+120., (2.*row_height) * (filtered_events.len() as f32)), |ui|{
            egui_extras::TableBuilder::new(ui)
                .column(egui_extras::Column::initial(100.).resizable(true))
                .column(egui_extras::Column::initial(80.).resizable(true))
                .columns(egui_extras::Column::initial(120.).resizable(true),2)
                .striped(true)
                .vscroll(true)
                .header(row_height, |mut row|{
                    row.col(|ui|{
                        ui.set_min_width(100.);
                        ui.label("Time");
                    });
                    row.col(|ui|{
                        ui.set_min_width(80.);
                        LevelMenuButton::default()
                            .state(&mut self.logs_state.level_filter)
                            .show(ui)
                    });
                    row.col(|ui|{
                        ui.set_min_width(120.);
                        TargetMenuButton::default()
                            .state(&mut self.logs_state.target_filter)
                            .show(ui)
                    });
                    row.col(|ui|{
                        ui.set_min_width(120.);
                        ui.label("Message");
                    });
                }).body(|mut body|{
                    let heights = filtered_events.iter().map(|event|{
                        let message = match event.fields.get("message") {
                            Some(message) => message.as_str(),
                            None => "No Message available",
                        };
                        egui::Label::new(
                            egui::RichText::new(message)
                                .color(Color32::WHITE)
                        ).wrap_mode(TextWrapMode::Extend)
                            .layout_in_ui(body.ui_mut())
                            .2
                            .rect
                            .height()
                    }).collect::<Vec<_>>();
                    body.heterogeneous_rows(heights.into_iter(), |mut row| {
                        match filtered_events.get(row.index()) {
                            None => {
                                for _ in 0..5 {
                                    row.col(|ui|{
                                        ui.label("Out of bounds index");
                                    });
                                }
                            }
                            Some(event) => {
                                row.col(|ui|{
                                    ui.add(egui::Label::new(egui::RichText::new(event.time.format_short()).color(Color32::GRAY)))
                                        .on_hover_text(event.time.format_detailed());
                                });
                                row.col(|ui|{
                                    ui.add(egui::Label::new(egui::RichText::new(event.level.as_str()).color(event.level.to_color32())));
                                });
                                row.col(|ui|{
                                    ui.add(egui::Label::new(egui::RichText::new(&event.target).color(Color32::GRAY)).wrap_mode(TextWrapMode::Truncate)).on_hover_text(&event.target);
                                });
                                row.col(|ui|{
                                    let message = match event.fields.get("message") {
                                        Some(message) => message.as_str(),
                                        None => "No Message available",
                                    };
                                    ui.add(egui::Label::new(egui::RichText::new(message).color(Color32::WHITE))
                                        .wrap_mode(TextWrapMode::Extend))
                                        .on_hover_text(message);
                                });
                            }
                        }
                    }
                )
            });
        }).response
    }
}
