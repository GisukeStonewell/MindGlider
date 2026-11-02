//! Windows may deliver IME Commit and its Enter key in either order or adjacent frames.
use eframe::egui::{Event, ImeEvent, Key};

#[derive(Default, Debug)]
pub struct ImeGuard {
    composing: bool,
    recent_commit: Option<f64>,
    enter_held: bool,
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
        if commit {
            self.recent_commit = Some(now);
        }
        let confirm = was_composing
            || preedit
            || commit
            || self.recent_commit.is_some_and(|t| now - t <= 0.15);
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
            true
        });
        if blocked {
            events.retain(|e| !matches!(e,Event::Text(s) if s=="\n"||s=="\r"));
        }
        for e in events.iter() {
            match e {
                Event::Ime(ImeEvent::Enabled) => self.composing = false,
                Event::Ime(ImeEvent::Preedit(s)) => self.composing = !s.is_empty(),
                Event::Ime(ImeEvent::Commit(_) | ImeEvent::Disabled) => self.composing = false,
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Modifiers;
    fn enter(pressed: bool) -> Event {
        Event::Key {
            key: Key::Enter,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
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
