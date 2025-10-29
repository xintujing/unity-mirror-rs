use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(unused)]
#[derive(Copy, Clone, Eq)]
#[repr(u8)]
pub enum GeneralSuit {
    // 红桃
    Heart = 0,
    // 黑桃
    Spade = 1,
    // 方块
    Diamond = 2,
    // 梅花
    Club = 3,
}

impl PartialEq for GeneralSuit {
    fn eq(&self, other: &Self) -> bool {
        *self as u8 == *other as u8
    }
}

impl Display for GeneralSuit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneralSuit::Heart => write!(f, "♥"),
            GeneralSuit::Spade => write!(f, "♠"),
            GeneralSuit::Diamond => write!(f, "♦"),
            GeneralSuit::Club => write!(f, "♣"),
        }
    }
}

impl TryFrom<u8> for GeneralSuit {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(GeneralSuit::Heart),
            1 => Ok(GeneralSuit::Spade),
            2 => Ok(GeneralSuit::Diamond),
            3 => Ok(GeneralSuit::Club),
            _ => Err("Invalid value for GeneralSuit"),
        }
    }
}

#[allow(unused)]
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum JokerSuit {
    // 小王
    Black = 4,
    // 大王
    Red = 5,
}

impl Display for JokerSuit {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            JokerSuit::Black => write!(f, "Black"),
            JokerSuit::Red => write!(f, "Red"),
        }
    }
}

#[allow(unused)]
#[derive(Copy, Clone, Eq)]
#[repr(u8)]
pub enum PokerRank {
    Four(GeneralSuit) = 4,
    Five(GeneralSuit) = 5,
    Six(GeneralSuit) = 6,
    Seven(GeneralSuit) = 7,
    Eight(GeneralSuit) = 8,
    Nine(GeneralSuit) = 9,
    Ten(GeneralSuit) = 10,
    Jack(GeneralSuit) = 11,
    Queen(GeneralSuit) = 12,
    King(GeneralSuit) = 13,
    Ace(GeneralSuit) = 14,
    Two(GeneralSuit) = 15,
    Three(GeneralSuit) = 16,
    Joker(JokerSuit) = 17,
}

impl TryFrom<(u8, u8)> for PokerRank {
    type Error = &'static str;
    fn try_from((value, suit): (u8, u8)) -> Result<Self, Self::Error> {
        match GeneralSuit::try_from(suit) {
            Ok(suit) => match value {
                4 => Ok(PokerRank::Four(suit)),
                5 => Ok(PokerRank::Five(suit)),
                6 => Ok(PokerRank::Six(suit)),
                7 => Ok(PokerRank::Seven(suit)),
                8 => Ok(PokerRank::Eight(suit)),
                9 => Ok(PokerRank::Nine(suit)),
                10 => Ok(PokerRank::Ten(suit)),
                11 => Ok(PokerRank::Jack(suit)),
                12 => Ok(PokerRank::Queen(suit)),
                13 => Ok(PokerRank::King(suit)),
                14 => Ok(PokerRank::Ace(suit)),
                15 => Ok(PokerRank::Two(suit)),
                16 => Ok(PokerRank::Three(suit)),
                _ => Err("Invalid value for GeneralSuit"),
            }
            Err(e) => Err(e)
        }
    }
}

impl PartialEq for PokerRank {
    fn eq(&self, other: &Self) -> bool {
        self.to_u8() == other.to_u8()
    }
}

impl Display for PokerRank {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PokerRank::Four(suit) => write!(f, "{}4", suit),
            PokerRank::Five(suit) => write!(f, "{}5", suit),
            PokerRank::Six(suit) => write!(f, "{}6", suit),
            PokerRank::Seven(suit) => write!(f, "{}7", suit),
            PokerRank::Eight(suit) => write!(f, "{}8", suit),
            PokerRank::Nine(suit) => write!(f, "{}9", suit),
            PokerRank::Ten(suit) => write!(f, "{}10", suit),
            PokerRank::Jack(suit) => write!(f, "{}J", suit),
            PokerRank::Queen(suit) => write!(f, "{}Q", suit),
            PokerRank::King(suit) => write!(f, "{}K", suit),
            PokerRank::Ace(suit) => write!(f, "{}A", suit),
            PokerRank::Two(suit) => write!(f, "{}2", suit),
            PokerRank::Three(suit) => write!(f, "{}3", suit),
            PokerRank::Joker(suit) => write!(f, "{}王", suit),
        }
    }
}


