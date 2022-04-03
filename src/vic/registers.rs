//! Color registers.

use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Register {
    Background,
    Border,
    Aux,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalColors {
    pub background: u8,
    pub border: u8,
    pub aux: u8,
}

impl Default for GlobalColors {
    fn default() -> Self {
        Self {
            background: 0,
            border: 1,
            aux: 2,
        }
    }
}
impl Index<Register> for GlobalColors {
    type Output = u8;
    fn index(&self, index: Register) -> &Self::Output {
        match index {
            Register::Background => &self.background,
            Register::Border => &self.border,
            Register::Aux => &self.aux,
        }
    }
}
impl IndexMut<Register> for GlobalColors {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        match index {
            Register::Background => &mut self.background,
            Register::Border => &mut self.border,
            Register::Aux => &mut self.aux,
        }
    }
}
