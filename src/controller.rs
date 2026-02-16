use super::*;

mod editor;
mod helpers;
mod modal;
mod runtime;

impl App {
    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        let message = match self.handle_editor_message(message) {
            Ok(task) => return task,
            Err(message) => message,
        };

        let message = match self.handle_modal_message(message) {
            Ok(task) => return task,
            Err(message) => message,
        };

        match self.handle_runtime_message(message) {
            Ok(task) => task,
            Err(_) => Task::none(),
        }
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        let record = if self.mode == Mode::Recording {
            // simple 60Hz polling recorder (mouse pos + button state)
            iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        };

        let pos = iced::time::every(Duration::from_millis(30)).map(|_| Message::PosTick);

        let capture = match &self.find_target_modal {
            Some(draft) if draft.capture_waiting => {
                iced::time::every(Duration::from_millis(16)).map(|_| Message::FindTargetCaptureTick)
            }
            _ => Subscription::none(),
        };

        let resized = iced::window::resize_events()
            .map(|(_id, size)| Message::WindowResized(size.width, size.height));

        Subscription::batch(vec![record, pos, capture, resized])
    }
}
