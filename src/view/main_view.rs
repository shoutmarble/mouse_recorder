use super::*;

impl App {
    pub(crate) fn view(&self) -> Element<'_, Message> {
        let bg = Color::from_rgb8(0x1f, 0x22, 0x26);
        let chrome = match self.mode {
            Mode::Recording => Color::from_rgb8(0xd3, 0x2f, 0x2f),
            Mode::Playing => Color::from_rgb8(0x2e, 0x7d, 0x32),
            Mode::Idle => bg,
        };

        let duration_ms = self
            .events
            .last()
            .map(|e| e.ms_from_start)
            .unwrap_or(0);

        let frame_progress_text = {
            let total = self.events.len();
            if total == 0 {
                "Frame 0-0".to_string()
            } else {
                let current = self
                    .playback_active_index
                    .map(|i| i.saturating_add(1).min(total))
                    .unwrap_or(0);
                format!("Frame {current}-{total}")
            }
        };

        let lower_left_text = match self.mode {
            Mode::Playing => format!("Playing: {frame_progress_text}"),
            Mode::Recording => format!("Recording: {frame_progress_text}"),
            Mode::Idle => format!("Idle: {frame_progress_text}"),
        };

        let can_record = self.mode != Mode::Playing;
        let can_play = !self.events.is_empty() && self.mode != Mode::Recording;

