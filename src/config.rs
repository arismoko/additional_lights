//! The original's one setting, `enableFireCrafting` (`AdditionalLightsConfig`): off unless the
//! player turns it on in `config/additional_lights-common.toml`. The fire recipes carry the
//! `additional_lights:enable_fire_crafting` condition, which reads it.

use std::sync::atomic::{AtomicBool, Ordering};

use mod_api::{ConditionError, LoadCondition};

/// The file's text when it does not exist yet.
pub const DEFAULT: &str = "#If true, Additional Lights fire blocks (for example fire_for_*) can be crafted via recipes.\n#Set this to false if another mod conflicts with them.\n#Run /reload or rejoin the world for the change to take effect.\nenableFireCrafting = false\n";

static FIRE_CRAFTING: AtomicBool = AtomicBool::new(false);

/// `enableFireCrafting` in a config file's text: the last such line wins, anything else is off.
pub fn parse_fire_crafting(text: &str) -> bool {
    let mut value = false;
    for line in text.lines() {
        let Some((key, rest)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == "enableFireCrafting" {
            value = rest.split('#').next().unwrap_or("").trim() == "true";
        }
    }
    value
}

/// Remember the setting a loaded or reloaded file holds.
pub fn set_from(text: &str) {
    FIRE_CRAFTING.store(parse_fire_crafting(text), Ordering::Relaxed);
}

/// `EnableFireCraftingCondition`.
pub struct EnableFireCrafting;

impl LoadCondition for EnableFireCrafting {
    fn test(&self, _args: &mod_api::serde_json::Value) -> Result<bool, ConditionError> {
        Ok(FIRE_CRAFTING.load(Ordering::Relaxed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_by_default_and_on_when_set() {
        assert!(!parse_fire_crafting(DEFAULT));
        assert!(parse_fire_crafting("enableFireCrafting = true"));
        assert!(parse_fire_crafting("# c\nenableFireCrafting=true # on\n"));
        assert!(!parse_fire_crafting("enableFireCrafting = \"true\""));
        assert!(!parse_fire_crafting(""));
    }
}
