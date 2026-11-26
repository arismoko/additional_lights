//! The material colours behind the original's colour sort (`CreativeTabOrderToggleBehavior`):
//! a material's colour is the average of its blocks-atlas sprite, read on the CPU
//! (`RenderAssets::atlas_sprite_pixels`, sunrise#4601). The averaging and the sort key are plain
//! data, so `cargo test` checks them without a game.

use mod_api::{RenderAssets, SpriteRef};

/// The texture a material's pedestals and torches are named after, as `texture_sprites` takes it.
pub fn material_texture(material: &str) -> String {
    let name = match material {
        "quartz_block" => "quartz_block_side",
        other => other,
    };
    format!("minecraft:block/{name}")
}

/// The average colour (`0xRRGGBB`) of a sprite's RGBA8 pixels, each weighted by its alpha so
/// clear pixels (glass, cut-outs) don't darken it. None for a sprite with no visible pixel or
/// whose buffer is not `width * height * 4` bytes.
pub fn average_colour(width: u32, height: u32, rgba: &[u8]) -> Option<u32> {
    let expected = u64::from(width)
        .checked_mul(u64::from(height))?
        .checked_mul(4)?;
    if u64::try_from(rgba.len()).ok()? != expected {
        return None;
    }
    let (mut r, mut g, mut b, mut weight) = (0u64, 0u64, 0u64, 0u64);
    for px in rgba.chunks_exact(4) {
        let a = u64::from(px[3]);
        r += u64::from(px[0]) * a;
        g += u64::from(px[1]) * a;
        b += u64::from(px[2]) * a;
        weight += a;
    }
    if weight == 0 {
        return None;
    }
    let channel = |sum: u64| u32::try_from(sum / weight).unwrap_or(255);
    Some(channel(r) << 16 | channel(g) << 8 | channel(b))
}

/// A material's colour read from its blocks-atlas sprite: its first frame, untinted. None when
/// the atlas does not hold the sprite yet or the sprite has no visible pixel.
pub fn sprite_colour(assets: &dyn RenderAssets, sprite: SpriteRef) -> Option<u32> {
    let pixels = assets.atlas_sprite_pixels(sprite).ok()?;
    average_colour(pixels.width, pixels.height, &pixels.rgba)
}

/// The colour sort's key: greys first by brightness, then the colours by hue (whole degrees,
/// 0 to 359) and brightness.
pub fn sort_key(colour: u32) -> (u8, u32, u32) {
    let r = i32::from((colour >> 16) as u8);
    let g = i32::from((colour >> 8) as u8);
    let b = i32::from(colour as u8);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let brightness = u32::try_from(r + g + b).unwrap_or(0);
    if max == min {
        return (0, 0, brightness);
    }
    let span = max - min;
    let sector = if max == r {
        (g - b) * 60 / span
    } else if max == g {
        120 + (b - r) * 60 / span
    } else {
        240 + (r - g) * 60 / span
    };
    (
        1,
        u32::try_from(sector.rem_euclid(360)).unwrap_or(0),
        brightness,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_average_is_weighted_by_alpha() {
        let px = [255, 0, 0, 255, 0, 255, 0, 0];
        assert_eq!(average_colour(2, 1, &px), Some(0xff0000));
        let px = [200, 100, 0, 255, 100, 50, 0, 255];
        assert_eq!(average_colour(1, 2, &px), Some(0x963200));
    }

    #[test]
    fn a_clear_or_misshapen_sprite_has_no_colour() {
        assert_eq!(average_colour(1, 1, &[9, 9, 9, 0]), None);
        assert_eq!(average_colour(2, 2, &[0; 4]), None);
        assert_eq!(average_colour(u32::MAX, u32::MAX, &[]), None);
    }

    #[test]
    fn greys_sort_before_colours_and_colours_by_hue() {
        assert!(sort_key(0x202020) < sort_key(0x808080));
        assert!(sort_key(0x808080) < sort_key(0xff0000));
        assert!(sort_key(0xff0000) < sort_key(0x00ff00));
        assert!(sort_key(0x00ff00) < sort_key(0x0000ff));
        assert_eq!(sort_key(0xff00ff).1, 300);
    }

    #[test]
    fn quartz_uses_its_side_texture() {
        assert_eq!(material_texture("stone"), "minecraft:block/stone");
        assert_eq!(
            material_texture("quartz_block"),
            "minecraft:block/quartz_block_side"
        );
    }
}
