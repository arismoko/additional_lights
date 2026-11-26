//! The standing torches' and fire pits' rules, read from Additional Lights' `PedestalTrait`,
//! `Pedestal`, `FireBase`, `FireTypeWandTrait` and the wands, over a [`World`] the game glue
//! implements. Everything here is plain data, so `cargo test` checks it without a game.

use std::collections::BTreeMap;

pub const MOD: &str = "additional_lights";

/// `PedestalTypes`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    StandingTorchS,
    StandingTorchL,
    FirePitS,
    FirePitL,
}

impl Kind {
    pub const ALL: [Kind; 4] = [
        Kind::StandingTorchS,
        Kind::StandingTorchL,
        Kind::FirePitS,
        Kind::FirePitL,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Kind::StandingTorchS => "standing_torch_s",
            Kind::StandingTorchL => "standing_torch_l",
            Kind::FirePitS => "fire_pit_s",
            Kind::FirePitL => "fire_pit_l",
        }
    }

    /// The pedestal kind of a block id, `additional_lights:fire_pit_l_stone` and the like.
    pub fn of_pedestal(block: &str) -> Option<Kind> {
        let rest = block.strip_prefix(MOD)?.strip_prefix(':')?;
        Kind::ALL.into_iter().find(|kind| {
            rest.strip_prefix(kind.name())
                .is_some_and(|m| m.starts_with('_'))
        })
    }

    /// The ignition sound: `Pedestal` picks the large one for the `L` sizes.
    pub fn ignition_sound(self) -> &'static str {
        match self {
            Kind::StandingTorchL | Kind::FirePitL => "additional_lights:fire_ignition_l",
            Kind::StandingTorchS | Kind::FirePitS => "additional_lights:fire_ignition_s",
        }
    }

    /// `Pedestal.playIgnitionSound`: louder for a fire pit.
    pub fn ignition_volume(self) -> f32 {
        match self {
            Kind::FirePitS | Kind::FirePitL => 2.0,
            Kind::StandingTorchS | Kind::StandingTorchL => 1.5,
        }
    }

    /// `FireBaseSpec.SMOKE_POS`: the smoke's height above the fire block.
    pub fn smoke_height(self) -> f64 {
        match self {
            Kind::StandingTorchS => 0.2,
            Kind::StandingTorchL => 0.7,
            Kind::FirePitS => 0.0,
            Kind::FirePitL => 0.8,
        }
    }

    /// `FireBase`'s particle: smoke over a standing torch, large smoke over a fire pit.
    pub fn smoke_particle(self) -> &'static str {
        match self {
            Kind::StandingTorchS | Kind::StandingTorchL => "minecraft:smoke",
            Kind::FirePitS | Kind::FirePitL => "minecraft:large_smoke",
        }
    }
}

/// `FireTypes`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FireType {
    Normal,
    Soul,
    Light,
}

impl FireType {
    pub fn name(self) -> &'static str {
        match self {
            FireType::Normal => "normal",
            FireType::Soul => "soul",
            FireType::Light => "light",
        }
    }

    fn parse(value: &str) -> Option<FireType> {
        match value {
            "normal" => Some(FireType::Normal),
            "soul" => Some(FireType::Soul),
            "light" => Some(FireType::Light),
            _ => None,
        }
    }

    /// `PedestalComparatorOutputSpec.resolve`.
    fn comparator_output(self) -> i32 {
        match self {
            FireType::Normal => 6,
            FireType::Soul => 3,
            FireType::Light => 9,
        }
    }

    fn fire_prefix(self) -> &'static str {
        match self {
            FireType::Normal => "fire",
            FireType::Soul => "soul_fire",
            FireType::Light => "light_fire",
        }
    }

    /// The wand an item id is, and the fire type it sets.
    pub fn of_wand(item: &str) -> Option<FireType> {
        match item.strip_prefix(MOD)?.strip_prefix(':')? {
            "soul_wand" => Some(FireType::Soul),
            "light_wand" => Some(FireType::Light),
            _ => None,
        }
    }
}

