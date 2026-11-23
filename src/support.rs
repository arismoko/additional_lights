//! The game glue: the block parts on the pedestals, fires and torches, and the [`World`] the
//! rules in [`crate::pedestal`] run over.
//!
//! A neighbour change can't write the level from its hook, so a pedestal that heard one is
//! queued and settled after its level's tick (`Hooks::level_tick_post`), with the full server
//! host: the same tick as the change.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, MutexGuard};

use mod_api::{
    self, AnalogOutputEvent, AnimateTickEvent, BlockFace, BlockPart, BlockUpdateFlags, ClientHost,
    Direction, HasAnalogOutputEvent, Host, InteractionResult, LevelHost, LevelRandomHost, LevelRef,
    LevelSide, LevelTicksHost, NeighborChangedEvent, NeighbourUpdate, ParticleKey, ParticleOptions,
    PlayerRef, Registrar, ServerHost, SetPlacedByEvent, SoundSource, StateForPlacementEvent,
    SupportType, UpdateShapeEvent, UseItemOnEvent,
};

use crate::pedestal::{self, FireType, Kind, MOD, Reader, State, World};
use crate::torch;

/// A level on the server, read and written through the server host.
struct ServerLevel<'a> {
    host: &'a mut dyn ServerHost,
    level: LevelRef,
}

impl Reader for ServerLevel<'_> {
    fn state(&self, pos: [i32; 3]) -> Option<State> {
        let state = self.level.block_state(&*self.host, pos)?;
        State::parse(&self.host.block_state_string(state)?)
    }
}

impl World for ServerLevel<'_> {
    fn set(&mut self, pos: [i32; 3], state: &State) -> bool {
        let Some(id) = self.host.block_state_id(&state.text()) else {
            return false;
        };
        self.level
            .set_block(
                self.host,
                pos,
                id,
                BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS,
            )
            .unwrap_or(false)
    }

    fn destroy(&mut self, pos: [i32; 3]) {
        let _ = self.level.destroy_block(self.host, pos, true, None, false);
    }

    fn powered(&self, pos: [i32; 3]) -> bool {
        self.level
            .best_neighbor_signal(&*self.host, pos)
            .unwrap_or(0)
            > 0
    }

    fn default_state(&self, block: &str) -> Option<State> {
        let id = self.host.block_state_id(block)?;
        State::parse(&self.host.block_state_string(id)?)
    }
}

/// A level read from a hook that only reads.
struct Reading<'a>(&'a dyn LevelHost);

impl Reader for Reading<'_> {
    fn state(&self, pos: [i32; 3]) -> Option<State> {
        let state = self.0.block_state(pos)?;
        State::parse(&self.0.state_string(state)?)
    }
}

fn block_id(host: &dyn Host, stack: Option<&mod_api::ItemStack>) -> Option<String> {
    Some(stack?.item().id(host)?.to_owned())
}

fn sneaking(host: &dyn Host, player: PlayerRef) -> bool {
    player.entity().is_shift_key_down(host).unwrap_or(false)
}

/// The wand in a player's off hand (inventory slot 40).
fn offhand_wand(host: &dyn Host, player: PlayerRef) -> Option<FireType> {
    player
        .with_inventory(host, |view| {
            let item = view.get(40)?.item();
            FireType::of_wand(item.id(host)?)
        })
        .flatten()
}

/// The player a placement came from, if a player placed it.
fn placer(host: &mut dyn ServerHost, event: &SetPlacedByEvent<'_>) -> Option<PlayerRef> {
    let uuid = event.placer?.uuid(host)?;
    host.player(uuid)
}

fn center([x, y, z]: [i32; 3]) -> [f64; 3] {
    [f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5]
}

fn play(
    host: &mut dyn ServerHost,
    level: LevelRef,
    sound: &str,
    at: [f64; 3],
    source: SoundSource,
    volume: f32,
) {
    let _ = level.play_sound(host, sound, at, source, volume, 1.0, None);
}

