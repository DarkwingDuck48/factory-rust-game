use std::collections::BTreeMap;

use crate::{FactoryId, PlayerFactoryMeta};

pub struct GameState {
    factories: BTreeMap<FactoryId, PlayerFactoryMeta>,
}