impl PokerRank {
    pub const MAX_FACE_VALUE: u8 = 147;
    pub fn to_u8(self) -> u8 {
        match self {
            PokerRank::Four(_) => 4,
            PokerRank::Five(_) => 5,
            PokerRank::Six(_) => 6,
            PokerRank::Seven(_) => 7,
            PokerRank::Eight(_) => 8,
            PokerRank::Nine(_) => 9,
            PokerRank::Ten(_) => 10,
            PokerRank::Jack(_) => 11,
            PokerRank::Queen(_) => 12,
            PokerRank::King(_) => 13,
            PokerRank::Ace(_) => 14,
            PokerRank::Two(_) => 15,
            PokerRank::Three(_) => 16,
            PokerRank::Joker(_) => 17,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Poker {
    card_deputy: u8,
    rank: PokerRank,
    pub used_big_ace: bool,
}

impl Display for Poker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rank)
    }
}

impl Debug for Poker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[allow(unused)]
impl Poker {
    pub fn new(card_deputy: u8, rank: PokerRank) -> Self {
        Poker {
            card_deputy,
            rank,
            used_big_ace: false,
        }
    }

    pub fn new_with_id(id: u16) -> Self {
        let card_deputy = (id >> 9) as u8;
        let used_big_ace = (id >> 8) & 0b1 == 1;
        let suit_value = ((id >> 5) & 0b111) as u8;
        let rank_value = id & 0b11111;

        let mut poker = Self {
            card_deputy,
            rank: PokerRank::Ace(GeneralSuit::Heart), // Default value
            used_big_ace: false,
        };

        // 王特殊处理
        if suit_value > 3 && rank_value == 17 {
            if suit_value == JokerSuit::Black as u8 {
                poker.rank = PokerRank::Joker(JokerSuit::Black)
            } else {
                poker.rank = PokerRank::Joker(JokerSuit::Red)
            }
            return poker;
        }

        let suit = match GeneralSuit::try_from(suit_value) {
            Ok(v) => { v }
            Err(_) => {
                log::error!("Invalid suit for Poker {}", suit_value);
                GeneralSuit::Heart
            }
        };

        poker.rank = match rank_value {
            4 => PokerRank::Four(suit),
            5 => PokerRank::Five(suit),
            6 => PokerRank::Six(suit),
            7 => PokerRank::Seven(suit),
            8 => PokerRank::Eight(suit),
            9 => PokerRank::Nine(suit),
            10 => PokerRank::Ten(suit),
            11 => PokerRank::Jack(suit),
            12 => PokerRank::Queen(suit),
            13 => PokerRank::King(suit),
            14 => PokerRank::Ace(suit),
            15 => PokerRank::Two(suit),
            16 => PokerRank::Three(suit),
            _ => {
                log::error!("Invalid PokerRank value: {}", rank_value);
                PokerRank::Ace(suit)
            }
        };

        poker
    }

    pub fn id(&self) -> u16 {
        let suit = match self.rank {
            PokerRank::Four(suit) => suit as u16,
            PokerRank::Five(suit) => suit as u16,
            PokerRank::Six(suit) => suit as u16,
            PokerRank::Seven(suit) => suit as u16,
            PokerRank::Eight(suit) => suit as u16,
            PokerRank::Nine(suit) => suit as u16,
            PokerRank::Ten(suit) => suit as u16,
            PokerRank::Jack(suit) => suit as u16,
            PokerRank::Queen(suit) => suit as u16,
            PokerRank::King(suit) => suit as u16,
            PokerRank::Ace(suit) => suit as u16,
            PokerRank::Two(suit) => suit as u16,
            PokerRank::Three(suit) => suit as u16,
            PokerRank::Joker(joker_suit) => joker_suit as u16,
        };
        (self.card_deputy as u16) << 9 | (self.used_big_ace as u16) << 8 | suit << 5 | self.rank.to_u8() as u16
    }

    // pub fn rank(&self) -> PokerRank {
    //     self.rank
    // }
    //
    // pub fn suit(&self) -> GeneralSuit {
    //     match self.rank {
    //         PokerRank::Four(suit) => suit,
    //         PokerRank::Five(suit) => suit,
    //         PokerRank::Six(suit) => suit,
    //         PokerRank::Seven(suit) => suit,
    //         PokerRank::Eight(suit) => suit,
    //         PokerRank::Nine(suit) => suit,
    //         PokerRank::Ten(suit) => suit,
    //         PokerRank::Jack(suit) => suit,
    //         PokerRank::Queen(suit) => suit,
    //         PokerRank::King(suit) => suit,
    //         PokerRank::Ace(suit) => suit,
    //         PokerRank::Two(suit) => suit,
    //         PokerRank::Three(suit) => suit,
    //         _ => {
    //             GeneralSuit::Heart
    //         }
    //     }
    // }
}