        let record_icon = svg(iced::widget::svg::Handle::from_memory(
            include_bytes!("../../assets/icons/record.svg").as_slice(),
        ))
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(18.0));
        let stop_icon = svg(iced::widget::svg::Handle::from_memory(
            include_bytes!("../../assets/icons/stop.svg").as_slice(),
        ))
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(18.0));
        let play_icon = svg(iced::widget::svg::Handle::from_memory(
            include_bytes!("../../assets/icons/play.svg").as_slice(),
        ))
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(18.0));
        let clear_icon = svg(iced::widget::svg::Handle::from_memory(
            include_bytes!("../../assets/icons/clear.svg").as_slice(),
        ))
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(18.0));

        let run_group = row![
            tooltip(
                button(record_icon)
                    .padding(8)
                    .on_press_maybe(can_record.then_some(Message::StartRecording)),
                "Starts recording on the first click.",
                TooltipPosition::Top,
            ),
            tooltip(
                button(stop_icon).padding(8).on_press(Message::StopRecording),
                "Stops recording or playback. Press ESC as a shortcut.",
                TooltipPosition::Top,
            ),
            tooltip(
                button(play_icon)
                    .padding(8)
                    .on_press_maybe(can_play.then_some(Message::StartPlayback)),
                "Plays back the current event list.",
                TooltipPosition::Top,
            ),
            tooltip(
                button(clear_icon).padding(8).on_press(Message::Clear),
                "Clears all recorded rows.",
                TooltipPosition::Top,
            ),
        ]
        .spacing(10)
        .align_y(alignment::Alignment::Center);

        let file_group = row![
            text("File:").size(14),
            text_input("recording.yaml", &self.file_path)
                .on_input(Message::FilePathChanged)
                .width(Length::Fixed(240.0)),
            tooltip(
                button(text("⭳").size(18))
                    .padding(8)
                    .on_press(Message::LoadFromFile),
                "Loads a recording from the selected file.",
                TooltipPosition::Top,
            ),
            tooltip(
                button(text("💾").size(18))
                    .padding(8)
                    .on_press(Message::SaveToFile),
                "Saves the current recording to the selected file.",
                TooltipPosition::Top,
            ),
        ]
        .spacing(8)
        .align_y(alignment::Alignment::Center);

        let ribbon_row = row![
            text("Mouse Recorder").size(18),
            container(iced::widget::Space::new()).width(Length::Fixed(10.0)),
            run_group,
            container(iced::widget::Space::new()).width(Length::Fixed(10.0)),
            file_group,
            container(iced::widget::Space::new()).width(Length::Fill),
        ]
        .spacing(8)
        .align_y(alignment::Alignment::Center);

        let ribbon = container(ribbon_row)
            .padding(10)
            .style(|_| iced::widget::container::Style {
                text_color: None,
                background: Some(Background::Color(Color::from_rgb8(0x18, 0x1b, 0x1f))),
                border: Border {
                    color: Color::from_rgb8(0x3a, 0x3f, 0x46),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            });

        let table_header = row![
            text("#").size(14).width(Length::Fixed(32.0)),
            text("Img").size(14).width(Length::Fixed(48.0)),
            text("Action").size(14).width(Length::Fixed(220.0)),
            text("Value").size(14).width(Length::Fill),
            text("Ops").size(14).width(Length::Fixed(112.0)),
        ]
        .spacing(10)
        .align_y(alignment::Alignment::Center);

        let mut last_pos: Option<(i32, i32)> = None;
        let mut rows: Vec<Element<Message>> = Vec::with_capacity(self.events.len().min(1000) + 1);
        let dynamic_moves_view = true;

        let mut i = 0usize;
        let mut shown_rows = 0usize;
        while i < self.events.len() && shown_rows < 1000 {
            let row_start = i;
            let (row_end, action, value, thumb, new_last_pos) = if dynamic_moves_view {
                if let RecordedEventKind::Move { .. } = self.events[i].kind {
                    let mut j = i;
                    let mut points = Vec::new();
                    while j < self.events.len() {
                        if let RecordedEventKind::Move { x, y } = self.events[j].kind {
                            points.push((x, y));
                            j += 1;
                        } else {
                            break;
                        }
                    }

                    if points.len() >= 2 {
                        let last_point = points.last().copied();
                        (
                            j - 1,
                            format!("MOVES {}-{}|(X,Y)", i, j - 1),
                            format!("{} pts | dynamic", points.len()),
                            container(iced::widget::Space::new())
                                .width(Length::Fixed(48.0))
                                .height(Length::Fixed(48.0))
                                .into(),
                            last_point,
                        )
                    } else {
                        let ev = &self.events[i];
                        let (action, value, new_last_pos) = format_event_with_prev(ev, last_pos);
                        let thumb: Element<Message> = if let Some(handle) = self.thumb_handle_for_event(ev) {
                            container(image(handle).width(Length::Fixed(48.0)).height(Length::Fixed(48.0)))
                                .width(Length::Fixed(48.0))
                                .height(Length::Fixed(48.0))
                                .into()
                        } else {
                            container(iced::widget::Space::new())
                                .width(Length::Fixed(48.0))
                                .height(Length::Fixed(48.0))
                                .into()
                        };
                        (i, action, value, thumb, new_last_pos)
                    }
                } else {
                    let ev = &self.events[i];
                    let (action, value, new_last_pos) = format_event_with_prev(ev, last_pos);
                    let thumb: Element<Message> = if let Some(handle) = self.thumb_handle_for_event(ev) {
                        container(image(handle).width(Length::Fixed(48.0)).height(Length::Fixed(48.0)))
                            .width(Length::Fixed(48.0))
                            .height(Length::Fixed(48.0))
                            .into()
                    } else {
                        container(iced::widget::Space::new())
                            .width(Length::Fixed(48.0))
                            .height(Length::Fixed(48.0))
                            .into()
                    };
                    (i, action, value, thumb, new_last_pos)
                }
            } else {
                let ev = &self.events[i];
                let (action, value, new_last_pos) = format_event_with_prev(ev, last_pos);
                let thumb: Element<Message> = if let Some(handle) = self.thumb_handle_for_event(ev) {
                    container(image(handle).width(Length::Fixed(48.0)).height(Length::Fixed(48.0)))
                        .width(Length::Fixed(48.0))
                        .height(Length::Fixed(48.0))
                        .into()
                } else {
                    container(iced::widget::Space::new())
                        .width(Length::Fixed(48.0))
                        .height(Length::Fixed(48.0))
                        .into()
                };
                (i, action, value, thumb, new_last_pos)
            };

            last_pos = new_last_pos;

            let is_selected = self
                .selected_index
                .map(|idx| idx >= row_start && idx <= row_end)
                .unwrap_or(false);
            let is_playing_row = self
                .playback_active_index
                .map(|idx| idx >= row_start && idx <= row_end)
                .unwrap_or(false)
                && self.mode == Mode::Playing;

            let clickable = mouse_area(
                row![
                    text(row_start.to_string()).size(14).width(Length::Fixed(32.0)),
                    thumb,
                    text(action).size(14).width(Length::Fixed(220.0)),
                    text(value).size(14).width(Length::Fill),
                ]
                .spacing(10)
                .align_y(alignment::Alignment::Center),
            )
            .on_press(Message::SelectRow(row_end));

            let row_ops = row![
                tooltip(
                    button(text("↗").size(14))
                        .padding([4, 8])
                        .on_press(Message::RowJump(row_end)),
                    "Jump to this row target",
                    TooltipPosition::Top,
                ),
                tooltip(
                    button(text("⧉").size(14))
                        .padding([4, 8])
                        .on_press(Message::RowClone(row_end)),
                    "Clone this row",
                    TooltipPosition::Top,
                ),
                tooltip(
                    button(text("🗑").size(14))
                        .padding([4, 8])
                        .on_press(Message::RowDelete(row_end)),
                    "Delete this row",
                    TooltipPosition::Top,
                ),
            ]
            .spacing(6)
            .width(Length::Fixed(112.0))
            .align_y(alignment::Alignment::Center);

            let row_content = row![container(clickable).width(Length::Fill), row_ops]
                .spacing(10)
                .align_y(alignment::Alignment::Center);

            let is_even = row_start % 2 == 0;
            let styled = container(row_content)
                .padding(6)
                .style(move |_| iced::widget::container::Style {
                    text_color: None,
                    background: Some(Background::Color(if is_playing_row {
                        Color::from_rgb8(0x33, 0x35, 0x24)
                    } else if is_selected {
                        Color::from_rgb8(0x2a, 0x30, 0x38)
                    } else if is_even {
                        Color::from_rgb8(0x28, 0x2c, 0x34)
                    } else {
                        Color::from_rgb8(0x22, 0x26, 0x2c)
                    })),
                    border: Border {
                        color: if is_playing_row {
                            Color::from_rgb8(0xd4, 0xc3, 0x5a)
                        } else if is_selected {
                            Color::from_rgb8(0x4a, 0xa3, 0xff)
                        } else {
                            Color::from_rgb8(0x3a, 0x3f, 0x46)
                        },
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                });

            rows.push(styled.into());
            shown_rows += 1;
            i = row_end + 1;
        }

        let pos_value = if let Some((x, y)) = self.current_pos {
            format!(
                "({}, {})  L:{}  R:{}  M:{}",
                x,
                y,
                if self.current_left_down { "down" } else { "up" },
                if self.current_right_down { "down" } else { "up" },
                if self.current_middle_down { "down" } else { "up" }
            )
        } else {
            format!(
                "(unknown)  L:{}  R:{}  M:{}",
                if self.current_left_down { "down" } else { "up" },
                if self.current_right_down { "down" } else { "up" },
                if self.current_middle_down { "down" } else { "up" }
            )
        };

        rows.push(
            row![
                container(iced::widget::Space::new()).width(Length::Fixed(32.0)),
                container(iced::widget::Space::new()).width(Length::Fixed(48.0)),
                text("LIVE|(X,Y)").size(14).width(Length::Fixed(220.0)),
                text(pos_value).size(14).width(Length::Fill),
                container(iced::widget::Space::new()).width(Length::Fixed(112.0)),
            ]
            .spacing(10)
            .align_y(alignment::Alignment::Center)
            .into(),
        );

        let list = scrollable(column(rows).spacing(6))
            .id(self.events_scroll_id.clone())
            .height(Length::Fill)
            .width(Length::Fill);

        let actions_panel = container(
            iced::widget::column![table_header, list]
                .spacing(8)
                .width(Length::Fill),
        )
        .padding(8)
        .width(Length::FillPortion(3))
        .height(Length::Fill)
        .style(|_| iced::widget::container::Style {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x1a, 0x1e, 0x24))),
            border: Border {
                color: Color::from_rgb8(0x4a, 0x54, 0x62),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        });

        let right_panel = iced::widget::column![
            self.view_scale_panel(),
            self.view_properties_panel(),
        ]
        .spacing(10)
        .width(Length::Fixed(self.right_panel_width_px()))
        .height(Length::Fill);

        let bottom = row![actions_panel, right_panel]
            .spacing(12)
            .height(Length::Fill);

        let footer = container(
            row![
                text(lower_left_text).size(12),
                text("•").size(12),
                text(&self.status).size(12),
                container(iced::widget::Space::new()).width(Length::Fill),
                text(format!("Events: {}", self.events.len())).size(12),
                text(format!("Duration: {} ms", duration_ms)).size(12),
                text("ESC stops record/play").size(12),
            ]
            .spacing(14)
            .align_y(alignment::Alignment::Center),
        )
        .padding(8)
        .style(|_| iced::widget::container::Style {
            text_color: None,
            background: Some(Background::Color(Color::from_rgb8(0x18, 0x1b, 0x1f))),
            border: Border {
                color: Color::from_rgb8(0x3a, 0x3f, 0x46),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        });

        let main = container(
            iced::widget::column![ribbon, bottom, footer]
                .spacing(14)
                .padding(16),
        )
        .style(move |_| iced::widget::container::Style {
            text_color: None,
            background: Some(Background::Color(bg)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        });

        let chrome_container = container(main)
            .padding(4)
            .style(move |_| iced::widget::container::Style {
                text_color: None,
                background: Some(Background::Color(chrome)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        if let Some(draft) = &self.find_target_modal {
            let modal_panel = self.view_find_target_modal(draft);
            stack![chrome_container, modal_panel].into()
        } else if let Some(draft) = &self.wait_modal {
            let modal_panel = self.view_wait_modal(draft);
            stack![chrome_container, modal_panel].into()
        } else {
            chrome_container
        }
    }
}
