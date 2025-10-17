# 7th october 2025:

**Got the basics of the renderer setup, rendering a default triangle and a square, experienced several segfault crashes but fixed them all through various means and help by others**
**Time worked on: 13 hours**

## Project Size:

| Language | files | lines |
| -------- | ----- | ----- |
| Rust     | 5     | ~900  |
| GLSL     | 2     | 40    |
| Total:   | 10    | ~950  |

## Changelog:

### Engine:

#### Renderer:

- got renderer setup
- shader loading added
- triangle :>

---

# 8th october 2025:

**Got a fully functional 3d perspective camera to render and a fully functional input manager, experienced 1 segfault and 3 computer crashes while doing this lmao**
**Time worked on: 11 hours**

## Project Size:

| Language | files | lines |
| -------- | ----- | ----- |
| Rust     | 8     | 1425  |
| GLSL     | 2     | 39    |
| Total:   | 10    | 1523  |

## Changelog:

### Engine:

#### Renderer:

- added a perspective class
- square []
- half functional depth buffer

#### Handlers:

- added input manager

---

# 9th octover 2025:

**Finished with the depth buffer finally, I missed a variable in the viewport ;~;**
**Time worked on: 3 hours**

## Project Size:

| Language | files | lines |
| -------- | ----- | ----- |
| Rust     | 9     | 1535  |
| GLSL     | 2     | 1     |
| Total:   | 10    | 1596  |

## Changelog:

### Engine:

#### Renderer:

- added depth buffer fully
- added vertex buffers

---

# 11th october 2025:

**Finally added an index buffer and fully implimented a seperation of engine and game along with the creation of uniform buffers**
**Time worked on: 5 hours**

## Project Size:

| Language | files | lines |
| -------- | ----- | ----- |
| Rust     | 16    | 1785  |
| GLSL     | 2     | 24    |
| Total:   | 18    | 1809  |

## Change Log:

### Engine:

#### Renderer:

- added index buffers
- added uniform buffers

### Game:

#### World:

- added test chunks for world generation

# 12th october 2025:

**Added the index buffer fully by replacing `cmd_draw` with `cmd_draw_indexed`, added face culling**

## Change Log:

### Engine:

#### Renderer:

- fixed uniform buffer rendering
- added back face fulling
- removed hidden faces of voxels

# 13th octover 2025:

**Fixed an issue with faces rendering 6 times as often as they should. Index buffer and vertex buffer now hold enough information for a chunk**
**Time worked on: 5 hours**

## Project Size:

| Language | files | lines |
| -------- | ----- | ----- |
| Rust     | 16    | 1901  |
| GLSL     | 2     | 24    |
| Total:   | 18    | 1925  |

## Change Log:

### Engine:

#### Renderer:

- fixed buffers being given 6x the information they needed

# 14th october 2025:

**Added release and pressed keybinds to the input manager, meaning keys dont have to be just held constantly**

## Changelog:

### Engine:

#### Input Manager:

- added release buttons
- added pressed keys

# 16th october 2025:

**Fixed issues with chunks not rendering, added chunk generation**

## Changelog

### Engine:

#### Renderer:

- fixed chunks not rendering due to incorrect vertex descriiptors

### Game:

#### Generation:

- added primative chunk generation

# 18th october 2025:

**Added full chunk meshing, chunk unloading and chunk generation**
**Time worked on: 6 hours**

## Project Size:

| Language | files | lines |
| -------- | ----- | ----- |
| Rust     | 20    | 2101  |
| GLSL     | 2     | 28    |
| Total:   | 18    | 2129  |

## Changelog

### Engine:

#### Renderer:

- chunks now hide their interior faces
- chunks will deload when past a certaint point

### Game:

#### Generation:

- chunks now generate infinitely on the x y and z axis