/// `PedestalFireSpec.resolve`: the fire block a pedestal kind summons for a fire type.
pub fn fire_block(kind: Kind, fire: FireType) -> String {
    format!("{MOD}:{}_for_{}", fire.fire_prefix(), kind.name())
}

/// The fire type and pedestal kind of one of the twelve fire blocks.
pub fn of_fire(block: &str) -> Option<(FireType, Kind)> {
    let rest = block.strip_prefix(MOD)?.strip_prefix(':')?;
    [FireType::Light, FireType::Soul, FireType::Normal]
        .into_iter()
        .find_map(|fire| {
            let kind = rest
                .strip_prefix(fire.fire_prefix())?
                .strip_prefix("_for_")?;
            Kind::ALL
                .into_iter()
                .find(|k| k.name() == kind)
                .map(|k| (fire, k))
        })
}

/// A block state as `/setblock` writes it: the block and its properties.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub block: String,
    pub props: BTreeMap<String, String>,
}

impl State {
    /// `minecraft:oak_stairs[facing=east,half=bottom]`, or the block alone.
    pub fn parse(text: &str) -> Option<State> {
        let (block, props) = match text.split_once('[') {
            Some((block, rest)) => (block, rest.strip_suffix(']')?),
            None => (text, ""),
        };
        let props = props
            .split(',')
            .filter(|pair| !pair.is_empty())
            .map(|pair| {
                pair.split_once('=')
                    .map(|(k, v)| (k.to_owned(), v.to_owned()))
            })
            .collect::<Option<_>>()?;
        Some(State {
            block: block.to_owned(),
            props,
        })
    }

    pub fn text(&self) -> String {
        if self.props.is_empty() {
            return self.block.clone();
        }
        let props: Vec<String> = self.props.iter().map(|(k, v)| format!("{k}={v}")).collect();
        format!("{}[{}]", self.block, props.join(","))
    }

    fn flag(&self, key: &str) -> bool {
        self.props.get(key).is_some_and(|v| v == "true")
    }

    fn with(&self, key: &str, value: impl ToString) -> State {
        let mut next = self.clone();
        next.props.insert(key.to_owned(), value.to_string());
        next
    }

    pub fn fire_type(&self) -> FireType {
        self.props
            .get("firetype")
            .and_then(|v| FireType::parse(v))
            .unwrap_or(FireType::Normal)
    }

    fn previous_fire_type(&self) -> FireType {
        self.props
            .get("previous_firetype")
            .and_then(|v| FireType::parse(v))
            .unwrap_or(FireType::Normal)
    }

    fn is_light(&self) -> bool {
        self.fire_type() == FireType::Light
    }

    fn lit(&self) -> bool {
        self.flag("lit")
    }
}

/// The level reads the rules need.
pub trait Reader {
    fn state(&self, pos: [i32; 3]) -> Option<State>;
}

/// The level writes the rules make, as `Level` makes them.
pub trait World: Reader {
    /// `Level.setBlockAndUpdate`; true when the block changed.
    fn set(&mut self, pos: [i32; 3], state: &State) -> bool;
    /// `Level.destroyBlock(pos, true)`.
    fn destroy(&mut self, pos: [i32; 3]);
    /// `Level.hasNeighborSignal`.
    fn powered(&self, pos: [i32; 3]) -> bool;
    /// `Block.defaultBlockState`.
    fn default_state(&self, block: &str) -> Option<State>;
}

fn up([x, y, z]: [i32; 3]) -> [i32; 3] {
    [x, y + 1, z]
}

/// `Fire_Light.getDestroyProgress`: the position whose state a light block above a pedestal
/// mines as, the pedestal under it. None for a fire that isn't a light block, or one with no
/// pedestal of its own kind under it.
pub fn mines_as(level: &dyn Reader, pos: [i32; 3], state: &State) -> Option<([i32; 3], State)> {
    let (fire, kind) = of_fire(&state.block)?;
    if fire != FireType::Light {
        return None;
    }
    let [x, y, z] = pos;
    let below = [x, y - 1, z];
    let pedestal = level.state(below)?;
    (Kind::of_pedestal(&pedestal.block) == Some(kind)).then_some((below, pedestal))
}