impl Hash for Poker {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(self.rank.to_u8())
    }
}

#[derive(Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum PokerType {
    /// 单张<br>
    /// 张数 == 1<br>
    /// 例如 4, 5, 6, 7, 8, 9, 10, J, Q, K, A, 2, 3, 王, 大A
    Single(Poker),

    /// 对子<br>
    /// 张数 == 2 && 都相同<br>
    /// 例如 4 4, 5 5, 6 6, ...
    Pair(Poker),

    /// 龙<br>
    /// 张数 > 3 && 单倍连续递增至少三步<br>
    /// 特殊情况:<br>
    /// 当 A 2 3 时 A面值为 14, B面值为15, C面值为16<br>
    /// 当 A 2 3 4 时 A面值为 1, B面值为2, C面值为3<br>
    /// 当 J Q K A 时 A面值为 14 且A为结束牌, 不能再出现 2,3...<br>
    /// 例如 3 4 5 6 7, J Q K A, ...
    Straight(Vec<Poker>),

    /// 双龙<br>
    /// 张数 >= 6 && % 2 == 0 && <br>两倍连续递增至少三步<br>
    /// 特殊情况与 Straight 一样
    DoubleStraight(Vec<Poker>),

    /// 蛋子<br>
    /// 张数 == 3 && 都相同<br><br>
    /// 例如 4 4 4, 5 5 5, 6 6 6, ...
    Triplet(Poker),

    /// 大蛋子<br>
    /// 张数 >= 4 && < 7<br>
    /// 例如 3 3 3 3, 4 4 4 4 4, 5 5 5 5 5 5, ...
    BigTriplet(usize, Poker),

    /// 双小王<br>
    /// 张数 == 2 && 小王×2
    DoubleBlackJoker,

    /// 双王<br>
    /// 张数 == 2 && 小王×1 && 大王×1
    DoubleJoker,

    /// 双大王<br>
    /// 张数 == 2 && 大王×2
    DoubleRedJoker,

    /// 连珠蛋<br>
    /// 张数 >= 9 && % 3 == 0 && <br>三倍连续递增至少三步<br>
    /// 特殊情况与 Straight 一样<br>
    /// 例如 3 3 3 4 4 4 5 5 5, 6 6 6 7 7 7 8 8 8 9 9 9, ...
    ConsecutiveTriplet(Vec<Poker>),

    /// 假八<br>
    /// 张数 == 8 && 四倍连续递增两步<br>
    /// 特殊情况与 Straight 一样<br>
    /// 例如 3 3 3 3 4 4 4 4, 5 5 5 5 6 6 6 6, 7 7 7 7 8 8 8 8, ...
    FakeEight(Vec<Poker>),

    /// 真七<br>
    /// 张数 >= 7 && <= 8 && 都相同<br>
    /// 特殊情况与 Straight 一样<br>
    /// 例如 3 3 3 3 3 3 3, 4 4 4 4 4 4 4 4, ...
    RealSeven(usize, Poker),

    /// 三王<br>
    /// 张数 == 3 && (大王|小王)×3
    TripleJoker,

    /// 四王<br>
    /// 张数 == 4 && (大王|小王)×4
    QuadrupleJoker,

    /// 双大A<br>
    /// 张数 == 2 && 牌值 == A && 花色 == 亮A花色<br>
    /// 例如(亮♥A) ♥A ♥A
    DoubleBigAce,

    /// 无效牌型
    Invalid,
}

