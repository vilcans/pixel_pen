use std::fmt;

use crate::{error::DisallowedAction, vic::DrawMode, Document};

pub struct Action {
    pub action: ActionType,
    previous: Option<Document>,
}

impl Action {
    pub fn new(action: ActionType) -> Self {
        Self {
            action,
            previous: None,
        }
    }
}

pub enum ActionType {
    Plot {
        x: i32,
        y: i32,
        color: u8,
        draw_mode: DrawMode,
    },
}

impl undo::Action for Action {
    type Target = Document;
    type Output = bool;
    type Error = Box<dyn DisallowedAction>;

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        let previous = target.clone();
        match &self.action {
            ActionType::Plot {
                x,
                y,
                color,
                draw_mode,
            } => match target.image.set_pixel(*x, *y, draw_mode, *color) {
                Ok(true) => {
                    self.previous = Some(previous);
                    Ok(true)
                }
                Ok(false) => Err(Box::new(NoChange)),
                other => other,
            },
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match self.previous.take() {
            Some(previous) => {
                *target = previous;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

#[derive(Debug)]
struct NoChange;

impl fmt::Display for NoChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No change")
    }
}

impl DisallowedAction for NoChange {}
