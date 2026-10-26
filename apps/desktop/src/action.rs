#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Shot,
    Text,
}

impl Action {
    pub const ALL: [Action; 2] = [Action::Shot, Action::Text];

    pub const fn id(self) -> &'static str {
        match self {
            Action::Shot => "shot",
            Action::Text => "text",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Action::Shot => "Capture a screen region",
            Action::Text => "Copy the text in a screen region",
        }
    }

    pub const fn preferred_trigger(self) -> &'static str {
        match self {
            Action::Shot => "Print",
            Action::Text => "SHIFT+Print",
        }
    }

    pub const fn args(self) -> &'static [&'static str] {
        match self {
            Action::Shot => &["shot"],
            Action::Text => &["shot", "text"],
        }
    }

    pub fn from_id(id: &str) -> Option<Action> {
        Action::ALL.into_iter().find(|a| a.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip() {
        for action in Action::ALL {
            assert_eq!(Action::from_id(action.id()), Some(action));
        }
        assert_eq!(Action::from_id("pick"), None);
    }
}
