# Application icons

`master.svg` is the editable application icon. `master.png` is its production raster export.
`tray.svg` is the monochrome menu-bar master. Tauri packages the committed platform exports in `src-tauri/icons`.

Regenerate platform icons from the repository root:

```sh
npm run tauri icon -- assets/icon/master.svg
```

Keep committed exports synchronized with the source. Check small sizes and both desktop appearances before changing the icon.