fn is_fire(state: &State) -> bool {
    of_fire(&state.block).is_some()
}

/// `isAir` and `isWater` compare the block itself: only `minecraft:air` and `minecraft:water`.
fn is_open(state: &State) -> bool {
    state.block == "minecraft:air" || state.block == "minecraft:water"
}

/// `PedestalTrait.getComparatorOutput`.
pub fn comparator_output(level: &dyn Reader, pos: [i32; 3], state: &State) -> i32 {
    if state.is_light() && !state.lit() {
        return 0;
    }
    match level.state(up(pos)) {
        Some(upper) if is_fire(&upper) && upper.flag("summoned") => {
            state.fire_type().comparator_output()
        }
        _ => 0,
    }
}

/// `PedestalTrait.withLightLit`.
fn with_light_lit(state: State, value: bool) -> State {
    if !state.is_light() || state.lit() == value {
        return state;
    }
    state.with("lit", value)
}

/// `PedestalTrait.updateLightLit`.
fn update_light_lit(level: &mut dyn World, pos: [i32; 3], state: State, value: bool) -> State {
    if !state.is_light() || state.lit() == value {
        return state;
    }
    let next = state.with("lit", value);
    level.set(pos, &next);
    next
}

/// `Pedestal.createFireState`: the summoned fire, lit as the pedestal is for a light fire.
fn fire_state(level: &dyn World, pedestal: &State) -> Option<State> {
    let kind = Kind::of_pedestal(&pedestal.block)?;
    let fire = level
        .default_state(&fire_block(kind, pedestal.fire_type()))?
        .with("set", true)
        .with("summoned", true);
    Some(if pedestal.is_light() {
        fire.with("lit", pedestal.lit())
    } else {
        fire
    })
}

fn place_fire(level: &mut dyn World, pos: [i32; 3], pedestal: &State) -> bool {
    match fire_state(level, pedestal) {
        Some(fire) => level.set(up(pos), &fire),
        None => false,
    }
}

/// `PedestalTrait.igniteByInteraction`: a right click lights the pedestal.
pub fn ignite_by_interaction(level: &mut dyn World, pos: [i32; 3], state: State) -> bool {
    let lit_changed = state.is_light() && !state.lit();
    let state = if lit_changed {
        update_light_lit(level, pos, state, true)
    } else {
        state
    };
    let light_fire_changed = ensure_light_fire(level, pos, &state);
    let changed = lit_changed || light_fire_changed;
    let Some(upper) = level.state(up(pos)) else {
        return changed;
    };
    let fire = is_fire(&upper);
    if !(is_open(&upper) || fire) || (fire && upper.flag("summoned")) {
        return changed;
    }
    if fire {
        level.destroy(up(pos));
    }
    place_fire(level, pos, &state) || changed
}

/// `PedestalTrait.igniteFire`.
pub fn ignite_fire(
    level: &mut dyn World,
    pos: [i32; 3],
    state: &State,
    replace_only: bool,
) -> bool {
    let Some(upper) = level.state(up(pos)) else {
        return false;
    };
    let fire = is_fire(&upper);
    if replace_only && !fire && !state.is_light() {
        return false;
    }
    if !(is_open(&upper) || fire) {
        return false;
    }
    if fire && !upper.flag("summoned") {
        level.destroy(up(pos));
    }
    place_fire(level, pos, state)
}

/// `PedestalTrait.removeFire`.
fn remove_fire(level: &mut dyn World, pos: [i32; 3]) {
    if level.state(up(pos)).is_some_and(|upper| is_fire(&upper)) {
        level.destroy(up(pos));
    }
}

