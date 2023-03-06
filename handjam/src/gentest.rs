use ive_macros::ive_chain;

#[derive(Default,Copy,Clone,PartialEq)]
pub enum DirtyEnum {
    #[default]
    NeedCompute,
    Stale,
    Clean,
}

fn add_one(a: u32) -> u32 {
    a + 1
}

fn zero() -> u32 {
    0
}

pub const CHAIN_LENGTH: usize = 1000;

ive_chain!(1000); 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain() {
        let mut state = ChainState::default();
        let mut dirty = ChainDirty::default();

        state.value0 = Some(0);

        let count = chain(&mut state, &mut dirty);

        assert_eq!(state.value3, Some(3));
        assert_eq!(count, CHAIN_LENGTH);
    }

    #[test]
    fn test_straightchain() {
        let mut state = ChainState::default();
        let mut dirty = ChainDirty::default();

        state.value0 = Some(0);

        let count = chain_straightline(&mut state, &mut dirty);

        assert_eq!(state.value3, Some(3));
        assert_eq!(count, CHAIN_LENGTH);
    }
}
