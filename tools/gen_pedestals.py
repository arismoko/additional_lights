#!/usr/bin/env python3
"""Generates the standing torches, fire pits, their fires and the two wands from the original.

usage: python3 tools/gen_pedestals.py <AdditionalLights checkout> <datagen output>

<datagen output> is the folder the original's `tools/datagen.py --gen-dir` wrote (its Java
templates are not needed). From it come the blockstates, block models and item definitions; from
the original's resources the model templates, textures, sounds, recipes, recipe advancements and
strings. This writes:

- 96 pedestals (`standing_torch_s_*`, `standing_torch_l_*`, `fire_pit_s_*`, `fire_pit_l_*`), each
  on its material block as template (the original copies the material's properties), with the
  original's shapes, no light of its own, and the state properties `Pedestal` declares;
- 12 fires (`fire_for_*`, `soul_fire_for_*`, `light_fire_for_*`), without collision, broken
  instantly, with the original's outline and light, dropping themselves only when placed by hand;
- the soul wand and the light wand (one to a stack, as in the original);
- their recipes (the fires' are off by default in the original, so they are left out), recipe
  advancements, tags, sounds, creative tab entries and strings in every language the original has.
"""
import csv, json, shutil, sys
from pathlib import Path

ORIG, GEN = Path(sys.argv[1]), Path(sys.argv[2])
ROOT = Path(__file__).resolve().parent.parent
MOD = 'additional_lights'
RES = ORIG / 'common/src/main/resources'
GEN_ASSETS = GEN / 'resources/assets' / MOD
ASSETS = ROOT / 'assets' / MOD
DATA = ROOT / 'data'

KINDS = ['standing_torch_s', 'standing_torch_l', 'fire_pit_s', 'fire_pit_l']
FIRES = ['fire', 'soul_fire', 'light_fire']
FIRE_TYPES = ['normal', 'soul', 'light']

# The *Spec boxes, in 1/16 block.
PEDESTAL_BOXES = {
    'standing_torch_s': [[4, 0, 4, 12, 2, 12], [6, 2, 6, 10, 8, 10], [4, 8, 4, 12, 12, 12]],
    'standing_torch_l': [[4, 0, 4, 12, 2, 12], [5, 0, 5, 11, 4, 11], [6, 4, 6, 10, 12, 10],
                         [5, 12, 5, 11, 14, 11], [4, 14, 4, 12, 16, 12]],
    'fire_pit_s': [[2, 0, 2, 14, 4, 14], [0, 4, 0, 16, 8, 16]],
    'fire_pit_l': [[0, 0, 0, 16, 6, 16], [2, 6, 2, 14, 10, 14], [0, 10, 0, 16, 16, 16]],
}
# FireBaseSpec.SHAPES_COORDS and LightFireShapeSpec.
FIRE_BOX = {'standing_torch_s': [4, -6, 4, 12, 2, 12], 'standing_torch_l': [4, -2, 4, 12, 6, 12],
            'fire_pit_s': [0, -10, 0, 16, 2, 16], 'fire_pit_l': [0, -2, 0, 16, 7, 16]}
LIGHT_FIRE_BOX = {'standing_torch_s': [5, -4, 5, 11, -2, 11], 'standing_torch_l': [5, 0, 5, 11, 3, 11],
                  'fire_pit_s': [2, -10, 2, 14, -4.75, 14], 'fire_pit_l': [1, -2, 1, 15, 4.5, 15]}
# Fire, Fire_Soul and Fire_Light: MapColor.FIRE, COLOR_LIGHT_BLUE and SNOW.
FIRE_MAP_COLOR = {'fire': 4, 'soul_fire': 3, 'light_fire': 8}
FIRE_SOUNDS = {'fire': 'minecraft:white_wool', 'soul_fire': 'minecraft:white_wool', 'light_fire': 'minecraft:white_wool'}


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')


def copy(src, dst):
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(src, dst)


def boxes(rows):
    return [[v / 16 for v in row] for row in rows]


def flag(name):
    return {name: {'type': 'boolean'}}


def loot(item, when=None):
    pool = {'rolls': 1, 'entries': [{'type': 'minecraft:item', 'name': item}],
            'conditions': [{'condition': 'minecraft:survives_explosion'}]}
    if when:
        pool['conditions'].insert(0, {'condition': 'minecraft:block_state_property',
                                      'block': item, 'properties': when})
    return {'type': 'minecraft:block', 'pools': [pool], 'random_sequence': f'{MOD}:blocks/{item.split(":")[1]}'}


