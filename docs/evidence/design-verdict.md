## verdict

1. Resolved — avatar initials: the replacement light/dark captures at 1280×720, 800×600 and 1440×859 visibly show complete “UV” and “FA” initials. The former dark blank/clipped glyphs are absent. Explicit dark avatar surfaces/foregrounds use existing semantic tokens, with the softly squared footprint preserved. This bounded native rendering/accessibility adaptation is documented in DESIGN.md.
2. Resolved — compact sidebar labels: both replacement 800×600 captures show “Find messages” and its shortcut on one line, with “People nearby” and the connected count whole on one line. Normal desktop captures retain the fuller labels and existing proportions.
3. Resolved — production design documentation: DESIGN.md describes backend-driven people/delivery, Rust-owned SQLite persistence, Local Chat branding, native framing and the repaired dark avatars. The referenced .impeccable/design.json now exists. Historical prototype copies remain the reference authority.

## remaining

Clear for the three scored fixes. No material regression from this repair batch is visible in the six replacement captures. All eight named evidence files retain valid 2× JPEG dimensions. This ship verdict covers the scored fixes, not the whole surface. The unchanged 1280×800 production captures are historical functional evidence; this pass does not certify the pending final installed-app capture, 1440×900 native sizing, Windows/Linux, or broader native behavior.

disposition: ship
