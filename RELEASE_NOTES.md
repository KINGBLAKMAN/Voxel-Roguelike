## Emberwild 0.5.0: rebuilt in Rust

**Download:** `Emberwild-0.5.0.exe` is the whole game in one file; no installer and no browser. Windows may show a SmartScreen warning because the exe isn't code-signed: click *More info*, then *Run anyway*.

Emberwild has been rewritten from scratch in Rust on the Bevy engine. It is now a native game with an HDR renderer (bloom, ACES tone mapping, cascaded sun shadows, fog), a sky shader with sun, moon and stars, animated water, and grass and leaves that sway in the wind.

### Spells you can see
Every ability now drops a real object into the world instead of just flashing light:
- **Meteor:** a jagged basalt rock split by glowing magma falls out of the sky trailing fire and smoke. It blasts a crater into the terrain, throws the blocks out as tumbling chunks, and stays half-buried and smoking before it crumbles.
- **Arrow Storm:** you fire one arrow straight up, a storm cloud heaps up over the target, and real arrows rain out of it and stick in the ground.
- **Snare Trap:** a steel bear trap with chain and stake tumbles through the air, springs open, and its toothed jaws snap shut on the first monster to step in.
- **Frost Nova:** three rings of ice crystals punch out of the ground one after another, frost spreads, and enemies freeze inside blocks of ice.
- **Smite:** a colossal sword of light falls point-first and drives into the earth, then dissolves.
- **Ground Slam:** you leap, and pillars of the ground itself (grass cap, dirt, stone) heave up around the impact.
- **Battle Cry:** four war banners crash down around you and stand while the cry lasts.
- **Mend:** a lotus of light unfolds at your feet. **Divine Shield:** six kite shields of light orbit you inside a golden dome. **Consecration:** white obelisks with burning runes rise around the holy ground.
- Also: a gyroscope Arcane Orb with spinning rings, Blink rifts, a straw-dummy Decoy, Mirror Image, a Whirlwind funnel with ghost swords, Charge's ram wedge, spinning Bladestorm blades.

### Everything else
- Vampire Survivors-style horde director: packs spawn just out of sight and hunt you, numbers and monster types climb over time, with ring ambushes, bat swarms, elite war bands and Ancient Golem bosses.
- 4 classes, 16 abilities, 64 ability upgrades and masteries, 24 weapon upgrades, 7 relics, 10 chaos augments, 22 boons.
- Chests, shrines, altar trials, loot urns, food and hunger, a minimap, floating damage numbers and a generative soundtrack whose drums swell as the fight heats up.

**Controls:** WASD move, mouse aim, left click attack, 1-4 abilities, Q swap weapon, E interact, F eat, Shift sprint, M mute, Esc pause.

Co-op from the old browser version isn't in the Rust build yet.