impl Display for PokerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PokerType::Single(p) => write!(f, "Single {}", p.rank),
            PokerType::Pair(p) => write!(f, "Pair {}", p.rank),
            PokerType::Straight(ps) => write!(
                f,
                "Straight {}",
                ps.iter()
                    .map(|p| format!("{}", p.rank))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            PokerType::DoubleStraight(ps) => write!(
                f,
                "DoubleStraight {}",
                ps.iter()
                    .map(|p| format!("{}", p.rank))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            PokerType::Triplet(p) => write!(f, "Triplet {}", p.rank),
            PokerType::BigTriplet(_, p) => write!(f, "BigTriplet {}", p.rank),
            PokerType::DoubleBlackJoker => write!(f, "DoubleBlackJoker"),
            PokerType::DoubleJoker => write!(f, "DoubleJoker"),
            PokerType::DoubleRedJoker => write!(f, "DoubleRedJoker", ),
            PokerType::ConsecutiveTriplet(ps) => write!(
                f,
                "ConsecutiveTriplet {}",
                ps.iter()
                    .map(|p| format!("{}", p.rank))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            PokerType::FakeEight(ps) => write!(
                f,
                "FakeEight {}",
                ps.iter()
                    .map(|p| format!("{}", p.rank))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            PokerType::RealSeven(_, p) => write!(f, "RealSeven {}", p.rank),
            PokerType::TripleJoker => write!(f, "TripleJoker"),
            PokerType::QuadrupleJoker => write!(f, "QuadrupleJoker"),
            PokerType::DoubleBigAce => write!(f, "DoubleBigAce"),
            PokerType::Invalid => write!(f, "Invalid"),
        }
    }
}

impl PokerType {
    #[allow(unused)]
    fn to_i8(&self) -> i8 {
        match self {
            PokerType::Single(_) => 1,
            PokerType::Pair(_) => 2,
            PokerType::Straight(_) => 3,
            PokerType::DoubleStraight(_) => 4,
            PokerType::Triplet(_) => 5,
            PokerType::BigTriplet(_, _) => 6,
            PokerType::DoubleBlackJoker => 7,
            PokerType::DoubleJoker => 8,
            PokerType::DoubleRedJoker => 9,
            PokerType::ConsecutiveTriplet(_) => 10,
            PokerType::FakeEight(_) => 11,
            PokerType::RealSeven(_, _) => 12,
            PokerType::TripleJoker => 13,
            PokerType::QuadrupleJoker => 14,
            PokerType::DoubleBigAce => 15,
            PokerType::Invalid => 16,
        }
    }
}

pub struct PokerHelper;

impl PartialOrd for Poker {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.rank.to_u8().cmp(&other.rank.to_u8()))
    }
}

impl PartialOrd for JokerSuit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (*self as u8).partial_cmp(&(*other as u8))
    }
}

#[allow(unused)]
impl PokerHelper {
    // 花色掩码
    // ( >> 5 & PokerHelper::SUIT_MASK) as u8
    pub const SUIT_MASK: u16 = 0b111;

    // 牌值掩码
    // ( & PokerHelper::RANK_MASK) as u8
    pub const RANK_MASK: u16 = 0b11111;

    // 牌副掩码
    // ( >> 9 & PokerHelper::DEPUTY_MASK) as u8
    pub const DEPUTY_MASK: u16 = 0b1111111;

    // 牌掩码
    // ( & PokerHelper::POKER_MASK) as u8
    pub const POKER_MASK: u16 = 0b11111111;

    // used_big_ace_mask
    // ( >> 8 & PokerHelper::USED_BIG_ACE_MASK) as u8
    pub const USED_BIG_ACE_MASK: u16 = 0b1;

    pub fn has_any_ace(poker_ids: &[&u16]) -> bool {
        // 去掉牌副的 Ace id
        let club_ace = Poker::new(0, PokerRank::Ace(GeneralSuit::Club)).id() & PokerHelper::POKER_MASK;
        let diamond_ace = Poker::new(0, PokerRank::Ace(GeneralSuit::Diamond)).id() & PokerHelper::POKER_MASK;
        let heart_ace = Poker::new(0, PokerRank::Ace(GeneralSuit::Heart)).id() & PokerHelper::POKER_MASK;
        let spade_ace = Poker::new(0, PokerRank::Ace(GeneralSuit::Spade)).id() & PokerHelper::POKER_MASK;

        poker_ids.iter().any(|&id| {
            (id & PokerHelper::POKER_MASK) == club_ace ||
                (id & PokerHelper::POKER_MASK) == diamond_ace ||
                (id & PokerHelper::POKER_MASK) == heart_ace ||
                (id & PokerHelper::POKER_MASK) == spade_ace
        })
    }

    // 生成牌
    pub fn generate_cards(mut n: u8) -> (Vec<Poker>, Vec<u16>) {
        if n == 0 {
            n = 1
        }

        let mut pokers: Vec<Poker> = Vec::new();
        let mut ids: Vec<u16> = Vec::new();

        for i in 0..n {
            for general_suit in 0..4 {
                // A 2 3
                for rank in 14..17 {
                    if let Ok(poker_rank) = PokerRank::try_from((rank, general_suit)) {
                        let poker = Poker::new(i, poker_rank);
                        ids.push(poker.id());
                        pokers.push(poker);
                    }
                }
                // 4 5 6 7 8 9 10 J Q K
                for rank in 4..14 {
                    match PokerRank::try_from((rank, general_suit)) {
                        Ok(poker_rank) => {
                            let poker = Poker::new(i, poker_rank);
                            ids.push(poker.id());
                            pokers.push(poker);
                        }
                        Err(e) => {}
                    }
                }
            }
            // 添加小王
            let black_joker = Poker::new(i, PokerRank::Joker(JokerSuit::Black));
            ids.push(black_joker.id());
            pokers.push(black_joker);
            // 添加大王
            let red_joker = Poker::new(i, PokerRank::Joker(JokerSuit::Red));
            ids.push(red_joker.id());
            pokers.push(red_joker);
        }

        (pokers, ids)
    }

