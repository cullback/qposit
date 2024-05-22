#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    #[must_use]
    pub const fn new(is_buy: bool) -> Self {
        if is_buy {
            Self::Buy
        } else {
            Self::Sell
        }
    }

    #[must_use]
    pub fn is_buy(self) -> bool {
        self == Self::Buy
    }
}

impl std::ops::Not for Side {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}