/// `PedestalTrait.ensureLightFire`: a light pedestal keeps its light block, lit as it is.
pub fn ensure_light_fire(level: &mut dyn World, pos: [i32; 3], state: &State) -> bool {
    if !state.is_light() {
        return false;
    }
    let upper = level.state(up(pos));
    let light = upper.as_ref().filter(|u| {
        of_fire(&u.block).is_some_and(|(f, _)| f == FireType::Light) && u.flag("summoned")
    });
    let Some(upper) = light else {
        return ignite_fire(level, pos, state, false);
    };
    if upper.lit() == state.lit() {
        return false;
    }
    let next = upper.with("lit", state.lit());
    level.set(up(pos), &next)
}

/// `PedestalTrait.handleNeighborUpdate`: redstone lights and puts out a pedestal that accepts
/// power. True when a fire was lit, for `onIgnited`'s sound.
pub fn handle_neighbor_update(level: &mut dyn World, pos: [i32; 3]) -> bool {
    let Some(state) = level.state(pos) else {
        return false;
    };
    if Kind::of_pedestal(&state.block).is_none() {
        return false;
    }
    let powered = level.powered(pos);
    if !state.flag("accept_power") {
        ensure_light_fire(level, pos, &state);
        return false;
    }
    if powered {
        if state.flag("activated") {
            ensure_light_fire(level, pos, &state);
            return false;
        }
        let next = with_light_lit(state.with("activated", true).with("ispowered", true), true);
        level.set(pos, &next);
        if next.is_light() {
            ensure_light_fire(level, pos, &next);
            false
        } else {
            ignite_fire(level, pos, &next, false)
        }
    } else if state.flag("ispowered") && state.flag("activated") {
        let next = with_light_lit(
            state.with("activated", false).with("ispowered", false),
            false,
        );
        if !state.is_light() {
            remove_fire(level, pos);
        }
        level.set(pos, &next);
        ensure_light_fire(level, pos, &next);
        false
    } else {
        ensure_light_fire(level, pos, &state);
        false
    }
}

/// `Pedestal.setPlacedBy`: a placed pedestal lights at once; placed sneaking it stays unlit and
/// ignores redstone. A wand in the placer's off hand sets its fire type first.
pub fn placed(
    level: &mut dyn World,
    pos: [i32; 3],
    state: State,
    sneaking: bool,
    offhand: Option<FireType>,
) {
    let state = match offhand {
        Some(fire @ (FireType::Soul | FireType::Light)) => state.with("firetype", fire.name()),
        _ => state,
    };
    if sneaking {
        let next = with_light_lit(state.with("accept_power", false), false);
        level.set(pos, &next);
        ensure_light_fire(level, pos, &next);
    } else {
        let next = with_light_lit(state, true);
        level.set(pos, &next);
        ignite_fire(level, pos, &next, false);
    }
}

/// The two sounds a wand plays (`ModSoundList.Change` and `Undo`), with their volumes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WandSound {
    Change,
    Undo,
}

impl WandSound {
    pub fn id(self) -> &'static str {
        match self {
            WandSound::Change => "additional_lights:change",
            WandSound::Undo => "additional_lights:undo",
        }
    }

    pub fn volume(self) -> f32 {
        match self {
            WandSound::Change => 0.8,
            WandSound::Undo => 0.6,
        }
    }
}

/// `FireTypeWandTrait.computeFireChange`: a click turns the fire to the wand's type, a sneaking
/// click on that type turns it back to the one before.
fn fire_change(
    target: FireType,
    current: FireType,
    prev: FireType,
    sneaking: bool,
) -> Option<(FireType, FireType, WandSound)> {
    let prev = if prev == target {
        FireType::Normal
    } else {
        prev
    };
    match (sneaking, current == target) {
        (true, true) => Some((prev, prev, WandSound::Undo)),
        (false, false) => Some((target, current, WandSound::Change)),
        _ => None,
    }
}

