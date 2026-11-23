# Additional Lights for Sunrise

Lamps, floor torches and wall torches made from 28 vanilla materials.

A port of [Additional Lights](https://github.com/mgen256/AdditionalLights) by mgen256 to [Sunrise](https://playsunrise.net) (Minecraft Java 26.3), as a content mod: the original's blocks, models, textures, recipes and strings, with no code.

## What it does

- 24 lamps (light 15), placed on any face like an end rod.
- 28 floor torches and 28 wall torches (light 14) in stone, wood, metal, wool, glass and ice materials.
- Each torch's light follows its fire type as in the original: 14 for normal fire, 10 for soul fire and 15 for light.
- The original's recipes (glowstone over the material for lamps), drops and creative tab, in every language it ships.

## Differences from the original

- The standing torches, fire pits, fires and the fire-type, soul and light wands are not ported yet.
- A floor torch item places only the floor torch; place the wall torch from its own block (a content mod can't give a torch item a wall form yet).

## Building

With the Sunrise mod tool (`cargo sunrise-mod`, from the Sunrise SDK):

```sh
cargo sunrise-mod check
cargo sunrise-mod build    # target/sunrise-mod/additional_lights.smod
```

This version is made for the Sunrise Dev build `dev-2026-11-22-7cfdf81`.

## Licence and credits

Additional Lights is by mgen256 (https://github.com/mgen256/AdditionalLights). Its textures, models, recipes and strings are mgen256's and are used here under the **MIT licence**, the original's; `LICENSE` holds the text and its copyright notice, and `LICENSE-NOTICE.md` says what this port carries.

Carried to Minecraft 26.3 by AdLf0208 ([AdLf0208/additional_lights](https://github.com/AdLf0208/additional_lights)), and ported to Sunrise by arismoko. This port is distributed under the same licence.
