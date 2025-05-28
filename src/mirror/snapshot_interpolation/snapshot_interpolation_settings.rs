#![allow(dead_code)]
pub struct SnapshotInterpolationSettings {
    // 本地模拟落后 sendInterval * multiplier 秒。
    // 这保证了我们在缓冲区中始终有足够的快照来缓解延迟和抖动。
    // 如果模拟不流畅，请增加此值。默认情况下，它应该在 2 左右。
    pub buffer_time_multiplier: f64,

    // 如果客户端不能足够快地处理快照，就不要存储太多。
    pub buffer_limit: i32,

    // 当本地时间线向远程时间移动得太快时，速度就会开始减慢。阈值以帧为单位，表示快照的帧数。
    // 该值必须为负数。除非您知道自己在做什么，否则不要进行修改。
    pub catchup_negative_threshold: f32, // 小心，不要用完快照

    // 当本地时间线移动得太慢并且与远程时间相差太远时，开始追赶。阈值以帧为单位的快照。
    // 这必须是正数。除非您知道自己在做什么，否则不要进行修改。
    pub catchup_positive_threshold: f32,

    // 追赶过程中的本地时间线加速（以%）。
    pub catchup_speed: f64, // 查看 snap interp 演示。1% 太慢了。

    // 本地时间线减速% 同时减速。
    pub slowdown_speed: f64, // 放慢速度，这样我们就不会遇到空缓冲区（=抖动）

    // 追赶/减速是通过 n 秒指数移动平均线进行调整的。
    pub drift_ema_duration: i32, // 不需要修改它，但无论如何都要公开它

    // 自动调整 buffer_time_multiplier 以获得平滑的结果。
    // 在稳定的连接上设置较低的乘数，在抖动的连接上设置较高的乘数。
    pub dynamic_adjustment: bool,

    // 始终添加到动态 buffer_time_multiplier 调整中的安全缓冲区
    pub dynamic_adjustment_tolerance: f32, // 1 实际上就很好了，即使抖动达到 20%，2 也非常安全。也可以是半帧。

    // 动态调整是根据 n 秒指数移动平均标准差计算的。
    pub delivery_time_ema_duration: i32, // 建议 1-2 秒来捕捉平均交货时间
}

impl SnapshotInterpolationSettings {
    pub fn new() -> Self {
        Self {
            buffer_time_multiplier: 2.0,
            buffer_limit: 32,
            catchup_negative_threshold: -1.0,
            catchup_positive_threshold: 1.0,
            catchup_speed: 0.02,
            slowdown_speed: 0.04,
            drift_ema_duration: 1,
            dynamic_adjustment: true,
            dynamic_adjustment_tolerance: 1.0,
            delivery_time_ema_duration: 2,
        }
    }
}
