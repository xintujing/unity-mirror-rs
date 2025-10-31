#[derive(Default, Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum PlayType {
    #[default]
    None = 0,
    T1_4,
    T2_3,
}

#[derive(Default)]
pub struct Settlement {
    // 底分
    pub base_score: u32,
    // PlayType
    pub play_type: PlayType,
    // 大 A 玩家出去的数量
    pub ace_out_count: u32,
    // 普通玩家出牌数量
    pub ordinary_out_count: u32,
}

impl Settlement {
    pub fn new(base_score: u32) -> Self {
        Self {
            base_score,
            play_type:
            PlayType::None,
            ace_out_count: 0,
            ordinary_out_count: 0,
        }
    }

    pub fn can_settlement(&self) -> bool {
        match self.play_type {
            PlayType::T1_4 => {
                if self.ace_out_count == 1 || self.ordinary_out_count == 4 {
                    return true;
                }
            }
            PlayType::T2_3 => {
                if self.ace_out_count == 2 || self.ordinary_out_count == 3 || (self.ordinary_out_count == 1 && self.ace_out_count == 2) || (self.ace_out_count == 2 && self.ordinary_out_count == 2) {
                    return true;
                }
            }
            _ => {
                log::error!("Settlement can not settlement, play_type is None");
                return false;
            }
        }
        false
    }

    pub fn calculate_score(&self) {}

    pub fn reset(&mut self) {
        self.base_score = 1;
        self.play_type = PlayType::None;
        self.ace_out_count = 0;
        self.ordinary_out_count = 0;
    }
}
