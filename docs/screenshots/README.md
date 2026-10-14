# Native screenshots

All screenshots are JPEG at 2× Retina scale. File names describe logical window dimensions, including the native title bar. The six layout captures use a dedicated release `Local Chat Test.app` and isolated signed synthetic fixture content. They exercise the real frontend and Rust SQLite backend. No fixture data is bundled into production.

Light and Dark were checked at 1280×720 and the supported minimum 800×600. A requested 1440×900 native window was clamped by this Mac's available work area to 1440×859; the filenames record that actual size. Native Light/Dark switching was repeated after the avatar repair. The three material design fixes were independently scored resolved in `../evidence/design-verdict.md`; that verdict covers those fixes.

The 1280×800 captures show the production app communicating with an actual separate peer on the physical interface. They are functional evidence, not invented online people. Reference captures remain under `../reference/baselines/`; those early files contain JPEG bytes despite their original `.png` extension. The approved source HTML and screenshot remain untouched under `../reference/outputs/`.
