# Design

How Ventricle looks and feels. When this file and your instincts disagree, this file wins. Change it on purpose, in the same commit as the code.

## The look in one line

A plain stompbox in arterial red, stomped through a hundred basement shows.

## Principles

Every plugin gets its own look, but they share an approach.

- **Skeuomorphic.** The editor is a physical object (Patina an OP-1-style synth, Halfwit a stompbox, Ghosted a vintage radio), with real materials like powder coat, bakelite, walnut, brass and grille cloth, built in CSS gradients and shadows rather than images.
- **One theme.** Pick the object from the plugin's vibe and name, and let it run through everything: the wordmark, the art and the control labels. Names should be playful and capture the vibe.
- **Restrained.** One knob style and colour family per plugin, and few controls. Fold many settings into a handful of knobs with evocative names rather than exposing every one.
- **Colour and art mean something.** If two waves or knobs differ in colour, it's because they differ in function; otherwise make them the same. Art can follow parameters (Ghosted's ghosts fade with Haunt, the dial needle follows Distance), but don't fake audio metering.
- **Plain labels.** Words a guitarist understands, not DSP terms or fieldset jargon. Numbers go in a tooltip readout.
- **Type with character.** Embed a display face for the wordmark (Futura-like Jost, pillowy Modak), never a system or "commercial" font, and centre it optically on its badge.
- **Drawn, not vector-perfect.** Vary repeated elements (size, tilt, expression), and give outlines weight that swells on the shadow side, like inked comic art.
- **A footswitch on effects.** Pedals and other effects get an on/off toggle (`stomp`, on by default). Off fades the effect out and passes the dry signal; a lamp or LED shows the state, and the art reacts (Ghosted's dial dims and its ghosts leave).
- **The version on the panel.** Print it small and quiet as part of the object, like a model number on a pedal's base plate or a serial plate on a radio's back (`v1.2.0`), so anyone can tell which build they're running. Local installs show `X.Y.Z-dev+<sha>`.
- **Physical details behave physically.** Shadows stay put while a knob turns, pressed keys sink, lit things glow.
- **The object casts a shadow.** It sits inside a slightly larger window and casts an outer drop shadow (Ghosted's radio: `0 30px 44px` and `0 6px 10px`, 560px wide in a 600px window). The README screenshot keeps that shadow on transparency, so an object that fills its window edge to edge, with only inset shadows, looks pasted on.

## This plugin

Set by the `design-kickoff` skill.

- **Object:** A plain guitar stompbox, not a themed object, 400 × 560 in a 460 × 620 window. Road-worn: paint chipped through to bare aluminium around the edges and in a few chips across the face, the wordmark scuffed.
- **Palette:** Arterial red powder coat, cream printing, bare aluminium where the paint is gone, black knurled knobs, a red LED, a chrome hex footswitch.
- **Wordmark font:** Pirata One, a readable blackletter, for the wordmark and knob labels, with no subtitle under the wordmark, embedded as a base64 ASCII subset (`ui/fonts`). Chosen over Rubik Dirt, Black Ops One and Permanent Marker.
- **Controls:** Strain, Growl, Rasp, Vowel, Cup and Level in a 3 × 2 grid, plus the Stomp footswitch and its LED. Presets set the throat size and the number of doubles behind the scenes. Numbers show in a pill under a knob while it's hovered, focused or turned.
- **Version placement:** Stamped into a bare-metal patch, bottom left, where the red has worn off.
- **Art:** Nothing reacts beyond the knobs turning and the LED.

## Tokens

Every color, size and space comes from a token. If nothing fits, add a token here before using a new value.

| Token | Value | Used for |
|---|---|---|
| `--coat`, `--coat-hi`, `--coat-lo` | `#8e1414`, `#b3221d`, `#5a0a0b` | The powder coat |
| `--metal`, `--metal-hi`, `--metal-lo` | `#c4c6c8`, `#eceeef`, `#8a8d90` | Worn-through aluminium, screws, footswitch, version stamp |
| `--cream` | `#efe6cf` | Printing, pointers, ticks |
| `--knob`, `--knob-hi` | `#151413`, `#34312e` | Knurled knob skirts |
| `--led`, `--led-off` | `#ff3b2f`, `#4a1310` | The LED |
| `--pill` | `#120f0c` | Value readout |
| `--pedal-w`, `--pedal-h` | `400px`, `560px` | The pedal |
| `--knob-size` | `62px` | Knob diameter |
| `--spots`, `--scuffed` | SVG noise masks | Edge wear; the wordmark's scuffs |

## Components

- **Knob:** cream tick dots (11, with larger ends and middle), a knurled skirt and cap with a cream pointer that turn, and a shine and drop shadow that stay put. Same as Halfwit's.
- **Footswitch and LED:** a chrome hex nut with a round button that sinks when pressed; the LED glows while the effect is on.
- **Wear:** the edge wear is a metal border masked by `--spots`; the chips are a handful of placed `.chip` shapes, varied in size and tilt.

## Patterns

## Adding something new

1. Look for an existing component or pattern that already does it.
2. Build it from tokens only.
3. Check Reduce Motion if it moves.
4. Retake the screenshot if it shows in one.
5. Record it above.

## Known drift

- Shadows, highlights and gradient stops in `ui/index.html` use raw values (`#3a0607`, `#ff8a80`, `#26231f`, `#5d6064` and the like) rather than tokens, and the readout pill uses a system font.

Existing debt to clean up, not to copy.
