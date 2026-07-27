use crate::modules::risk::domain::context::RiskContext;

pub fn score(ctx: &RiskContext) -> u8 {
    if ctx.original_user_agent != ctx.user_agent {
        return 15;
    }

    if ctx.device_count_30d >= 5 {
        return 15;
    }

    0
}
