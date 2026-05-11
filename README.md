# Apostasy Engine

Apostasy is an experimental Rust game engine prototype built specifically for the game *Apostasy*, a Morrowind-inspired RPG with 1990s era design goals. The engine is designed to support the kind of open-world RPG systems and scene-driven gameplay needed by that game, while also providing basic editor-style tooling.

## What the engine does

- launches a window and render loop
- maintains a shared `World` containing scene nodes
- updates input state and propagates transforms
- loads assets such as shaders, scenes, and materials
- renders content through a Vulkan backend
- provides editor scaffolding for inspecting and editing runtime state

The runtime is driven by an engine crate and a small root application that starts the engine.

## How it works

The engine initializes a window and Vulkan context, then creates:
- a `World` with default scene nodes and input handling
- an `AssetServer` for loading resources from `res/`
- a renderer for drawing frames
- editor state for runtime inspection

Input events are dispatched into the world, and update hooks can operate directly on the shared `World`.

### World and scene graph

The world is a tree of `Node` objects. Each node has:
- an identifier and name
- a default transform component
- optional child nodes
- a collection of typed components

Components are stored as dynamic trait objects, which lets the engine attach behavior such as cameras, physics, velocity, and terrain to nodes.

Transforms are propagated through the node hierarchy so child nodes inherit position, rotation, and scale from their parents.

### Update flow

The engine exposes fixed-update hooks for game logic. The root app uses a macro to register a function that receives:
- a mutable reference to the `World`
- the time delta for the current update

This is where gameplay code can query input, move objects, and modify component state.

### Rendering flow

Rendering is handled by an internal renderer that manages:
- Vulkan swapchain and surface setup
- shader loading
- material and model rendering
- window resize and redraw requests

The engine supports multiple windows and updates render targets when window state changes.

## Usage

Run the engine from the repository root with:

```bash
cargo run
```

This starts the application and opens the engine window.

For a release build:

```bash
cargo build --release
```

## Custom logic

Game code can be attached through the engine's update hooks rather than through a separate game loop. A sample root app demonstrates:
- starting the engine
- reading input from the world
- locating nodes by component type
- moving a player node using velocity and transform components

The engine's macros simplify registering those update callbacks.

## Asset loading

Assets are loaded through an engine asset server at runtime. The current setup registers loaders for:
- shaders
- materials
- scenes

Scene files and shader assets are used to populate the runtime world and rendering state.

## Current limitations

Apostasy is not a finished engine. Existing limitations include:
- limited system scheduling and ECS behavior
- basic lighting and rendering support
- incomplete asset import/export
- prototype editor UI
- early-stage API and architecture

## Requirements

- Rust toolchain (stable, edition 2024)
- Vulkan-capable system and drivers
- `cargo` available on PATH

## Notes

This project is primarily a learning and experimentation engine. It is useful as a reference for how a Rust engine can wire together windowing, Vulkan rendering, a scene graph, and runtime update hooks.
