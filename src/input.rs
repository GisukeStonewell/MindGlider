//! Windows can report IME composition changes and their keys in separate frames.
use eframe::egui::{Event, ImeEvent, Key};

#[derive(Default, Debug)]
pub struct ImeGuard {
    composing: bool,
    recent_commit: Option<f64>,
    recent_cancel: Option<f64>,
    enter_held: bool,
    escape_held: bool,
}
impl ImeGuard {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn filter(&mut self, events: &mut Vec<Event>, now: f64) {
        let was_composing = self.composing;
        let commit = events
            .iter()
            .any(|e| matches!(e, Event::Ime(ImeEvent::Commit(_))));
        let preedit = events
            .iter()
            .any(|e| matches!(e,Event::Ime(ImeEvent::Preedit(s)) if !s.is_empty()));
        let composing = was_composing || preedit;
        if commit {
            self.recent_commit = Some(now);
            self.recent_cancel = None;
        } else if composing
            && events.iter().any(|e| match e {
                Event::Ime(ImeEvent::Preedit(s)) => s.is_empty(),
                Event::Ime(ImeEvent::Disabled) => true,
                _ => false,
            })
        {
            self.recent_cancel = Some(now);
        }
        let confirm = composing || commit || self.recent_commit.is_some_and(|t| now - t <= 0.15);
        let mut blocked = false;
        events.retain(|e| {
            if let Event::Key {
                key: Key::Enter,
                pressed,
                ..
            } = e
            {
                if *pressed && (confirm || self.enter_held) {
                    self.enter_held = true;
                    blocked = true;
                    return false;
                }
                if !pressed && self.enter_held {
                    self.enter_held = false;
                    self.recent_commit = None;
                    return false;
                }
            }
            if let Event::Key {
                key: Key::Escape,
                pressed,
                ..
            } = e
            {
                if *pressed {
                    let cancel = composing || self.recent_cancel.is_some_and(|t| now - t <= 0.15);
                    if cancel || self.escape_held {
                        self.escape_held = true;
                        return false;
                    }
                } else {
                    self.recent_cancel = None;
                    if std::mem::take(&mut self.escape_held) {
                        return false;
                    }
                }
            }
            true
        });
        if blocked {
            events.retain(|e| !matches!(e,Event::Text(s) if s=="\n"||s=="\r"));
        }
        let mut index = 0;
        while let Some(event) = events.get(index) {
            match event {
                Event::Ime(ImeEvent::Enabled) => self.composing = false,
                Event::Ime(ImeEvent::Preedit(s)) => self.composing = !s.is_empty(),
                Event::Ime(ImeEvent::Disabled) if self.composing && !commit => {
                    // egui-winit reports a cancelled Windows preedit as Disabled.
                    // TextEdit needs an empty preedit to erase the uncommitted text.
                    events.insert(index, Event::Ime(ImeEvent::Preedit(String::new())));
                    self.composing = false;
                    index += 1;
                }
                Event::Ime(ImeEvent::Commit(_) | ImeEvent::Disabled) => self.composing = false,
                _ => {}
            }
            index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Modifiers;
    fn key_event(key: Key, pressed: bool) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }
    fn enter(pressed: bool) -> Event {
        key_event(Key::Enter, pressed)
    }

    #[test]
    fn escape_cancels_composition_and_the_next_press_is_available() {
        for frame_order in [0, 1, 2, 3] {
            let mut g = ImeGuard::default();
            g.filter(
                &mut vec![Event::Ime(ImeEvent::Preedit("にほん".into()))],
                0.,
            );
            let escape = key_event(Key::Escape, true);
            let cancel = Event::Ime(ImeEvent::Preedit(String::new()));
            let frames = match frame_order {
                0 => vec![vec![escape, cancel.clone()]],
                1 => vec![vec![cancel.clone(), escape]],
                2 => vec![vec![cancel.clone()], vec![escape]],
                _ => vec![vec![escape], vec![cancel.clone()]],
            };
            let mut remaining = Vec::new();
            for (index, mut events) in frames.into_iter().enumerate() {
                g.filter(&mut events, 0.01 * (index + 1) as f64);
                remaining.extend(events);
            }
            assert_eq!(remaining, [cancel]);
            let mut events = vec![key_event(Key::Escape, true)];
            g.filter(&mut events, 0.3);
            assert!(events.is_empty());
            let mut events = vec![key_event(Key::Escape, false), key_event(Key::Escape, true)];
            g.filter(&mut events, 0.31);
            assert_eq!(events, [key_event(Key::Escape, true)]);
        }
    }

    #[test]
    fn enabling_ime_without_a_preedit_does_not_consume_escape() {
        let mut g = ImeGuard::default();
        let escape = key_event(Key::Escape, true);
        let mut events = vec![Event::Ime(ImeEvent::Enabled), escape.clone()];
        g.filter(&mut events, 0.);
        assert!(events.contains(&escape));
    }
    #[test]
    fn commit_enter_in_either_order() {
        for reverse in [false, true] {
            let mut g = ImeGuard::default();
            let mut events = vec![
                Event::Ime(ImeEvent::Enabled),
                Event::Ime(ImeEvent::Preedit("にほん".into())),
            ];
            g.filter(&mut events, 0.);
            let mut events = vec![Event::Ime(ImeEvent::Commit("日本".into())), enter(true)];
            if reverse {
                events.reverse();
            }
            g.filter(&mut events, 0.1);
            assert!(!events.iter().any(|e| matches!(
                e,
                Event::Key {
                    key: Key::Enter,
                    ..
                }
            )));
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, Event::Ime(ImeEvent::Commit(_))))
            );
        }
    }
    #[test]
    fn delayed_confirmation_and_deliberate_second_enter() {
        let mut g = ImeGuard::default();
        let mut events = vec![Event::Ime(ImeEvent::Commit("日本".into()))];
        g.filter(&mut events, 0.);
        let mut events = vec![enter(true)];
        g.filter(&mut events, 0.03);
        assert!(events.is_empty());
        let mut events = vec![enter(false)];
        g.filter(&mut events, 0.04);
        let mut events = vec![enter(true)];
        g.filter(&mut events, 0.08);
        assert_eq!(events.len(), 1);
    }
    #[test]
    fn regular_enter_is_preserved() {
        let mut g = ImeGuard::default();
        let mut events = vec![enter(true)];
        g.filter(&mut events, 1.);
        assert_eq!(events.len(), 1);
    }
    #[test]
    fn enabling_ime_without_a_preedit_does_not_consume_enter() {
        let mut g = ImeGuard::default();
        let mut events = vec![Event::Ime(ImeEvent::Enabled)];
        g.filter(&mut events, 0.);
        let mut events = vec![enter(true)];
        g.filter(&mut events, 0.1);
        assert_eq!(events.len(), 1);
    }
}