    // ids 2 cards
    pub fn ids_to_cards(ids: &[u16]) -> Vec<Poker> {
        ids.iter().map(|&id| Poker::new_with_id(id)).collect()
    }

    // cards 2 ids
    pub fn cards_to_ids(cards: &[Poker]) -> Vec<u16> {
        cards.iter().map(|card| card.id()).collect()
    }

    fn get_poker_type(pokers: &[Poker], big_ace_suit: GeneralSuit) -> PokerType {
        if pokers.is_empty() {
            return PokerType::Invalid;
        }

        let all_same = pokers.windows(2).all(|w| w[0].rank == w[1].rank);

        let joker_count = pokers
            .iter()
            .filter(|p| {
                if let PokerRank::Joker(_) = &p.rank {
                    return true;
                }
                false
            })
            .count();

        let big_ace_count = pokers
            .iter()
            .filter(|p| {
                if let PokerRank::Ace(suit) = &p.rank {
                    return big_ace_suit == *suit;
                }
                false
            })
            .count();

        // 特殊牌型优先判断
        {
            // 双王
            if pokers.len() == 2 && joker_count == 2 {
                let joker_suits = pokers
                    .iter()
                    .map(|p| {
                        if let PokerRank::Joker(suit) = p.rank {
                            return Some(suit);
                        }
                        None
                    })
                    .filter(|s| s.is_some())
                    .map(|s| s.unwrap())
                    .collect::<Vec<_>>();

                if joker_suits
                    .iter()
                    .filter(|j| **j == JokerSuit::Black)
                    .count()
                    == 2
                {
                    return PokerType::DoubleBlackJoker;
                }

                if joker_suits.iter().filter(|j| **j == JokerSuit::Red).count() == 2 {
                    return PokerType::DoubleRedJoker;
                }

                if joker_suits.contains(&JokerSuit::Black) && joker_suits.contains(&JokerSuit::Red) {
                    return PokerType::DoubleJoker;
                }
            }

            // 三王
            if pokers.len() == 3 && joker_count == 3 {
                return PokerType::TripleJoker;
            }

            // 四王
            if pokers.len() == 4 && joker_count == 4 {
                return PokerType::QuadrupleJoker;
            }

            // 双大A
            if pokers.len() == 2 && big_ace_count == 2 {
                return PokerType::DoubleBigAce;
            }
        }

        // 单张
        if pokers.len() == 1 {
            return PokerType::Single(pokers[0]);
        }

        // 对子
        if pokers.len() == 2 && all_same && big_ace_count != pokers.len() {
            return PokerType::Pair(pokers[0]);
        }

        // 蛋子
        if pokers.len() == 3 && all_same {
            return PokerType::Triplet(pokers[0]);
        }

        // 大蛋子
        if pokers.len() >= 4 && pokers.len() < 7 && all_same {
            return PokerType::BigTriplet(pokers.len(), pokers[0]);
        }

        // 真七
        if pokers.len() >= 7 && pokers.len() <= 8 && all_same {
            return PokerType::RealSeven(pokers.len(), pokers[0]);
        }

        let mut adjusted_values = pokers
            .iter()
            .map(|poker| Self::adjusted_value(poker, &pokers))
            .collect::<Vec<_>>();

        adjusted_values.sort();

        if pokers.len() >= 3 && Self::is_consecutive(adjusted_values.clone(), 1, 3) {
            return PokerType::Straight(pokers.to_vec());
        }

        if pokers.len() >= 6
            && pokers.len() % 2 == 0
            && Self::is_consecutive(adjusted_values.clone(), 2, 3)
        {
            return PokerType::DoubleStraight(Self::repeat(&pokers));
        }

        if pokers.len() == 9 && Self::is_consecutive(adjusted_values.clone(), 3, 3) {
            return PokerType::ConsecutiveTriplet(Self::repeat(&pokers));
        }

        if pokers.len() == 8 && Self::is_consecutive(adjusted_values.clone(), 4, 2) {
            return PokerType::FakeEight(Self::repeat(&pokers));
        };

        PokerType::Invalid
    }

