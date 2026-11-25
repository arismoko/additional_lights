//! The tooltips of the pedestals and the wands: `PedestalBlockItemTrait.buildPedestalTooltip`
//! and `WandTrait.buildWandTooltip`. A line asks for Shift; with Shift held the item lists
//! its usage instead.

use mod_api::{ClientHost, ItemPart, Text, TooltipEvent};

const SHIFT: &str = "additional_lights.txt.shift";
const TIPS: &str = "additional_lights.txt.tips";
const USAGE: &str = "additional_lights.txt.usage";
const PEDESTAL: [&str; 3] = [
    "additional_lights.txt.block.pedestal.rightclick",
    "additional_lights.txt.block.pedestal.sneaking",
    "additional_lights.txt.block.pedestal.signals",
];
const SNEAK_RIGHT_CLICK: &str = "additional_lights.txt.item.wand.sneak_rightclick";

/// Which item a tooltip is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tip {
    Pedestal,
    SoulWand,
    LightWand,
}

impl Tip {
    /// The translation keys to show, in order, when Shift is `down`.
    pub fn keys(self, down: bool) -> Vec<&'static str> {
        if !down {
            return vec![SHIFT];
        }
        match self {
            Tip::Pedestal => {
                let mut keys = vec![TIPS];
                keys.extend(PEDESTAL);
                keys
            }
            Tip::SoulWand => vec![
                USAGE,
                "additional_lights.txt.item.soul_wand.rightclick",
                SNEAK_RIGHT_CLICK,
                "additional_lights.txt.item.soul_wand.lefthand",
                "additional_lights.txt.item.soul_wand.piglin",
            ],
            Tip::LightWand => vec![
                USAGE,
                "additional_lights.txt.item.light_wand.rightclick",
                SNEAK_RIGHT_CLICK,
                "additional_lights.txt.item.light_wand.lefthand",
            ],
        }
    }
}

/// The part on every pedestal item and on the wands.
pub struct Tooltip(pub Tip);

#[mod_api::sunrise_part]
impl ItemPart for Tooltip {
    fn tooltip(&self, host: &dyn ClientHost, event: &mut TooltipEvent<'_, '_>) {
        for key in self.0.keys(host.modifiers().shift()) {
            // `u32::MAX` keeps the text's own colour, as the original adds plain components.
            let _ = event
                .lines
                .push((Text::translate(key, Vec::new()), u32::MAX));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_shift_every_item_only_asks_for_it() {
        for tip in [Tip::Pedestal, Tip::SoulWand, Tip::LightWand] {
            assert_eq!(tip.keys(false), [SHIFT]);
        }
    }

    #[test]
    fn with_shift_the_pedestal_lists_tips_and_the_soul_wand_the_piglins() {
        assert_eq!(Tip::Pedestal.keys(true).len(), 4);
        assert_eq!(Tip::SoulWand.keys(true).len(), 5);
        assert_eq!(Tip::LightWand.keys(true).len(), 4);
        assert!(
            Tip::LightWand
                .keys(true)
                .iter()
                .all(|k| !k.contains("piglin"))
        );
    }

    #[test]
    fn every_key_is_in_every_language_file() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/additional_lights/lang");
        for file in std::fs::read_dir(dir).unwrap() {
            let text = std::fs::read_to_string(file.unwrap().path()).unwrap();
            for tip in [Tip::Pedestal, Tip::SoulWand, Tip::LightWand] {
                for key in tip.keys(true).into_iter().chain(tip.keys(false)) {
                    assert!(text.contains(&format!("\"{key}\"")), "{key}");
                }
            }
        }
    }
}
