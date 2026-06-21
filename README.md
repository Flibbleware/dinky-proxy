<img width="1058" height="492" alt="DinkyProxy app" src="https://github.com/user-attachments/assets/ec04d455-8305-4d0c-9120-321d3800cc5d" />

Lightweight precision proxy app for macOS & Windows.

<img width="1584" height="733" alt="Screenshot 2026-06-21 at 20 36 06" src="https://github.com/user-attachments/assets/19b06c2b-c31a-4515-acaf-e8bc312a9282" />

## Download & Install

Download the latest macOS (`.dmg`) and Windows (`.msi` / `.exe`) builds from the [Releases page](https://github.com/Flibbleware/dinky-proxy/releases/latest).

### macOS

The builds are **not code-signed or notarized yet**, so after downloading a `.dmg` your browser tags it with a quarantine flag and macOS shows **"DinkyProxy is damaged and can't be opened."** The app isn't actually damaged — Gatekeeper is blocking an un-notarized download.

Drag `DinkyProxy.app` into `/Applications`, then remove the quarantine flag (you only need to do this once per download):

```bash
xattr -cr /Applications/DinkyProxy.app
```

Then open the app normally.

### Windows

Run the `.msi` or `.exe` installer and follow the prompts.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to build and run the project locally.