/// A wand used on a block: the change's sound at the player, as `Wand.playSound` plays it.
fn wand_sound(
    host: &mut dyn ServerHost,
    level: LevelRef,
    player: Option<PlayerRef>,
    sound: pedestal::WandSound,
) {
    let Some(at) = player.and_then(|p| p.entity().block_position(&*host)) else {
        return;
    };
    play(
        host,
        level,
        sound.id(),
        center(at),
        SoundSource::Players,
        sound.volume(),
    );
}

/// Comparators beside a pedestal read it again (`Level.updateNeighbourForOutputSignal`).
fn update_comparators(host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3]) {
    if let Some(state) = level.block_state(&*host, pos) {
        let _ = level.update_neighbors(host, pos, state, NeighbourUpdate::OutputSignal);
    }
}

/// A wand click on the pedestal at `pos`, for the pedestal itself or the fire above it.
fn wand_on_pedestal(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    wand: FireType,
    player: Option<PlayerRef>,
) {
    let sneak = player.is_some_and(|p| sneaking(&*host, p));
    let sound = pedestal::wand_on_pedestal(&mut ServerLevel { host, level }, pos, wand, sneak);
    if let Some(sound) = sound {
        wand_sound(host, level, player, sound);
    }
    update_comparators(host, level, pos);
}

/// The 96 standing torches and fire pits.
pub struct Pedestal;

#[mod_api::sunrise_part]
impl BlockPart for Pedestal {
    /// `Pedestal.useItemOn`, and a wand's `use` aimed at the pedestal.
    fn use_item_on(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseItemOnEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = event.click;
        let level = host.level(click.dimension.key().id())?;
        let player = host.player(click.player);
        if let Some(wand) = block_id(&*host, click.stack).and_then(|id| FireType::of_wand(&id)) {
            wand_on_pedestal(host, level, click.pos, wand, player);
            return Some(InteractionResult::Success);
        }
        let mut world = ServerLevel { host, level };
        let state = world.state(click.pos)?;
        let kind = Kind::of_pedestal(&state.block)?;
        if !pedestal::ignite_by_interaction(&mut world, click.pos, state) {
            return None;
        }
        update_comparators(host, level, click.pos);
        play(
            host,
            level,
            kind.ignition_sound(),
            center(click.pos),
            SoundSource::Blocks,
            kind.ignition_volume(),
        );
        Some(InteractionResult::Success)
    }

    /// `Pedestal.setPlacedBy`.
    fn set_placed_by(&self, host: &mut dyn ServerHost, event: &mut SetPlacedByEvent<'_>) {
        let Some(player) = placer(host, event) else {
            return;
        };
        let (sneak, wand) = (sneaking(&*host, player), offhand_wand(&*host, player));
        let level = event.level;
        let mut world = ServerLevel { host, level };
        if let Some(state) = world.state(event.pos) {
            pedestal::placed(&mut world, event.pos, state, sneak, wand);
        }
        update_comparators(host, level, event.pos);
    }

    /// `Pedestal.neighborChanged`: settled after the level's tick.
    fn neighbor_changed(&self, _: &mut dyn LevelTicksHost, event: &mut NeighborChangedEvent) {
        let pos = [event.pos.x, event.pos.y, event.pos.z];
        let mut queue = queue();
        let level = queue
            .entry(event.dimension.key().id().to_owned())
            .or_default();
        if level.len() < MAX_QUEUED {
            level.insert(pos);
        }
    }

    fn has_analog_output(&self, _: &dyn LevelHost, event: &mut HasAnalogOutputEvent) {
        event.output = true;
    }

    /// `Pedestal.getAnalogOutputSignal`.
    fn analog_output(&self, level: &mut dyn LevelRandomHost, event: &mut AnalogOutputEvent) {
        let pos = [event.pos.x, event.pos.y, event.pos.z];
        let reading = Reading(&*level);
        if let Some(state) = reading.state(pos) {
            event.output = pedestal::comparator_output(&reading, pos, &state);
        }
    }
}

/// The twelve fires that sit in the pedestals.
pub struct Fire {
    smoke: [ParticleKey; 2],
}

