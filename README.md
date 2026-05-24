# mixosc

[![crates.io](https://img.shields.io/crates/v/mixosc.svg)](https://crates.io/crates/mixosc)

Rust OSC client for Behringer **X32** and **X-Air** digital mixers.

![MixOSC](images/mixosc.png)

`mixosc` currently contains two pieces:

- A desktop GUI built with `iced` that discovers mixers on the local network and exposes a mixer-style control surface.
- A Rust library for UDP/OSC discovery, connection probing, state loading, state updates, and meter parsing.

## Supported mixers

| Mixer | Channels | Buses | FX Returns | DCAs | Port |
|-------|----------|-------|------------|------|------|
| X32 / X32 Compact / X32 Producer / X32 Rack | 32 + 8 aux | 16 | 8 | 8 | 10023 |
| XR18 / XR16 / XR12 / X18 / X16 / X12 | 16 | 6 | 4 + 1 aux | 4 | 10024 |

## Current GUI behavior

The application discovers mixers automatically and loads a control surface matching the detected model.

### X32 layout

- `CH 01-32`
- `AUX 01-08`
- `BUS 01-12`
- `FX 01-08`
- `MTX 01-06`
- `DCA 1-8`
- `Main LR`

### X-Air layout

- `CH 01-16`
- `BUS 01-06`
- `FX 01-04` + `AUX`
- `DCA 1-4`
- `Main LR`

For the strips that support them, the UI shows and updates in real time:

- Name and scribble-strip color
- Input gain or trim
- Sends
- Pan
- Fader level
- Input, bus, main/matrix meters, and RTA (Real-Time Analyzer)
- Mute
- Solo

The app subscribes to live OSC updates with `/xremote` and meter subscriptions, so local changes on the mixer are reflected back into the UI.

## What is implemented per strip type

### X32

- Channels and aux inputs: gain/trim, sends to buses 1-16, pan, fader, mute, solo, color, name, meters
- FX returns: sends to buses 1-16, pan, fader, mute, solo, color, name
- Buses: sends to matrices 1-6, pan, fader, mute, solo, color, name, bus meters
- Matrices: fader, mute, color, name, matrix meter
- DCAs: fader, mute, color, name
- Main LR: fader, mute, color, stereo meter, RTA

### X-Air

- Channels: headamp gain, sends to buses 1-6, pan, fader, mute, solo, color, name, meters
- FX returns: sends to buses 1-6, pan, fader, mute, solo, color, name
- Buses: fader, mute, solo, color, name, bus meters
- DCAs: fader, mute, color, name
- Main LR: fader, mute, color, stereo meter, RTA

Implementation details from the current code:

- Gain uses headamp control where available and trim otherwise.
- On X32, channel `17-32` use trim gain; earlier channels and supported aux inputs can use headamp gain.
- On X-Air, all 16 channels have headamp gain.
- Gain is not exposed for buses, FX returns, matrices, or DCAs.
- Pan is not exposed for matrices or DCAs.
- DCA and matrix solo are not sent to the mixer.
- Master solo is only a local UI toggle right now; it is not sent to the mixer.

## Running

Automatic discovery on the local network:

```bash
cargo run
```

Connect to a specific mixer:

```bash
cargo run -- 192.168.1.62
```

You can also include a custom port:

```bash
cargo run -- 192.168.1.62:10023
cargo run -- 192.168.1.62:10024
```

Or use the environment variable:

```bash
MIXOSC_MIXER_ADDR=192.168.1.62 cargo run
```

Default ports:

- X32: `10023`
- X-Air (XR18, etc.): `10024`

When no port is given, the app tries the default ports automatically.

## Library surface

The crate exports OSC helpers from `src/common.rs`, with model-specific implementations in `src/x32.rs` and `src/xr18.rs`:

- Discovery and connectivity: `DiscoveryProbe`, `ConnectionProbe`, `DiscoveredMixer`, `ProbeOutcome`
- Strip state loading and control: `FaderBankProbe`, `PanBankProbe`, `GainBankProbe`, `SendBankProbe`, `MuteBankProbe`, `SoloBankProbe`, `NameBankProbe`, `ColorBankProbe`
- Meter handling: `batchsubscribe_meter_request`, `renew_request`, `parse_input_meter_packet`, `parse_main_meter_packet`, `parse_rta_meter_packet`
- Console update parsing: `parse_console_update`, `ConsoleUpdate`
- Address parsing and constants: `parse_target`, `X32_DEFAULT_PORT`, `X32_BROADCAST_ADDR`, `XR18_DEFAULT_PORT`, `XR18_BROADCAST_ADDR`, `XREMOTE_REQUEST`

This repository has a single binary entry point in `src/main.rs` and a reusable library in `src/lib.rs`.