    fn adjusted_value(poker: &Poker, pokers: &[Poker]) -> u8 {
        match &poker.rank {
            PokerRank::Ace(_) | PokerRank::Two(_) | PokerRank::Three(_) => {
                let ranks = pokers.iter().map(|p| p.rank).collect::<Vec<_>>();

                let mut has_ace = false;
                let mut has_two = false;
                let mut has_three = false;
                let mut has_four = false;
                let mut has_king = false;

                ranks.iter().for_each(|r| match r {
                    PokerRank::Four(_) => has_four = true,
                    PokerRank::King(_) => has_king = true,
                    PokerRank::Ace(_) => has_ace = true,
                    PokerRank::Two(_) => has_two = true,
                    PokerRank::Three(_) => has_three = true,
                    _ => {}
                });

                match &poker.rank {
                    PokerRank::Ace(_) if has_two && !has_king => poker.rank.to_u8() - 13,
                    PokerRank::Two(_) if has_ace && has_three => poker.rank.to_u8() - 13,
                    PokerRank::Three(_) if has_two && has_four => poker.rank.to_u8() - 13,
                    _ => poker.rank.to_u8(),
                }
            }
            _ => poker.rank.to_u8(),
        }
    }

    fn is_consecutive(values: Vec<u8>, step: u8, group: u8) -> bool {
        if (values.len() as u8) < group * step {
            return false;
        }

        let sum_face_value: u8 = values.iter().sum();

        let has_king = values.contains(&PokerRank::King(GeneralSuit::Club).to_u8());
        let has_ace = values.contains(&PokerRank::Ace(GeneralSuit::Club).to_u8());
        let has_two = values.contains(&PokerRank::Two(GeneralSuit::Club).to_u8());

        if has_king
            && has_ace
            && has_two
            && sum_face_value != (PokerRank::MAX_FACE_VALUE - 17) * step
        {
            return false;
        }

        let mut group_count = 0;
        let mut row_count = 0; // 记录连续相同差值的次数

        for i in 1..values.len() {
            let diff = values[i] - values[i - 1];
            if diff == 0 {
                row_count += 1;
                if row_count > step {
                    return false;
                }
            } else if diff == 1 {
                row_count = 0;
                group_count += 1;
            } else {
                return false;
            }
        }

        row_count == step - 1 && group_count >= group - 1
    }

    fn repeat(values: &[Poker]) -> Vec<Poker> {
        let mut set = HashSet::new();
        for value in values {
            set.insert(value);
        }

        let mut vec = set.iter().map(|x| **x).collect::<Vec<_>>();
        vec.sort_by(|a, b| {
            let ordering = a.rank.to_u8().cmp(&b.rank.to_u8());
            ordering
        });
        vec
    }

