//! Additional Lights for Sunrise: the lamps and torches are content in `data/` and `assets/`;
//! this code runs the standing torches, fire pits, their fires and the wands.
//!
//! | Moment | Where | What it does |
//! |---|---|---|
//! | `register` | [`support::declare`] | attaches the pedestal, fire and torch parts |
//! | `use_item_on` | [`support::Pedestal`] | lights a pedestal |
//! | `use_on` | [`support::Wand`] | a wand turns a pedestal's or torch's fire, or turns it back sneaking |
//! | `set_placed_by` | the parts | a placed pedestal lights; a fire sits in its own pedestal |
//! | `neighbor_changed` | [`support::Pedestal`] | queues a pedestal whose neighbour changed |
//! | `level_tick_post` | this module | redstone lights or puts out each queued pedestal |
//! | `tooltip` | [`tips::Tooltip`] | the pedestals' and wands' Shift-for-info lines |
//! | `sound_type` | [`support::FireSound`] | the fires' extinguish sound |
//! | `register` (config), `register_load_conditions` | this module | `enableFireCrafting` and the condition on the fire recipes |
//! | `analog_output` | [`support::Pedestal`] | comparators read the fire's type |
//! | `update_shape` | [`support::Fire`] | a fire with nothing under it goes out |

pub mod config;
pub mod pedestal;
pub mod support;
pub mod tips;

use std::sync::Arc;

use mod_api::{
    self, ConfigEvent, ConfigKind, ConfigSnapshot, ConfigSpec, Hooks, HostSide, Identifier,
    LevelTickPostEvent, RegisterLoadConditionsEvent, Registrar, RegistrationContext,
};

/// The pedestals' materials (`tools/block_definitions.csv`).
pub const MATERIALS: [&str; 24] = [
    "cobblestone",
    "cut_sandstone",
    "diamond_block",
    "end_stone",
    "end_stone_bricks",
    "gold_block",
    "iron_block",
    "magenta_wool",
    "mossy_cobblestone",
    "mossy_stone_bricks",
    "nether_bricks",
    "packed_ice",
    "pink_wool",
    "polished_andesite",
    "polished_blackstone",
    "polished_diorite",
    "polished_granite",
    "quartz_block",
    "quartz_bricks",
    "red_nether_bricks",
    "sandstone",
    "smooth_stone",
    "stone",
    "stone_bricks",
];

/// The floor and wall torches' materials.
pub const TORCH_MATERIALS: [&str; 28] = [
    "acacia_planks",
    "birch_planks",
    "blackstone",
    "cobblestone",
    "crimson_planks",
    "dark_oak_planks",
    "diamond_block",
    "end_stone",
    "end_stone_bricks",
    "glass",
    "gold_block",
    "iron_block",
    "jungle_planks",
    "magenta_wool",
    "mossy_cobblestone",
    "mossy_stone_bricks",
    "nether_bricks",
    "oak_planks",
    "packed_ice",
    "pink_wool",
    "quartz_block",
    "red_nether_bricks",
    "sandstone",
    "smooth_stone",
    "spruce_planks",
    "stone",
    "stone_bricks",
    "warped_planks",
];

#[derive(Default)]
struct Mod;

#[mod_api::sunrise_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        support::declare(r);
        // `AdditionalLightsConfig`: `config/additional_lights-common.toml`.
        let listener = |event: ConfigEvent<'_>, snapshot: &ConfigSnapshot| -> Result<(), String> {
            if matches!(event, ConfigEvent::Loading(_) | ConfigEvent::Reloading(_)) {
                config::set_from(&snapshot.text);
            }
            Ok(())
        };
        if let Err(err) = r.register_config(
            ConfigSpec::new(ConfigKind::Common, config::DEFAULT),
            Some(Arc::new(listener)),
        ) {
            eprintln!("additional_lights: config registration failed: {err:?}");
        }
    }

    /// `EnableFireCraftingCondition`.
    fn register_load_conditions(
        &self,
        _: &RegistrationContext,
        event: &mut RegisterLoadConditionsEvent<'_>,
    ) {
        if let Ok(id) = Identifier::parse("additional_lights:enable_fire_crafting") {
            let _ = event.conditions.register(id, config::EnableFireCrafting);
        }
    }

    fn level_tick_post(&self, host: HostSide<'_>, event: &mut LevelTickPostEvent) {
        if let HostSide::Server(host) = host {
            support::on_level_tick_post(host, event.level);
        }
    }
}
