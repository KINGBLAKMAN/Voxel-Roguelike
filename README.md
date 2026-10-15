# Emberwild

A voxel horde-survival roguelike in the spirit of Vampire Survivors. Pick one of six classes and fight through three five-minute stages, each on its own walled-in map, against a horde that rises from the ground all around you and never stops growing. Each stage ends with its boss, whose death opens a portal to the next map; the Ember King waits on the last one. Level up, stack your abilities without limit, evolve relics, dodge and parry, swim, and take on events. The gold you bring home buys cosmetics at the Camp: all power is earned inside the run. Play alone or with up to three friends.

## Play

Emberwild is a native Windows game written in Rust on the [Bevy](https://bevyengine.org) engine. It opens fullscreen; set `EMBERWILD_WINDOWED=1` to run it in a window. The pause menu has **Quit to desktop**.

**Download:** each release is a pair of zips: `Emberwild-<version>-windows.zip` holds the game (a single exe, no installer), and `Emberwild-<version>-windows-dev.zip` holds the same game plus the developer tools (the balance bot and the scripted test hooks). To build them yourself, run `rust/build-windows.sh`, which cross-compiles both exes and writes the zips to `rust/dist/`.

**From source:** install Rust, then `cd rust && cargo run --release`. On Linux you need the usual Bevy packages (`libasound2-dev libudev-dev libxkbcommon-x11-0`).

The original browser version is still in `index.html` and `src/`, with its Electron wrapper in `desktop/`.

## Controls

| Key | Action |
|---|---|
| WASD | Move |
| Mouse | Aim |
| Left mouse | Attack (hold to draw the bow) |
| 1 to 4 | Abilities |
| Q | Swap weapon |
| 1 to 3, R | On level-up: pick an upgrade, reroll |
| Shift | Sprint |
| Space | Jump or swim |
| Ctrl or Alt | Dodge roll (perfectly timed: refunds stamina and guarantees crits briefly) |
| Right mouse | Block; raise it just before a hit to parry |
| E | Interact: chests, shrines, altars, totems, the merchant; hold to revive a teammate |
| F | Eat |
| Esc | Pause |
| M | Mute |
| Tab | Character panel: stats, abilities and every upgrade as icons (hover for details) |
| C, S, H, J | On the title screen: Camp, Settings, Host co-op, Join co-op |

Gamepads work too: left stick moves, right stick looks, RT attacks, LT blocks, A jumps, B dodges, X, Y, LB and RB cast, and D-pad right opens the character panel. Hover an ability slot whenever the cursor is free (pause, level-up, character panel) for a tooltip with its cooldown and upgrades. Sensitivity, volume, field of view, view distance, shadows, screen shake and more are under **Settings**.

## Runs, maps and bosses

- **Three stages, five minutes each.** Every stage is a procedurally generated map walled in by cliffs, and when its five minutes are up its boss arrives. Killing the boss opens a **portal**; step in (or wait 40 seconds) and the whole party moves to the next map with everything they've earned.
  1. **The Greenwood:** forests and meadows, ending with the **Ancient Golem**.
  2. Your choice on the title screen (**Route**, or press R): **The Drowned Fen**, a swamp of pools, mud, willows and green mist, ending with the **Broodmother**; or **The Bone Wastes**, dunes, terraced mesas and oases, ending with the **Bone Colossus**. Random picks one for you.
  3. **The Ember Throne:** basalt and ash cut by rivers of lava under a red sky, ending with the three-phase **Ember King**. Killing him wins the run, after which you can keep going in endless mode.
- Each map has its own monsters: spiders, slimes and bloaters in the fen, skeletons and sand wurms in the wastes, imps and cultists on the volcano.
- The horde's size and toughness climb all run long and carry across maps: the third map starts where the second left off.
- Something happens every 70 to 100 seconds: a timed challenge, a cursed shrine (a minute of a nastier horde in exchange for a relic), a war totem to defend, or a treasure goblin that drips gold as it flees. A **wandering merchant** visits every few minutes and sells relic caches, healing, rerolls and more for run gold.
- **Relic evolutions:** a max-rank relic plus the matching boon evolves into something much stronger (Storm of Swords, Storm Throne, Heart of the Sun, Choir of Spirits, Reaper's Cyclone, Inferno Cask, Absolute Winter).
- **Ember 1 to 10:** each win unlocks the next difficulty level, and each level adds a twist. Pick it on the title screen.

## The Camp: cosmetics only

Every class and relic is open from the start, and nothing you buy makes you stronger. Runs pay out gold, and the Camp's wardrobe spends it on 38 cosmetics, shown on a turning mannequin:

- **Outfits:** three recolours per class (for example the Ranger's Nightshade, the Mage's Frostweave, the Warrior's Gilded Plate, the Necromancer's Plague Doctor, the Gunslinger's Sheriff).
- **Headgear:** Iron Crown, Demon Horns, Laurel Wreath, Halo, Red Plume, Antlers.
- **Capes:** Royal, Ember, Frost, Shadow, Forest (and a cape for classes that had none).
- **Weapon finishes:** Gilded, Obsidian, Bone, Frostforged, Bloodsteel, applied to every weapon.
- **Auras:** Embers, Frost Motes, Fireflies, Shadow Wisps drifting around you.

Choices are saved per class, and co-op partners see what you wear. Saves from before 0.9 had their permanent upgrades and unlocks refunded as gold. Progress, settings and your last 30 runs are saved in `%APPDATA%/Emberwild/save.json`.

## Classes

| Class | Weapons | Abilities |
|---|---|---|
| Ranger | Longbow, Twin Daggers | Volley, Evasive Roll, Snare Trap, Arrow Storm |
| Mage | Fire Staff, Frost Wand | Blink, Frost Nova, Arcane Orb, Meteor |
| Warrior | Greatsword, Sword & Shield | Charge, Whirlwind, Ground Slam, Battle Cry |
| Priest | Holy Mace, Sacred Scepter | Mend, Smite, Divine Shield, Consecration |
| Necromancer | Reaper's Scythe, Skull Staff | Raise Dead (skeleton warriors and archers that fight for you), Bone Spear, Corpse Explosion, Soul Drain |
| Gunslinger | Twin Revolvers (12 rounds, then a twirl reload), Lever Rifle | Fan the Hammer, Smoke Roll, Dynamite, Deadeye (time slows, marked targets take guaranteed crits) |

Each level-up offers three picks (with a couple of rerolls per run) drawn from general boons, **relics**, chaos augments, **ability levels** and **upgrades for a specific ability or weapon**. Levelling up also blasts nearby enemies back.

### Ability levels and Unbound

Every card you take for an ability raises its level: its named upgrades and masteries, and **Empower** cards that simply add a level. Every level makes the ability bigger in its own way:

- **Volley** fires two more arrows per level and its fan widens until the arrows leave in a full 360-degree ring, with extra waves every few levels.
- **Smite** drops one more sword per level, so a well-fed Smite rains 10, 20 or 30 swords across the horde.
- Meteor showers grow, Raise Dead calls more skeletons, Fan the Hammer fires more shots in a wider arc, Deadeye marks more targets, Corpse Explosion detonates more bodies, and so on for all 24 abilities, along with more damage and shorter cooldowns.

Levels have no ceiling, but only two abilities can break past level 10: **the first two to reach 10 become Unbound** (gold frames in the HUD and the character panel) and keep growing forever. The other two stop at level 10, still strong, and their cards stop appearing. Rare **Surge** (+2 levels) and **Ascendance** (+1 to every ability) cards turn up too.

Each ability also has 3 named upgrades plus a **Mastery** that unlocks once you've put 2 ranks into it, and each weapon has 3. Some examples:

- **Splitting Arrows / Splitting Orb / Splitting Light:** projectiles burst into more projectiles on impact.
- **Cluster Fire, Explosive Snare, Vortex, Decoy, Sticky Bomb, Ossuary Wall:** change how an ability plays.
- **Masteries** transform an ability: Hail of Arrows, Mirror Image, Absolute Zero, Singularity, Armageddon, Bladestorm, Fissures, Warlord, Lifebloom, Judgment, Army of the Dead, Cascade, Hellfire, High Noon and more.

## Chaos augments

Chaos augments bend the rules of the whole run rather than strengthening one ability, and they stack into mayhem: **Chain Reaction** (kills explode), **Echo** (abilities recast themselves), **Arcane Surge** (every cast fires seeking bolts), **Stormcaller** (crits call lightning), **Frenzy** (kills cut cooldowns), **Gravity Well** (black holes swallow the horde), **Overload** (every 5th cast is a huge nova), **Ricochet** (projectiles bounce), **Phoenix Heart** (cheat death once) and **Berserker** (stronger at low health).

## Relics

Relics are auto-firing weapons any class can pick up. You can carry 4, and each has 5 ranks.

| Relic | What it does |
|---|---|
| Whirling Blades | Blades orbit you and cut what they touch |
| Thunder Call | Lightning strikes nearby enemies, later chaining |
| Ember Aura | A burning ring around you that sears and slows |
| Spirit Wisp | A companion that fires seeking bolts |
| Ricochet Axe | Boomerang axes that fly out through the horde and return |
| Firebomb Flask | Lobbed flasks that leave burning pools |
| Frost Halo | Periodic rings of ice shards that chill and freeze |

## The horde

Monsters don't wander: they hunt you. A director spawns packs in a ring just around the players (they claw up out of the ground), and both the headcount and their toughness climb with every minute you survive, across all three maps; the stage clock and threat level (Calm to Apocalypse) sit at the top of the screen. New kinds join as the run goes on: zombies and slimes first, then bats, skeletons, spiders, **bloaters** (they swell and burst; kill them in a crowd for chain reactions), cultists and club-swinging **brutes**, then monsters with jobs: **Gorehorn** chargers, **Shieldbearers** that block hits from the front, **Bone Shamans** that heal and haste the horde, **Ember Imps** that bomb from above and **Necromancers** that raise the dead. The Bone Wastes have **Sand Wurms** that tunnel and erupt under you. Monsters steer around lava and burn if they're knocked into it. Stragglers that fall far behind are recycled to the front of the fight.

- Every 75 to 100 seconds there's an event: a full ring of the dead rising around you, a bat swarm from one direction, or an elite war band.
- Every third night is a **Blood Moon**, and from minute 6 telegraphed eruptions land where you're heading.
- **Elites** carry affixes: Swift, Brute, Splitting, Vampiric and Volatile, and drop a treasure chest.
- Enemies drop experience gems (blue, green and red by value). Breakable **urns** and rare drops hold hearts, a Lodestone that pulls in every gem, and a Holy Relic that scours the horde.
- **Cursed altars** (red diamonds on the minimap): awaken one and survive 35 seconds of waves near it to earn a relic.

Enemy health scales with the number of players. To keep big fights smooth, the game caps enemies (130 hunting at once on High graphics, fewer on lower settings), projectiles, effects and pickups, and draws each enemy with a handful of merged meshes. It also lowers view distance, shadows and particles automatically when the frame rate drops. You can lock the graphics level in the pause menu.

## Graphics

Everything is generated in code; there are no art files.

- **Renderer:** HDR with bloom, SMAA anti-aliasing, screen-space ambient occlusion, cascaded soft sun shadows, and a thin haze around the camera that the sun lights into god rays through the trees. Tone mapping is filmic (TonyMcMapface) with gentle colour grading.
- **Sky:** a painted sky with drifting cumulus clouds that catch the sun, plus stars and the moon at night.
- **Blocks:** every block face has a 32px pixel-art texture with a matching normal and roughness map, all painted at startup. That covers grass with blades, cracked stone, bark, leaves, bricks, planks, magma and more.
- **Plants:** grass, flowers, ferns, reeds and lily pads are textured cut-out cards that sway in the wind. Bushes are domes of leaf clusters (berry bushes carry berries). Tree crowns have leafy fringes instead of cube edges, and all foliage casts leaf-shaped shadows.
- **Ambient life:** fireflies drift over meadows at night, pollen glints in sunbeams, and leaves fall in forests.
- **Characters:**
  - Bevelled parts, with elbows and knees.
  - Every weapon has keyframed attacks: a wind-up, a strike timed to the hit, a follow-through and a recovery. Melee weapons chain three different attacks, with a heavy third blow.
  - The bow draws back as it charges and kicks on release. Staffs thrust, wands flick and the scepter points.
  - Joints blend smoothly into each pose, and blades leave ribbon trails.
  - Running: hips turn towards where you're going while the chest keeps facing your aim, knees lift and absorb, the body leans into speed, free arms pump, and landings squash.
  - Turning: the head and chest turn towards the aim first; past about 35 degrees the feet step round to follow.
  - Casting: every ability has keyframed anticipation, release and follow-through (gathering power at the hip, summoning with arms sweeping up, an overhand throw, a prayer, a smite brought down). The bow nocks from the quiver and trembles at full draw; Volley is a sweeping fan shot. Guns snap up and kick with recoil, the rifle shoulders and works its lever, and the revolvers twirl on reload.
- **Effects:** anime-style rather than spell-circle magic. That means impact flashes with starbursts of speed lines, crescent slash marks, speed lines on dashes, sparkle glints, cel-shaded energy, sparks that streak, and a lens punch on big hits.

- **Interface:** a medieval voxel style painted in code: carved stone panels with iron rivets, iron-banded wooden plaques for buttons, parchment level-up cards with icons and ribbons, class banners hanging from a rod, stone ability slots and wood-framed bars. Buttons lift, brighten and knock softly on hover and sink when pressed, and screens slide in.

**Settings → Graphics quality:**

| Level | Ambient occlusion | God rays | Shadows |
|---|---|---|---|
| Low | none | none | small shadow maps |
| Medium | light | none | standard |
| High (default) | yes | yes | standard |
| Ultra | higher quality | longer | 4K shadow maps |

## Sound

Every sound is synthesised at startup. A mixer keeps fights readable: each sound has a minimum gap and a voice limit, rapid repeats get quieter, every play is pitched slightly differently (frequent sounds also have three takes), a crowd of the same sound in one frame plays once a little louder instead of stacking, and big moments (explosions, bosses, level-ups) briefly duck everything else. Each class has its own palette: the Ranger's soft bow thwips, the Mage's glassy wand ticks and warm staff puffs, the Warrior's heavy swings, the Priest's single soft bell, the Necromancer's bone rattles and soul whispers, and the Gunslinger's gun cracks, rifle lever and reload clicks.

## Tools

- **Dev build:** `Emberwild-<version>-dev.exe` keeps its own save (`save-dev.json`) with every cosmetic owned and every Ember level unlocked, so testing never touches your real progress. During a run, **F1** opens the dev menu: god mode, no cooldowns, full heal, levels, run gold, chests, rerolls, max relics, all upgrades, +5 levels to any ability, random chaos augments, weapon swap, skip to the stage boss, open a portal, go to the next stage, time of day, hordes, kill everything, boss spawns, every event and Camp gold.
- **Crash log:** if the game crashes, it writes `crash-<time>.log` (the error and a backtrace) to `%APPDATA%/Emberwild/`. Send that file along with a bug report.
- **Balance bot** (dev build only): run `Emberwild-<version>-dev.exe --bot` from a command prompt.
  - The game plays complete runs on its own in a small window, simulating several steps per frame so runs go much faster than real time. It aims, kites, draws the bow, casts abilities, dodges, eats, opens chests and picks level-up cards.
  - Each run is appended to `balance-<time>.csv` in the current folder, and a per-class summary goes to `balance-<time>-summary.txt`. The summary covers win rate, survival time, level, kills per minute, damage taken and healed per minute, and the heal-to-damage ratio.
  - Your save is never touched.
  - Options go in one quoted string, for example: `Emberwild.exe --bot "runs=5;classes=warrior,mage;minutes=20;turbo=8;ember=3"`.
    - `runs`: runs per class.
    - `classes`: any of ranger, mage, warrior, priest, necromancer, gunslinger.
    - `minutes`: the run cap.
    - `turbo`: steps per frame.
    - `ember`: difficulty.
    - `seed`: the base world seed. Each run's world is derived from it, so a change can be compared on the same worlds.

## The world

Each map is a walled-in arena about 220 blocks across. The Greenwood has rolling meadows, forests that open into glades and lakes with reeds and lily pads; the Drowned Fen is low bog full of pools, mud, willows and mist; the Bone Wastes have long dune ridges, striped mesas, cacti and palm-shaded oases; the Ember Throne is basalt and ash with obsidian spires, glowing vents and rivers of lava. Ruins and camps hold chests, shrines bless you, and altars offer trials. The day and night cycle continues (each new map starts at dawn), and the music builds as the horde closes in.

- **Water:** you float at shoulder depth, swim with a front crawl (Space to rise, swim into a bank to climb out) and tread water when still, with splashes, strokes and ripples.
- **Lava:** it burns hard and throws you back out, so a misstep hurts but doesn't trap you.

## Co-op

1. **Host:** on the title screen, click **Host co-op** (or press H). The title screen shows your LAN address, such as `192.168.1.20:7777`. Windows asks once to allow Emberwild through the firewall; allow it. Then pick a class.
2. **Friends:** click **Join co-op** (or press J), type that address (the port can be left off) and press Enter. You load into the host's world; pick a class.
3. To play over the internet, forward TCP port 7777 on the host's router, or use a LAN tunnel such as Tailscale or ZeroTier.

Up to four players. The host runs the horde, the bosses, events and the clock; everyone sees each other's spells and shares the XP from kills. Gold, loot and Camp progress are your own. A downed player can be revived by a teammate holding E next to them; the run ends when everyone is down. The game doesn't pause during co-op.

## Layout

| Path | What |
|---|---|
| `rust/src/main.rs` | App setup, the game tick, run start/restart, crash log |
| `rust/src/scene.rs`, `rust/src/shaders/` | Camera and post-processing, sky, water, terrain and foliage materials (WGSL), day/night, chunk streaming |
| `rust/src/textures.rs`, `rust/src/mesher.rs`, `rust/src/world.rs`, `rust/src/physics.rs` | Procedural block and foliage textures, chunk meshing (faces, bushes, canopy fringes, plants), the four map kinds, collision, swimming |
| `rust/src/models.rs`, `rust/src/player.rs`, `rust/src/anim.rs` | Box-built models, the player controller, camera, joint blending, weapon trails, and the keyframed weapon animations |
| `rust/src/combat.rs`, `abilities.rs`, `abilities2.rs`, `relics.rs`, `augments.rs` | Weapons, the 24 abilities and their levels, the Necromancer's skeletons, relics and evolutions, chaos augments |
| `rust/src/enemies.rs`, `horde.rs`, `director.rs`, `events.rs` | Monsters and their AI, bosses, the horde director, run events and the merchant |
| `rust/src/hud.rs`, `uikit.rs`, `camp.rs`, `cosmetics.rs`, `settings.rs`, `meta.rs` | Menus and HUD, the medieval UI kit, the Camp wardrobe and cosmetics, settings and gamepad, the save file |
| `rust/src/audio.rs` | Synthesised sounds, the mixer and the music |
| `rust/src/net.rs` | Co-op |
| `rust/src/bot.rs`, `testkit.rs` | The balance bot and the scripted screenshot tests |
| `rust/build-windows.sh` | Builds the player and dev Windows exes and zips them |
| `index.html`, `src/`, `desktop/` | The original browser version |