def model_tree(model_id, out):
    """`model_id` and its own parents, from the datagen output or the original's resources."""
    while model_id and model_id.startswith(f'{MOD}:'):
        rel = model_id.split(':')[1]
        src = GEN_ASSETS / 'models' / f'{rel}.json'
        if not src.exists():
            src = RES / 'assets' / MOD / 'models' / f'{rel}.json'
        out[rel] = src
        model_id = json.loads(src.read_text()).get('parent')


def main():
    rows = [r['reg_name'] for r in csv.DictReader((ORIG / 'tools/block_definitions.csv').open())]
    pedestals = [(r, k) for r in rows for k in KINDS if r.startswith(k + '_')]
    materials = sorted({r[len(k) + 1:] for r, k in pedestals})
    assert len(pedestals) == 96 and len(materials) == 24, (len(pedestals), len(materials))
    fires = [(f'{f}_for_{k}', f, k) for f in FIRES for k in KINDS]

    models = {}
    for reg, kind in pedestals:
        material = reg[len(kind) + 1:]
        write(DATA / MOD / 'sunrise/block' / f'{reg}.json', {
            'template': f'minecraft:{material}',
            'properties': {'light': 0},
            'shapes': {'default': boxes(PEDESTAL_BOXES[kind])},
            'state_properties': {
                **flag('waterlogged'),
                'firetype': {'type': 'enum', 'values': FIRE_TYPES},
                'previous_firetype': {'type': 'enum', 'values': FIRE_TYPES},
                **flag('accept_power'), **flag('ispowered'), **flag('activated'), **flag('lit'),
            },
            'default_state': {'waterlogged': 'false', 'firetype': 'normal', 'previous_firetype': 'normal',
                              'accept_power': 'true', 'ispowered': 'false', 'activated': 'false', 'lit': 'false'},
            'item': True,
        })
        write(DATA / MOD / 'loot_table/blocks' / f'{reg}.json', loot(f'{MOD}:{reg}'))
    for reg, fire, kind in fires:
        decl = {
            'template': 'minecraft:stone',
            'properties': {'hardness': 0.0, 'explosion_resistance': 0.0, 'requires_tool': False,
                           'light': 10 if fire == 'soul_fire' else 15, 'map_color': FIRE_MAP_COLOR[fire],
                           'sounds': FIRE_SOUNDS[fire], 'occludes': False},
            'shapes': {'default': boxes([(LIGHT_FIRE_BOX if fire == 'light_fire' else FIRE_BOX)[kind]])},
            'collision_shapes': {'default': []},
            'state_properties': {**flag('set'), **flag('summoned'), **flag('temp')},
            'default_state': {'set': 'false', 'summoned': 'false', 'temp': 'false'},
            # BlockSpec.getItemType: a light fire has no item.
            'item': fire != 'light_fire',
        }
        if fire == 'light_fire':
            decl['state_properties'].update(flag('lit'))
            decl['default_state']['lit'] = 'true'
            decl['state_light'] = {'properties': ['lit'], 'values': {'true': 15, 'false': 0}, 'default': 15}
        write(DATA / MOD / 'sunrise/block' / f'{reg}.json', decl)
        # FireBase.getDrops: nothing for a fire a pedestal summoned or a temporary one.
        if fire != 'light_fire':
            write(DATA / MOD / 'loot_table/blocks' / f'{reg}.json',
                  loot(f'{MOD}:{reg}', {'summoned': 'false', 'temp': 'false'}))
    for wand in ('soul_wand', 'light_wand'):
        write(DATA / MOD / 'sunrise/item' / f'{wand}.json', {
            'template': 'minecraft:stick',
            'max_stack_size': 1,
            'components': {'!minecraft:cooking_fuel': {}},
        })
        copy(RES / 'assets' / MOD / 'items' / f'{wand}.json', ASSETS / 'items' / f'{wand}.json')
        model_tree(f'{MOD}:item/{wand}', models)
        copy(RES / 'assets' / MOD / 'textures/item' / f'{wand}.png', ASSETS / 'textures/item' / f'{wand}.png')

    with_items = [(r, k) for r, k in pedestals] + [(r, k) for r, f, k in fires if f != 'light_fire']
    for reg in [r for r, _ in pedestals] + [r for r, _, _ in fires]:
        copy(GEN_ASSETS / 'blockstates' / f'{reg}.json', ASSETS / 'blockstates' / f'{reg}.json')
        for ref in (json.loads((GEN_ASSETS / 'blockstates' / f'{reg}.json').read_text())['variants'].values()):
            model_tree(ref['model'], models)
    for reg, kind in with_items:
        # datagen.resolve_item_model_id: the block's own model.
        model = f'{MOD}:block/{"fire" if reg.startswith(("fire_for_", "soul_fire_for_")) else kind}/{reg}'
        write(ASSETS / 'items' / f'{reg}.json', {'model': {'type': 'minecraft:model', 'model': model}})
        model_tree(model, models)
    textures = set()
    for rel, src in models.items():
        copy(src, ASSETS / 'models' / f'{rel}.json')
        for value in json.loads(src.read_text()).get('textures', {}).values():
            sprite = value.get('sprite') if isinstance(value, dict) else value
            if sprite.startswith(f'{MOD}:'):
                textures.add(sprite.split(':')[1])
    for rel in textures:
        for suffix in ('.png', '.png.mcmeta'):
            src = RES / 'assets' / MOD / 'textures' / f'{rel}{suffix}'
            if src.exists():
                copy(src, ASSETS / 'textures' / f'{rel}{suffix}')

    copy(RES / 'assets' / MOD / 'sounds.json', ASSETS / 'sounds.json')
    for ogg in (RES / 'assets' / MOD / 'sounds').glob('*.ogg'):
        copy(ogg, ASSETS / 'sounds' / ogg.name)

    for kind in KINDS:
        for src in (RES / 'data' / MOD / 'recipe' / kind).glob('*.json'):
            copy(src, DATA / MOD / 'recipe' / kind / src.name)
    for wand in ('soul_wand', 'light_wand'):
        copy(RES / 'data' / MOD / 'recipe' / f'{wand}.json', DATA / MOD / 'recipe' / f'{wand}.json')
        copy(RES / 'data' / MOD / 'advancement/recipes/al_wand' / f'{wand}.json',
             DATA / MOD / 'advancement/recipes/al_wand' / f'{wand}.json')
    for material in materials:
        copy(RES / 'data' / MOD / 'advancement/recipes/materials' / material / 'coal.json',
             DATA / MOD / 'advancement/recipes/materials' / material / 'coal.json')

    # The original's block tags, for the blocks this adds.
    ours = {f'{MOD}:{r}' for r, _ in pedestals}
    for tag in ('mineable/pickaxe', 'needs_stone_tool', 'needs_iron_tool'):
        values = [v for v in json.loads((RES / 'data/minecraft/tags/block' / f'{tag}.json').read_text())['values'] if v in ours]
        path = DATA / 'minecraft/tags/block' / f'{tag}.json'
        existing = json.loads(path.read_text())['values'] if path.exists() else []
        write(path, {'replace': False, 'values': existing + [v for v in values if v not in existing]})
    path = DATA / 'minecraft/tags/block/washed_away_by_fluids.json'
    tag = json.loads(path.read_text())
    tag['values'] += [f'{MOD}:{r}' for r, _, _ in fires if f'{MOD}:{r}' not in tag['values']]
    write(path, tag)

    tab_path = DATA / MOD / 'sunrise/creative_tab/tab.json'
    tab = json.loads(tab_path.read_text())
    for item in [r for r, _ in with_items] + ['soul_wand', 'light_wand']:
        if f'{MOD}:{item}' not in tab['items']:
            tab['items'].append(f'{MOD}:{item}')
    write(tab_path, tab)

    names = {f'block.{MOD}.{r}' for r, _ in pedestals} | {f'block.{MOD}.{r}' for r, _, _ in fires}
    names |= {f'item.{MOD}.soul_wand', f'item.{MOD}.light_wand'}
    for src in (RES / 'assets' / MOD / 'lang').glob('*.json'):
        upstream = json.loads(src.read_text(encoding='utf-8'))
        dst = ASSETS / 'lang' / src.name
        lang = json.loads(dst.read_text(encoding='utf-8')) if dst.exists() else {}
        for key in sorted(names):
            if key in upstream:
                lang[key] = upstream[key]
        write(dst, lang)
    print(f'{len(pedestals)} pedestals, {len(fires)} fires, 2 wands, {len(models)} models, {len(textures)} textures')


if __name__ == '__main__':
    main()