#[mod_api::sunrise_part]
impl BlockPart for Fire {
    /// `FireBase.updateShape`: a fire with nothing under it (or water) goes out.
    fn update_shape(
        &self,
        level: &mut dyn LevelTicksHost,
        event: &mut UpdateShapeEvent,
    ) -> Option<mod_api::BlockState> {
        if event.direction != Direction::Down {
            return None;
        }
        if pedestal::fire_supported(level.state_block(event.neighbour)?) {
            return None;
        }
        level.block_state_id("minecraft:air")
    }

    /// `FireBase.setPlacedBy`.
    fn set_placed_by(&self, host: &mut dyn ServerHost, event: &mut SetPlacedByEvent<'_>) {
        let mut world = ServerLevel {
            host,
            level: event.level,
        };
        if let Some(state) = world.state(event.pos) {
            pedestal::fire_placed(&mut world, event.pos, &state);
        }
    }

    /// A wand's `use` aimed at a fire changes the pedestal under it.
    fn use_item_on(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseItemOnEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = event.click;
        let wand = block_id(&*host, click.stack).and_then(|id| FireType::of_wand(&id))?;
        let level = host.level(click.dimension.key().id())?;
        let [x, y, z] = click.pos;
        let below = [x, y - 1, z];
        let state = ServerLevel { host, level }.state(below)?;
        Kind::of_pedestal(&state.block)?;
        let player = host.player(click.player);
        wand_on_pedestal(host, level, below, wand, player);
        Some(InteractionResult::Success)
    }

    /// `FireBase.animateTick`: smoke above the fire; a light block has none.
    fn animate_tick(&self, host: &mut dyn ClientHost, event: &mut AnimateTickEvent<'_>) {
        let Some(block) = event.state.block().id(&*host) else {
            return;
        };
        let Some((fire, kind)) = pedestal::of_fire(block) else {
            return;
        };
        if fire == FireType::Light {
            return;
        }
        let smoke = self.smoke[usize::from(kind.smoke_particle() == "minecraft:large_smoke")];
        let p = event.pos;
        let at = [
            f64::from(p.x) + 0.5,
            f64::from(p.y) + kind.smoke_height(),
            f64::from(p.z) + 0.5,
        ];
        host.add_particle(&ParticleOptions::simple(smoke), at, [0.0; 3]);
    }
}

/// The floor and wall torches: the soul wand turns their flame.
pub struct Torch;

#[mod_api::sunrise_part]
impl BlockPart for Torch {
    fn use_item_on(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseItemOnEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = event.click;
        let wand = block_id(&*host, click.stack).and_then(|id| FireType::of_wand(&id))?;
        let level = host.level(click.dimension.key().id())?;
        let player = host.player(click.player);
        let sneak = player.is_some_and(|p| sneaking(&*host, p));
        let sound =
            pedestal::wand_on_torch(&mut ServerLevel { host, level }, click.pos, wand, sneak)?;
        wand_sound(host, level, player, sound);
        Some(InteractionResult::Success)
    }

    /// `ALTorch.setPlacedBy`: a soul wand in the off hand places a soul torch.
    fn set_placed_by(&self, host: &mut dyn ServerHost, event: &mut SetPlacedByEvent<'_>) {
        let Some(player) = placer(host, event) else {
            return;
        };
        if offhand_wand(&*host, player) != Some(FireType::Soul) {
            return;
        }
        let mut world = ServerLevel {
            host,
            level: event.level,
        };
        if let Some(state) = world.state(event.pos) {
            let mut next = state.clone();
            next.props
                .insert("firetype".into(), FireType::Soul.name().into());
            world.set(event.pos, &next);
        }
    }
}

/// The floor torches: placed against a wall, the item puts down the wall torch.
pub struct FloorTorch;