/// A wand used on a floor or wall torch (`SoulWand.changeFire` on an `ALTorch`): its fire type
/// changes, which its light follows. The light wand does not take torches.
pub fn wand_on_torch(
    level: &mut dyn World,
    pos: [i32; 3],
    wand: FireType,
    sneaking: bool,
) -> Option<WandSound> {
    if wand == FireType::Light {
        return None;
    }
    let state = level.state(pos)?;
    let (next, prev, sound) = fire_change(
        wand,
        state.fire_type(),
        state.previous_fire_type(),
        sneaking,
    )?;
    level.set(
        pos,
        &state
            .with("firetype", next.name())
            .with("previous_firetype", prev.name()),
    );
    Some(sound)
}

/// A wand used on a pedestal (`SoulWand.changeFire` and `LightWand.changeFire`): its fire type
/// changes and the fire above follows.
pub fn wand_on_pedestal(
    level: &mut dyn World,
    pos: [i32; 3],
    wand: FireType,
    sneaking: bool,
) -> Option<WandSound> {
    let state = level.state(pos)?;
    Kind::of_pedestal(&state.block)?;
    let current = state.fire_type();
    let change = fire_change(wand, current, state.previous_fire_type(), sneaking);
    let mut state = state;
    if let Some((next, prev, _)) = change {
        let changed = state
            .with("firetype", next.name())
            .with("previous_firetype", prev.name());
        if level.set(pos, &changed) {
            state = changed;
        }
    }
    if wand == FireType::Light && state.is_light() {
        state = update_light_lit(level, pos, state, true);
    }
    // FireTypeWandCore.shouldReplacePedestalFireOnly.
    ignite_fire(level, pos, &state, current != FireType::Light);
    change.map(|(_, _, sound)| sound)
}

/// `FireBase.setPlacedBy`: a fire placed on its own pedestal kind sits in it.
pub fn fire_placed(level: &mut dyn World, pos: [i32; 3], state: &State) {
    let Some((_, kind)) = of_fire(&state.block) else {
        return;
    };
    let [x, y, z] = pos;
    let below = level.state([x, y - 1, z]);
    if below.is_some_and(|b| Kind::of_pedestal(&b.block) == Some(kind)) {
        level.set(pos, &state.with("set", true));
    }
}

