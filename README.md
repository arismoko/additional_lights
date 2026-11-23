# Additional Lights for Sunrise

Lamps, torches, standing torches and fire pits made from vanilla materials, with soul and light wands.

A port of [Additional Lights](https://github.com/mgen256/AdditionalLights) by mgen256 to [Sunrise](https://playsunrise.net) (Minecraft Java 26.3): the original's blocks, models, textures, sounds, recipes and strings, with its fire rules rewritten in Rust.

## What it does

- 24 lamps (light 15), placed on any face like an end rod.
- 28 torches (light 14) in stone, wood, metal, wool, glass and ice materials, standing on the floor or hung on a wall as you place them.
- 96 standing torches and fire pits in two sizes and 24 materials. A placed one lights at once; right-click relights it; placed while sneaking it stays unlit and ignores redstone.
- Redstone lights a standing torch or fire pit and puts it out again, and a comparator reads its fire: 6 for fire, 3 for soul fire, 9 for light.
- The soul wand turns a fire, standing torch or floor torch to soul fire, and the light wand turns a standing torch or fire pit into a light block; sneak and right-click to turn it back. Held in the off hand, a wand sets what you place.
- The original's recipes, drops, sounds and creative tab, in every language it ships.

## Differences from the original

- Redstone lights a standing torch or fire pit at the end of the tick its signal changes.
- Breaking a light block above a standing torch or fire pit breaks the light, not the pedestal under it.
- Items show no tooltips, and fires use the wool sounds rather than the extinguish sound when broken.

## Building

With the Sunrise mod tool (`cargo sunrise-mod`, from the Sunrise SDK) and Rust:

```sh
cargo sunrise-mod build    # compiles against the pinned game, packs target/sunrise-mod/additional_lights.smod
cargo sunrise-mod run      # builds it and starts the game with it
cargo test
```

`sunrise-mod.toml` pins the Sunrise Dev build `dev-2026-11-22-7cfdf81` (game commit `7cfdf81463`). Sunrise builds native mods from source on the player's computer, against the exact game build.

## Licence and credits

Additional Lights is by mgen256 (https://github.com/mgen256/AdditionalLights). Its textures, models, recipes and strings are mgen256's and are used here under the **MIT licence**, the original's; `LICENSE` holds the text and its copyright notice, and `LICENSE-NOTICE.md` says what this port carries.

Carried to Minecraft 26.3 by AdLf0208 ([AdLf0208/additional_lights](https://github.com/AdLf0208/additional_lights)), and ported to Sunrise by arismoko. This port is distributed under the same licence.