#[mod_api::sunrise_part]
impl BlockPart for FloorTorch {
    /// `StandingAndWallBlockItem.getPlacementState`.
    fn state_for_placement(
        &self,
        level: LevelSide,
        event: &mut StateForPlacementEvent<'_>,
    ) -> Option<Option<mod_api::BlockState>> {
        let (LevelSide::Server(level) | LevelSide::Client(level)) = level;
        let material = level
            .state_block(event.state)?
            .strip_prefix(MOD)?
            .strip_prefix(":al_torch_")?
            .to_owned();
        let ctx = event.context;
        let pos = [ctx.pos.x, ctx.pos.y, ctx.pos.z];
        let sturdy = |at: [i32; 3], face: BlockFace, support: SupportType| {
            level.block_face_sturdy(at, face, support).unwrap_or(false)
        };
        let looking = torch::nearest_looking(ctx.yaw, ctx.pitch, ctx.face, ctx.replace_clicked);
        let chosen = torch::placement(
            looking,
            // TorchBlock.canSurvive: the block below holds its centre.
            || {
                sturdy(
                    [pos[0], pos[1] - 1, pos[2]],
                    BlockFace::Up,
                    SupportType::Center,
                )
            },
            // WallTorchBlock.canSurvive: the block behind has a full face toward it.
            |facing| {
                let [dx, dy, dz] = facing.opposite().normal();
                sturdy(
                    [pos[0] + dx, pos[1] + dy, pos[2] + dz],
                    face_of(facing),
                    SupportType::Full,
                )
            },
        );
        Some(match chosen {
            Some(torch::Placement::Floor) => Some(event.state),
            Some(torch::Placement::Wall(facing)) => level.block_state_id(&format!(
                "{MOD}:al_wall_torch_{material}[facing={}]",
                torch::facing_name(facing)
            )),
            None => None,
        })
    }
}

fn face_of(d: Direction) -> BlockFace {
    match d {
        Direction::Down => BlockFace::Down,
        Direction::Up => BlockFace::Up,
        Direction::North => BlockFace::North,
        Direction::South => BlockFace::South,
        Direction::West => BlockFace::West,
        Direction::East => BlockFace::East,
    }
}

/// How many pedestals one level may keep queued; a `/fill` queues one per block.
const MAX_QUEUED: usize = 8192;

fn queue() -> MutexGuard<'static, BTreeMap<String, BTreeSet<[i32; 3]>>> {
    static QUEUE: Mutex<BTreeMap<String, BTreeSet<[i32; 3]>>> = Mutex::new(BTreeMap::new());
    QUEUE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Settle the pedestals of the level that just ticked: redstone lights or puts them out.
pub fn on_level_tick_post(host: &mut dyn ServerHost, level: LevelRef) {
    let Some(dimension) = level.dimension().id(&*host) else {
        return;
    };
    let Some(waiting) = queue().remove(dimension) else {
        return;
    };
    for pos in waiting {
        let mut world = ServerLevel { host, level };
        let Some(kind) = world.state(pos).and_then(|s| Kind::of_pedestal(&s.block)) else {
            continue;
        };
        if pedestal::handle_neighbor_update(&mut world, pos) {
            play(
                host,
                level,
                kind.ignition_sound(),
                center(pos),
                SoundSource::Blocks,
                kind.ignition_volume(),
            );
        }
        update_comparators(host, level, pos);
    }
}

/// Attach the parts to the blocks the package declares.
pub fn declare(r: &mut dyn Registrar) {
    for kind in Kind::ALL {
        for material in crate::MATERIALS {
            let block = r.block_key(&format!("{MOD}:{}_{material}", kind.name()));
            r.block_part(block, Pedestal);
        }
        for fire in [FireType::Normal, FireType::Soul, FireType::Light] {
            let block = r.block_key(&pedestal::fire_block(kind, fire));
            let smoke = [
                r.particle_key("minecraft:smoke"),
                r.particle_key("minecraft:large_smoke"),
            ];
            r.block_part(block, Fire { smoke });
        }
    }
    for material in crate::TORCH_MATERIALS {
        for torch in ["al_torch", "al_wall_torch"] {
            let block = r.block_key(&format!("{MOD}:{torch}_{material}"));
            r.block_part(block, Torch);
            if torch == "al_torch" {
                r.block_part(block, FloorTorch);
            }
        }
    }
}
