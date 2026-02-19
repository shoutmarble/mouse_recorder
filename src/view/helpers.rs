use super::*;

impl App {
    pub(crate) fn thumb_handle_from_base64(&self, png_base64: &str) -> Option<iced::widget::image::Handle> {
        use base64::engine::general_purpose;
        use base64::Engine;

        const THUMB_SIZE: u32 = 48;

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        png_base64.hash(&mut hasher);
        THUMB_SIZE.hash(&mut hasher);
        let key = hasher.finish();

        if let Some(handle) = self.thumb_cache.borrow().get(&key) {
            return Some(handle.clone());
        }

        let bytes = general_purpose::STANDARD.decode(png_base64).ok()?;

        let decoded = ::image::load_from_memory(&bytes).ok()?;
        let resized = decoded.resize(
            THUMB_SIZE,
            THUMB_SIZE,
            ::image::imageops::FilterType::Triangle,
        );

        let rgba = resized.to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());
        let pixels = rgba.into_raw();
        let handle = iced::widget::image::Handle::from_rgba(w, h, pixels);
        self.thumb_cache.borrow_mut().insert(key, handle.clone());
        Some(handle)
    }

    pub(crate) fn thumb_handle_for_event(&self, ev: &RecordedEvent) -> Option<iced::widget::image::Handle> {
        match &ev.kind {
            RecordedEventKind::FindTarget { patch_png_base64, .. } => {
                self.thumb_handle_from_base64(patch_png_base64)
            }
            RecordedEventKind::LeftClick { patch_png_base64: Some(b64) }
            | RecordedEventKind::RightClick { patch_png_base64: Some(b64) }
            | RecordedEventKind::LeftDown { patch_png_base64: Some(b64) }
            | RecordedEventKind::LeftUp { patch_png_base64: Some(b64) }
            | RecordedEventKind::RightDown { patch_png_base64: Some(b64) }
            | RecordedEventKind::RightUp { patch_png_base64: Some(b64) }
            | RecordedEventKind::MiddleDown { patch_png_base64: Some(b64) }
            | RecordedEventKind::MiddleUp { patch_png_base64: Some(b64) }
            | RecordedEventKind::MiddleClick { patch_png_base64: Some(b64) } => {
                self.thumb_handle_from_base64(b64)
            }
            _ => None,
        }
    }

    pub(crate) fn preview_handle_from_base64(&self, png_base64: &str) -> Option<iced::widget::image::Handle> {
        use base64::engine::general_purpose;
        use base64::Engine;

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        png_base64.hash(&mut hasher);
        let key = hasher.finish();

        if let Some(handle) = self.preview_cache.borrow().get(&key) {
            return Some(handle.clone());
        }

        let bytes = general_purpose::STANDARD.decode(png_base64).ok()?;
        let handle = iced::widget::image::Handle::from_bytes(bytes);
        self.preview_cache.borrow_mut().insert(key, handle.clone());
        Some(handle)
    }

    pub(crate) fn view_click_target_modes_row(
        &self,
        target: ClickTarget,
        selected_mode: ClickEdgeMode,
        on_mode_selected: fn(ClickEdgeMode) -> Message,
        target_col_w: f32,
    ) -> Element<'_, Message> {
        let enabled = self.editor_click_target == target;

        let target_toggle = radio(
            target.label(),
            target,
            Some(self.editor_click_target),
            Message::EditorClickTargetSelected,
        );

        let active_badge: Element<Message> = if enabled {
            container(text("ACTIVE").size(10))
                .padding([1, 6])
                .style(|_| iced::widget::container::Style {
                    text_color: Some(Color::from_rgb8(0xc8, 0xe6, 0xc9)),
                    background: Some(Background::Color(Color::from_rgb8(0x1f, 0x3a, 0x2a))),
                    border: Border {
                        color: Color::from_rgb8(0x5b, 0x9f, 0x6b),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                })
                .into()
        } else {
            container(iced::widget::Space::new())
                .width(Length::Fixed(0.0))
                .height(Length::Fixed(0.0))
                .into()
        };

        let target_cell: Element<Message> = container(
            row![
                target_toggle,
                container(iced::widget::Space::new()).width(Length::Fill),
                active_badge,
            ]
            .spacing(4)
            .align_y(alignment::Alignment::Center),
        )
        .width(Length::Fixed(target_col_w))
        .style(move |_| iced::widget::container::Style {
            text_color: if enabled {
                None
            } else {
                Some(Color::from_rgb8(0x74, 0x7c, 0x88))
            },
            background: None,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        })
        .into();

        let modes_row: Element<Message> = row![
            radio(
                ClickEdgeMode::Down.label(),
                ClickEdgeMode::Down,
                Some(selected_mode),
                on_mode_selected,
            ),
            radio(
                ClickEdgeMode::Up.label(),
                ClickEdgeMode::Up,
                Some(selected_mode),
                on_mode_selected,
            ),
            radio(
                ClickEdgeMode::Auto.label(),
                ClickEdgeMode::Auto,
                Some(selected_mode),
                on_mode_selected,
            ),
            radio(
                ClickEdgeMode::Double.label(),
                ClickEdgeMode::Double,
                Some(selected_mode),
                on_mode_selected,
            ),
        ]
        .spacing(6)
        .align_y(alignment::Alignment::Center)
        .into();

        let modes: Element<Message> = container(modes_row)
            .width(Length::Fill)
            .style(move |_| iced::widget::container::Style {
                text_color: if enabled {
                    None
                } else {
                    Some(Color::from_rgb8(0x74, 0x7c, 0x88))
                },
                background: Some(Background::Color(if enabled {
                    Color::from_rgb8(0x17, 0x1a, 0x1f)
                } else {
                    Color::from_rgb8(0x12, 0x15, 0x1a)
                })),
                border: Border {
                    color: if enabled {
                        Color::from_rgb8(0x5d, 0x68, 0x78)
                    } else {
                        Color::from_rgb8(0x3f, 0x47, 0x53)
                    },
                    width: 1.0,
                    radius: 6.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            })
            .padding(4)
            .into();

        row![target_cell, modes]
            .align_y(alignment::Alignment::Center)
            .spacing(6)
            .width(Length::Fill)
            .into()
    }
}