    // 比较牌
    pub fn calculate(pokers: &[Poker], next_pokers: &[Poker], big_ace_suit: GeneralSuit) -> (bool, Vec<Poker>) {
        let poker_type = Self::get_poker_type(pokers, big_ace_suit);
        log::info!("pre_poker_type: {}", poker_type);
        let next_poker_type = Self::get_poker_type(next_pokers, big_ace_suit);
        log::info!("cur_poker_type: {}", next_poker_type);

        if pokers.is_empty() {
            return (next_poker_type != PokerType::Invalid, vec![]);
        }

        if next_poker_type == PokerType::Invalid || poker_type == PokerType::Invalid {
            return (false, vec![]);
        }

        if let (PokerType::Single(poker), PokerType::Single(next_poker)) = (&poker_type, &next_poker_type) {
            // 都是 Joker
            if let (PokerRank::Joker(suit), PokerRank::Joker(next_suit)) = (&poker.rank, &next_poker.rank) {
                return (next_suit > suit, vec![]);
            };

            let mut pv = poker.rank.to_u8();
            let mut npv = next_poker.rank.to_u8();

            if let PokerRank::Ace(suit) = &poker.rank {
                log::debug!("big_ace_suit: {}, suit: {}", big_ace_suit,suit);
                if suit == &big_ace_suit {
                    pv = PokerRank::Joker(JokerSuit::Red).to_u8() + 1;
                }
            };

            if let PokerRank::Ace(next_suit) = &next_poker.rank {
                log::debug!("big_ace_suit: {}, next_suit: {}", big_ace_suit, next_suit);
                if next_suit == &big_ace_suit {
                    npv = PokerRank::Joker(JokerSuit::Red).to_u8() + 1;
                }
            };

            return (npv > pv, vec![]);
        }

        let next_pokers_all_four = next_pokers
            .iter()
            .filter(|p| {
                if let PokerRank::Four(_) = p.rank {
                    return true;
                }
                false
            })
            .count()
            == next_pokers.len();

        let pokers_all_same = pokers.windows(2).all(|w| w[0].rank == w[1].rank);

        if next_pokers.len() >= 2
            && next_pokers_all_four
            && pokers_all_same
            && pokers.len() - 1 == next_pokers.len()
        {
            return (
                true,
                pokers
                    .iter()
                    .map(|p| {
                        let mut poker_clone = p.clone();
                        poker_clone.used_big_ace = true;
                        poker_clone
                    })
                    .collect::<Vec<_>>(),
            );
        }

        if let (PokerType::Single(s_p), PokerType::BigTriplet(bt_l, _)) =
            (&poker_type, &next_poker_type)
        {
            if let PokerRank::Ace(suit) = s_p.rank {
                if *bt_l == 4 && suit == big_ace_suit {
                    return (false, vec![]);
                }
            }
        }

        if let (PokerType::Single(s_p), PokerType::Triplet(_)) = (&poker_type, &next_poker_type) {
            if let PokerRank::Ace(suit) = s_p.rank {
                return (suit == big_ace_suit, vec![]);
            }
            if let PokerRank::Joker(_) = s_p.rank {
                return (false, vec![]);
            }
        }
        let type_level = poker_type.to_i8();
        let next_type_level = next_poker_type.to_i8();
        if next_type_level > 4 && next_type_level > type_level {
            return (!(type_level == 4 && next_type_level == 5), vec![]);
        }

        if type_level != next_type_level {
            return (false, vec![]);
        }

        if let PokerType::RealSeven(l, _) = &next_poker_type {
            if *l > pokers.len() {
                return (true, vec![]);
            }
        }

        if let PokerType::BigTriplet(l, _) = &next_poker_type {
            if *l > pokers.len() {
                return (true, vec![]);
            }
        }

        if pokers.len() == next_pokers.len() {
            let adjusted = pokers
                .iter()
                .map(|p| Self::adjusted_value(p, pokers))
                .max()
                .unwrap();
            let next_adjusted = next_pokers
                .iter()
                .map(|p| Self::adjusted_value(p, pokers))
                .max()
                .unwrap();
            return (next_adjusted > adjusted, vec![]);
        }

        (false, vec![])
    }

    // 对称 洗牌
    pub fn shuffle_symmetry(poker: &[Poker], n: u8) -> Vec<Poker> {
        let mut list = poker.to_vec();
        let shuffle = |p: &mut Vec<Poker>| {
            let mut result = Vec::new();

            // 总牌数
            let len = p.len();
            // 中间位置
            let mid = len / 2;
            // 开平方
            let sqrt_mid = (mid as f32).sqrt() as usize;


            // 随机数生成器
            let mut rng = StdRng::seed_from_u64(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_nanos() as u64);

            // 标志
            let flag = rng.random_range(0..=sqrt_mid);

            // 随机中间位置
            let random_mid = mid + flag;

            // 先落下的牌
            let mut first;
            // 后落下的牌
            let mut back;

            if flag % 2 == 0 {
                first = p[..random_mid].to_vec();
                back = p[random_mid..].to_vec();
            } else {
                first = p[random_mid..].to_vec();
                back = p[..random_mid].to_vec();
            }

            // 反转 后面是 pop
            first.reverse();
            back.reverse();

            // 定义权重：1 的概率较大，2 和 3 的概率较小
            let choices = [1, 2, 3];
            let weights = [7, 2, 1]; // 权重：1 的权重是 7，2 的权重是 2，3 的权重是 1
            // 创建加权分布
            let dist = WeightedIndex::new(&weights).unwrap();

            // 洗牌
            while first.len() > 0 || back.len() > 0 {
                // 先落下随机 1-3
                let first_random = choices[dist.sample(&mut rng)];
                for _ in 0..first_random {
                    if let Some(card) = first.pop() {
                        result.push(card);
                    }
                }
                // 后落下随机 1-3
                let back_random = choices[dist.sample(&mut rng)];
                for _ in 0..back_random {
                    if let Some(card) = back.pop() {
                        result.push(card);
                    }
                }
            }

            *p = result;
        };
        for _ in 0..n {
            shuffle(&mut list);
        }
        list
    }

