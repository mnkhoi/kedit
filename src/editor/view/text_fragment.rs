#[derive(Debug, Clone, Copy)]
pub enum GraphemeWidth {
    Full,
    Half,
}

impl GraphemeWidth {
    pub fn saturating_add(&self, other: usize) -> usize {
        match self {
            Self::Half => other.saturating_add(1),
            Self::Full => other.saturating_add(2),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextFragment {
    pub grapheme: String,
    pub rendered_width: GraphemeWidth,
    pub replacement: Option<char>,
}