/// `FireBase.isValidPosition`: a fire needs a block under it, and not water.
pub fn fire_supported(below: &str) -> bool {
    !matches!(
        below,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:water"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct Level {
        blocks: HashMap<[i32; 3], State>,
        powered: bool,
        destroyed: Vec<[i32; 3]>,
    }

    impl Reader for Level {
        fn state(&self, pos: [i32; 3]) -> Option<State> {
            Some(
                self.blocks
                    .get(&pos)
                    .cloned()
                    .unwrap_or_else(|| State::parse("minecraft:air").unwrap()),
            )
        }
    }

    impl World for Level {
        fn set(&mut self, pos: [i32; 3], state: &State) -> bool {
            self.blocks.insert(pos, state.clone()).as_ref() != Some(state)
        }
        fn destroy(&mut self, pos: [i32; 3]) {
            self.blocks.remove(&pos);
            self.destroyed.push(pos);
        }
        fn powered(&self, _: [i32; 3]) -> bool {
            self.powered
        }
        fn default_state(&self, block: &str) -> Option<State> {
            let mut text = format!("{block}[set=false,summoned=false,temp=false]");
            if block.contains("light_fire") {
                text = format!("{block}[lit=true,set=false,summoned=false,temp=false]");
            }
            State::parse(&text)
        }
    }

    const POS: [i32; 3] = [0, 64, 0];
    const UP: [i32; 3] = [0, 65, 0];

    fn pedestal(fire: &str) -> State {
        State::parse(&format!(
            "additional_lights:fire_pit_s_stone[accept_power=true,activated=false,firetype={fire},ispowered=false,lit=false,previous_firetype=normal,waterlogged=false]"
        ))
        .unwrap()
    }

    #[test]
    fn states_read_and_write_as_setblock_does() {
        let text = "minecraft:oak_stairs[facing=east,half=bottom]";
        assert_eq!(State::parse(text).unwrap().text(), text);
        assert_eq!(
            State::parse("minecraft:stone").unwrap().text(),
            "minecraft:stone"
        );
        assert_eq!(
            Kind::of_pedestal("additional_lights:standing_torch_l_gold_block"),
            Some(Kind::StandingTorchL)
        );
        assert_eq!(Kind::of_pedestal("additional_lights:al_torch_stone"), None);
        assert_eq!(
            of_fire("additional_lights:soul_fire_for_fire_pit_l"),
            Some((FireType::Soul, Kind::FirePitL))
        );
        assert_eq!(of_fire("additional_lights:fire_pit_l_stone"), None);
    }

    #[test]
    fn a_placed_pedestal_lights_and_a_sneaking_placement_does_not() {
        let mut level = Level::default();
        placed(&mut level, POS, pedestal("normal"), false, None);
        let fire = level.state(UP).unwrap();
        assert_eq!(fire.block, "additional_lights:fire_for_fire_pit_s");
        assert!(fire.flag("summoned") && fire.flag("set"));
        assert_eq!(
            comparator_output(&level, POS, &level.state(POS).unwrap()),
            6
        );

        let mut level = Level::default();
        placed(&mut level, POS, pedestal("normal"), true, None);
        assert_eq!(level.state(UP).unwrap().block, "minecraft:air");
        assert!(!level.state(POS).unwrap().flag("accept_power"));
        level.powered = true;
        assert!(!handle_neighbor_update(&mut level, POS));
        assert_eq!(level.state(UP).unwrap().block, "minecraft:air");
    }

    #[test]
    fn a_soul_wand_in_the_off_hand_places_soul_fire() {
        let mut level = Level::default();
        placed(
            &mut level,
            POS,
            pedestal("normal"),
            false,
            Some(FireType::Soul),
        );
        assert_eq!(
            level.state(UP).unwrap().block,
            "additional_lights:soul_fire_for_fire_pit_s"
        );
        assert_eq!(
            comparator_output(&level, POS, &level.state(POS).unwrap()),
            3
        );
    }

    #[test]
    fn redstone_lights_and_puts_out_a_pedestal() {
        let mut level = Level::default();
        level.set(POS, &pedestal("normal"));
        level.powered = true;
        assert!(handle_neighbor_update(&mut level, POS));
        assert!(of_fire(&level.state(UP).unwrap().block).is_some());
        assert!(level.state(POS).unwrap().flag("activated"));
        // A second update while still powered changes nothing.
        assert!(!handle_neighbor_update(&mut level, POS));
        level.powered = false;
        handle_neighbor_update(&mut level, POS);
        assert_eq!(level.state(UP).unwrap().block, "minecraft:air");
        assert!(!level.state(POS).unwrap().flag("ispowered"));
    }

    #[test]
    fn a_click_relights_an_empty_pedestal_but_not_a_lit_one() {
        let mut level = Level::default();
        level.set(POS, &pedestal("normal"));
        assert!(ignite_by_interaction(&mut level, POS, pedestal("normal")));
        assert!(!ignite_by_interaction(&mut level, POS, pedestal("normal")));
        // A block above stops it.
        let mut level = Level::default();
        level.set(UP, &State::parse("minecraft:stone").unwrap());
        assert!(!ignite_by_interaction(&mut level, POS, pedestal("normal")));
    }

    #[test]
    fn a_hand_placed_fire_is_replaced_by_the_summoned_one() {
        let mut level = Level::default();
        level.set(POS, &pedestal("normal"));
        let loose = level
            .default_state("additional_lights:fire_for_fire_pit_s")
            .unwrap();
        level.set(UP, &loose);
        assert!(ignite_by_interaction(&mut level, POS, pedestal("normal")));
        assert_eq!(level.destroyed, vec![UP]);
        assert!(level.state(UP).unwrap().flag("summoned"));
    }

    #[test]
    fn wands_switch_the_fire_and_a_sneaking_click_reverts_it() {
        let mut level = Level::default();
        placed(&mut level, POS, pedestal("normal"), false, None);
        assert_eq!(
            wand_on_pedestal(&mut level, POS, FireType::Soul, false),
            Some(WandSound::Change)
        );
        assert_eq!(
            level.state(UP).unwrap().block,
            "additional_lights:soul_fire_for_fire_pit_s"
        );
        assert_eq!(
            level.state(POS).unwrap().props["previous_firetype"],
            "normal"
        );
        // Clicking again with the same wand changes nothing.
        assert_eq!(
            wand_on_pedestal(&mut level, POS, FireType::Soul, false),
            None
        );
        assert_eq!(
            wand_on_pedestal(&mut level, POS, FireType::Soul, true),
            Some(WandSound::Undo)
        );
        assert_eq!(
            level.state(UP).unwrap().block,
            "additional_lights:fire_for_fire_pit_s"
        );
    }

    #[test]
    fn the_light_wand_makes_a_lit_light_block() {
        let mut level = Level::default();
        placed(&mut level, POS, pedestal("normal"), false, None);
        assert_eq!(
            wand_on_pedestal(&mut level, POS, FireType::Light, false),
            Some(WandSound::Change)
        );
        let upper = level.state(UP).unwrap();
        assert_eq!(upper.block, "additional_lights:light_fire_for_fire_pit_s");
        assert!(upper.lit());
        assert!(level.state(POS).unwrap().lit());
        assert_eq!(
            comparator_output(&level, POS, &level.state(POS).unwrap()),
            9
        );
        // Unpowered after power, a light pedestal keeps its block, unlit.
        level.powered = true;
        handle_neighbor_update(&mut level, POS);
        level.powered = false;
        handle_neighbor_update(&mut level, POS);
        let upper = level.state(UP).unwrap();
        assert_eq!(upper.block, "additional_lights:light_fire_for_fire_pit_s");
        assert!(!upper.lit());
        assert_eq!(
            comparator_output(&level, POS, &level.state(POS).unwrap()),
            0
        );
    }

    #[test]
    fn the_soul_wand_takes_torches_and_the_light_wand_does_not() {
        let mut level = Level::default();
        let torch = State::parse(
            "additional_lights:al_torch_stone[firetype=normal,previous_firetype=normal]",
        )
        .unwrap();
        level.set(POS, &torch);
        assert_eq!(wand_on_torch(&mut level, POS, FireType::Light, false), None);
        assert_eq!(
            wand_on_torch(&mut level, POS, FireType::Soul, false),
            Some(WandSound::Change)
        );
        assert_eq!(level.state(POS).unwrap().fire_type(), FireType::Soul);
    }

    #[test]
    fn a_fire_sits_only_in_its_own_pedestal_kind() {
        let mut level = Level::default();
        level.set(POS, &pedestal("normal"));
        let fire = level
            .default_state("additional_lights:fire_for_fire_pit_s")
            .unwrap();
        fire_placed(&mut level, UP, &fire);
        assert!(level.state(UP).unwrap().flag("set"));
        let other = level
            .default_state("additional_lights:fire_for_fire_pit_l")
            .unwrap();
        level.set(UP, &other);
        fire_placed(&mut level, UP, &other);
        assert!(!level.state(UP).unwrap().flag("set"));
        assert!(!fire_supported("minecraft:water") && fire_supported("minecraft:stone"));
    }

    #[test]
    fn a_light_block_mines_as_the_pedestal_under_it() {
        let mut level = Level::default();
        let stone = State::parse("additional_lights:standing_torch_s_stone").unwrap();
        let kind = Kind::of_pedestal(&stone.block).unwrap();
        level.blocks.insert([0, 63, 0], stone.clone());
        let light = State::parse(&fire_block(kind, FireType::Light)).unwrap();
        let pos = [0, 64, 0];
        assert_eq!(mines_as(&level, pos, &light), Some(([0, 63, 0], stone)));
        let normal = State::parse(&fire_block(kind, FireType::Normal)).unwrap();
        assert_eq!(mines_as(&level, pos, &normal), None);
        assert_eq!(mines_as(&Level::default(), pos, &light), None);
    }
}