    // 普通 洗牌
    pub fn shuffle_simple(poker: &[Poker], n: u8) -> Vec<Poker> {
        if poker.is_empty() {
            return poker.to_vec();
        }

        let mut list = poker.to_vec();
        let shuffle = |p: &mut Vec<Poker>| {
            let mut result = Vec::new();

            // 总牌数
            let len = p.len();
            // 中间位置
            let mid = (len / 2) as i32;
            // 开平方
            let sqrt_mid = (mid as f32).sqrt() as i32;

            // 随机数生成器
            let mut rng = StdRng::seed_from_u64(SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_nanos() as u64);

            // 要抽的牌数
            let count = ((len as i32 / 3) + rng.random_range(-sqrt_mid..=sqrt_mid)) as usize;

            // 要抽的牌数的中间位置
            let count_mid = count / 2;

            // 中间可用区间
            let start_random = count_mid;
            let end_random = len - start_random;

            // 随机中间位置
            let random_mid = rng.random_range(start_random..end_random);

            // 下面的牌
            result.extend_from_slice(&p[..random_mid - count_mid]);
            // 上面的牌
            result.extend_from_slice(&p[random_mid + count_mid..]);
            // 抽出来的牌
            result.extend_from_slice(&p[random_mid - count_mid..random_mid + count_mid]);

            *p = result;
        };
        for _ in 0..n {
            shuffle(&mut list);
        }
        list
    }

    // 切牌
    pub fn cut_cards(poker: &[Poker], index: usize) -> Vec<Poker> {
        let mut list = poker.to_vec();
        let len = list.len();
        if len == 0 {
            return list;
        }
        if index >= len {
            return list;
        }
        let (left, right) = list.split_at(index);
        let mut result = right.to_vec();
        result.extend_from_slice(left);
        result
    }

    // 有 n 张 指定的 poker
    pub fn has_n_poker(cards: &[u16], poker: &Poker, n: usize) -> bool {
        let poker_id = (poker.id() & PokerHelper::POKER_MASK) as u8;
        n == cards.iter().filter(|&id| (id & PokerHelper::POKER_MASK) as u8 == poker_id).count()
    }

    // 有任何两张相同的 ACE
    pub fn has_two_ace(cards: &[u16]) -> bool {
        let mut count_1 = 0;
        let mut count_2 = 0;
        let mut count_3 = 0;
        let mut count_4 = 0;
        for id in cards.iter() {
            let poker = Poker::new_with_id(*id);
            if let PokerRank::Ace(suit) = poker.rank {
                match suit {
                    GeneralSuit::Heart => {
                        count_1 += 1;
                    }
                    GeneralSuit::Spade => {
                        count_2 += 1;
                    }
                    GeneralSuit::Diamond => {
                        count_3 += 1;
                    }
                    GeneralSuit::Club => {
                        count_4 += 1;
                    }
                }
            }
        }
        count_1 == 2 || count_2 == 2 || count_3 == 2 || count_4 == 2
    }
}

impl PokerHelper {
    #[allow(unused)]
    pub fn str_to_pokers(pokers_str: &str) -> Vec<Poker> {
        let mut pokers = vec![];
        for poker_str in pokers_str.split(' ') {
            let rank = match poker_str.to_uppercase().as_str() {
                "A" => PokerRank::Ace(GeneralSuit::Heart),
                "2" => PokerRank::Two(GeneralSuit::Heart),
                "3" => PokerRank::Three(GeneralSuit::Heart),
                "4" => PokerRank::Four(GeneralSuit::Heart),
                "5" => PokerRank::Five(GeneralSuit::Heart),
                "6" => PokerRank::Six(GeneralSuit::Heart),
                "7" => PokerRank::Seven(GeneralSuit::Heart),
                "8" => PokerRank::Eight(GeneralSuit::Heart),
                "9" => PokerRank::Nine(GeneralSuit::Heart),
                "10" => PokerRank::Ten(GeneralSuit::Heart),
                "J" => PokerRank::Jack(GeneralSuit::Heart),
                "Q" => PokerRank::Queen(GeneralSuit::Heart),
                "K" => PokerRank::King(GeneralSuit::Heart),
                "王" => PokerRank::Joker(JokerSuit::Red),
                _ => panic!("Invalid poker rank"),
            };
            pokers.push(Poker::new(1, rank));
        }
        pokers
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn face_value() {
        let poker1 = Poker::new(0, PokerRank::Joker(JokerSuit::Black));
        let poker2 = Poker::new(0, PokerRank::Joker(JokerSuit::Red));

        let calculate1 = PokerHelper::calculate(
            &[poker1],
            &[poker2, poker1],
            GeneralSuit::Diamond,
        );

        println!("{:?}", calculate1);
    }
}