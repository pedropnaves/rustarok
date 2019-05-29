use std::collections::btree_map::BTreeMap;
use std::collections::HashMap;

use nalgebra::{Vector2, Vector3};
use nalgebra_glm::triangle_normal;

use crate::common::BinaryReader;
use crate::opengl::GlTexture;
use sdl2::pixels::{PixelFormatEnum, Color};
use sdl2::rect::Rect;

pub struct Gnd {
    pub version: f32,
    pub width: u32,
    pub height: u32,
    pub zoom: f32,
    pub texture_names: Vec<String>,
    pub texture_indices: Vec<usize>,
    pub lightmaps: LightmapData,
    pub lightmap_image: Vec<u8>,
    pub tiles_color_image: Vec<u8>,
    pub shadowmap_image: Vec<u8>,
    pub tiles: Vec<Tile>,
    pub surfaces: Vec<Surface>,
    pub mesh: Vec<[MeshVertex; 6]>,
    pub mesh_vert_count: usize,
    pub water_vert_count: usize,
    pub water_mesh: Vec<[WaterVertex; 6]>,
    pub shadow_map: Vec<[WaterVertex; 6]>,
}

pub struct LightmapData {
    pub per_cell: u32,
    pub count: u32,
    pub data: Vec<u8>,
}

pub struct Tile {
    pub u1: f32,
    pub u2: f32,
    pub u3: f32,
    pub u4: f32,
    pub v1: f32,
    pub v2: f32,
    pub v3: f32,
    pub v4: f32,
    pub texture: usize,
    pub light: u16,
    pub color: [u8; 4],
}

pub struct Surface {
    pub height: [f32; 4],
    pub tile_up: isize,
    pub tile_front: isize,
    pub tile_right: isize,
}

#[repr(packed)]
#[derive(Debug)]
pub struct MeshVertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub texcoord: [f32; 2],
    pub lightcoord: [f32; 2],
    pub tilecoord: [f32; 2],
}

pub struct WaterVertex {
    pos: [f32; 3],
    texcoord: [f32; 2],
}

impl Gnd {
    pub fn load(mut buf: BinaryReader, water_level: f32, water_height: f32) -> Gnd {
        let header = buf.string(4);
        if header != "GRGN" {
            panic!("Invalig Gnd header: {}", header);
        }

        let version = buf.next_u8() as f32 + buf.next_u8() as f32 / 10f32;
        let width = buf.next_u32();
        let height = buf.next_u32();
        let zoom = buf.next_f32();
        println!("width: {}, height: {}, zoom: {}", width, height, zoom);

        let (texture_names, texture_indices) = Gnd::load_textures(&mut buf);
        let lightmaps = Gnd::load_lightmaps(&mut buf);
        let tiles = Gnd::load_tiles(&mut buf, texture_names.len(), &texture_indices);
        println!("tiles: {}", tiles.len());
        let surfaces = Gnd::load_surfaces(&mut buf, width, height);
        println!("surfaces: {}", surfaces.len());
        let normals = Gnd::smooth_normal(width as usize,
                                         height as usize,
                                         &surfaces,
                                         &tiles);

        let l_count_w = (lightmaps.count as f32).sqrt().round();
        let l_count_h = (lightmaps.count as f32).sqrt().ceil();
        let l_width = 2f32.powi((l_count_w * 8.0).log2().ceil() as i32);
        let l_height = 2f32.powi((l_count_h * 8.0).log2().ceil() as i32);

        let mut mesh = Vec::<[MeshVertex; 6]>::with_capacity((width * height * 3) as usize);
        let mut water = Vec::<[WaterVertex; 6]>::with_capacity((width * height * 3 / 2) as usize);
        for y in 0..height {
            for x in 0..width {
                let cell_a = &surfaces[(x + y * width) as usize];
                let h_a = cell_a.height;
                let x = x as f32;
                let y = y as f32;

                if cell_a.tile_up > -1 {
                    let tile = &tiles[cell_a.tile_up as usize];
                    let n = &normals[(y as u32 * width + x as u32) as usize];
                    let (u1, u2, v1, v2) = Gnd::lightmap_atlas(tile.light,
                                                               l_count_w,
                                                               l_count_h,
                                                               l_width,
                                                               l_height);
                    mesh.push([
                        MeshVertex {
                            pos: [(x + 0.0) * 2.0, h_a[0], (y + 0.0) * 2.0],
                            normal: [n[0][0], n[0][1], n[0][1]],
                            texcoord: [tile.u1, tile.v1],
                            lightcoord: [u1, v1],
                            tilecoord: [(x + 0.5) / width as f32, (y + 0.5) / height as f32],
                        },
                        MeshVertex {
                            pos: [(x + 1.0) * 2.0, h_a[1], (y + 0.0) * 2.0],
                            normal: [n[1][0], n[1][1], n[1][1]],
                            texcoord: [tile.u2, tile.v2],
                            lightcoord: [u2, v1],
                            tilecoord: [(x + 1.5) / width as f32, (y + 0.5) / height as f32],
                        },
                        MeshVertex {
                            pos: [(x + 1.0) * 2.0, h_a[3], (y + 1.0) * 2.0],
                            normal: [n[2][0], n[2][1], n[2][1]],
                            texcoord: [tile.u4, tile.v4],
                            lightcoord: [u2, v2],
                            tilecoord: [(x + 1.5) / width as f32, (y + 1.5) / height as f32],
                        },
                        MeshVertex {
                            pos: [(x + 1.0) * 2.0, h_a[3], (y + 1.0) * 2.0],
                            normal: [n[2][0], n[2][1], n[2][1]],
                            texcoord: [tile.u4, tile.v4],
                            lightcoord: [u2, v2],
                            tilecoord: [(x + 1.5) / width as f32, (y + 1.5) / height as f32],
                        },
                        MeshVertex {
                            pos: [(x + 0.0) * 2.0, h_a[2], (y + 1.0) * 2.0],
                            normal: [n[3][0], n[3][1], n[3][1]],
                            texcoord: [tile.u3, tile.v3],
                            lightcoord: [u1, v2],
                            tilecoord: [(x + 0.5) / width as f32, (y + 1.5) / height as f32],
                        },
                        MeshVertex {
                            pos: [(x + 0.0) * 2.0, h_a[0], (y + 0.0) * 2.0],
                            normal: [n[0][0], n[0][1], n[0][1]],
                            texcoord: [tile.u1, tile.v1],
                            lightcoord: [u1, v1],
                            tilecoord: [(x + 0.5) / width as f32, (y + 0.5) / height as f32],
                        },
                    ]);

                    fn one_if_zero(i: f32) -> f32 {
                        if i == 0.0 { 1.0 } else { i }
                    }
                    // Add water only if it's upper than the ground.
                    if h_a[0] > water_level - water_height ||
                        h_a[1] > water_level - water_height ||
                        h_a[2] > water_level - water_height ||
                        h_a[3] > water_level - water_height {
                        water.push([
                            WaterVertex {
                                pos: [(x + 0.0) * 2.0, water_level, (y) * 2.0],
                                texcoord: [x % 5.0 / 5.0, y % 5.0 / 5.0],
                            },
                            WaterVertex {
                                pos: [(x + 1.0) * 2.0, water_level, y * 2.0],
                                texcoord: [
                                    one_if_zero(((x + 1.0) % 5.0 / 5.0)),
                                    y % 5.0 / 5.0,
                                ],
                            },
                            WaterVertex {
                                pos: [(x + 1.0) * 2.0, water_level, (y + 1.0) * 2.0],
                                texcoord: [
                                    one_if_zero((x + 1.0) % 5.0 / 5.0),
                                    one_if_zero((y + 1.0) % 5.0 / 5.0),
                                ],
                            },
                            WaterVertex {
                                pos: [(x + 1.0) * 2.0, water_level, (y + 1.0) * 2.0],
                                texcoord: [
                                    one_if_zero((x + 1.0) % 5.0 / 5.0),
                                    one_if_zero((y + 1.0) % 5.0 / 5.0),
                                ],
                            },
                            WaterVertex {
                                pos: [(x + 0.0) * 2.0, water_level, (y + 1.0) * 2.0],
                                texcoord: [
                                    x % 5.0 / 5.0,
                                    one_if_zero((y + 1.0) % 5.0 / 5.0),
                                ],
                            },
                            WaterVertex {
                                pos: [(x + 0.0) * 2.0, water_level, y * 2.0],
                                texcoord: [x % 5.0 / 5.0, y % 5.0 / 5.0],
                            }
                        ]);
                    }
                }

                if (cell_a.tile_front > -1) && (y + 1.0 < height as f32) {
                    let tile = &tiles[cell_a.tile_front as usize];

                    let cell_b = &surfaces[(x + (y + 1.0) * width as f32) as usize];
                    let h_b = cell_b.height;
                    let (u1, u2, v1, v2) = Gnd::lightmap_atlas(tile.light,
                                                               l_count_w,
                                                               l_count_h,
                                                               l_width,
                                                               l_height);
                    mesh.push([
                        MeshVertex {
                            pos: [(x + 0.0) * 2.0, h_b[0], (y + 1.0) * 2.0],
                            normal: [0.0, 0.0, 1.0],
                            texcoord: [tile.u3, tile.v3],
                            lightcoord: [u1, v2],
                            tilecoord: [0.0, 0.0],
                        },
                        MeshVertex {
                            pos: [(x + 1.0) * 2.0, h_a[3], (y + 1.0) * 2.0],
                            normal: [0.0, 0.0, 1.0],
                            texcoord: [tile.u2, tile.v2],
                            lightcoord: [u2, v1],
                            tilecoord: [0.0, 0.0],
                        },
                        MeshVertex {
                            pos: [(x + 1.0) * 2.0, h_b[1], (y + 1.0) * 2.0],
                            normal: [0.0, 0.0, 1.0],
                            texcoord: [tile.u4, tile.v4],
                            lightcoord: [u2, v2],
                            tilecoord: [0.0, 0.0],
                        },
                        MeshVertex {
                            pos: [(x + 0.0) * 2.0, h_b[0], (y + 1.0) * 2.0],
                            normal: [0.0, 0.0, 1.0],
                            texcoord: [tile.u3, tile.v3],
                            lightcoord: [u1, v2],
                            tilecoord: [0.0, 0.0],
                        },
                        MeshVertex {
                            pos: [(x + 1.0) * 2.0, h_a[3], (y + 1.0) * 2.0],
                            normal: [0.0, 0.0, 1.0],
                            texcoord: [tile.u2, tile.v2],
                            lightcoord: [u2, v1],
                            tilecoord: [0.0, 0.0],
                        },
                        MeshVertex {
                            pos: [(x + 0.0) * 2.0, h_a[2], (y + 1.0) * 2.0],
                            normal: [0.0, 0.0, 1.0],
                            texcoord: [tile.u1, tile.v1],
                            lightcoord: [u1, v1],
                            tilecoord: [0.0, 0.0],
                        }
                    ]);
                }
                // Check tile right
                if (cell_a.tile_right > -1) && (x + 1.0 < width as f32) {
                    let tile = &tiles[cell_a.tile_right as usize];

                    let cell_b = &surfaces[((x + 1.0) + y * width as f32) as usize];
                    let h_b = cell_b.height;
                    let (u1, u2, v1, v2) = Gnd::lightmap_atlas(tile.light,
                                                               l_count_w,
                                                               l_count_h,
                                                               l_width,
                                                               l_height);
                    mesh.push([
                        MeshVertex {
                            pos: [(x + 1.0) * 2.0, h_a[1], (y + 0.0) * 2.0],
                            normal: [1.0, 0.0, 0.0],
                            texcoord: [tile.u2, tile.v2],
                            lightcoord: [u2, v1],
                            tilecoord: [0.0, 0.0],
                        },
                        MeshVertex {
                            pos: [(x + 1.0) * 2.0, h_a[3], (y + 1.0) * 2.0],
                            normal: [1.0, 0.0, 0.0],
                            texcoord: [tile.u1, tile.v1],
                            lightcoord: [u1, v1], // (l.u1, l.v1)
                            tilecoord: [0.0, 0.0],
                        },
                        MeshVertex {
                            pos: [(x + 1.0) * 2.0, h_b[0], (y + 0.0) * 2.0],
                            normal: [1.0, 0.0, 0.0],
                            texcoord: [tile.u4, tile.v4],
                            lightcoord: [u2, v2], // (l.u1, l.v1)
                            tilecoord: [0.0, 0.0],
                        },
                        MeshVertex {
                            pos: [(x + 1.0) * 2.0, h_b[0], (y + 0.0) * 2.0],
                            normal: [1.0, 0.0, 0.0],
                            texcoord: [tile.u4, tile.v4],
                            lightcoord: [u2, v2], // (l.u1, l.v1)
                            tilecoord: [0.0, 0.0],
                        },
                        MeshVertex {
                            pos: [(x + 1.0) * 2.0, h_b[2], (y + 1.0) * 2.0],
                            normal: [1.0, 0.0, 0.0],
                            texcoord: [tile.u3, tile.v3],
                            lightcoord: [u1, v2], // (l.u1, l.v1)
                            tilecoord: [0.0, 0.0],
                        },
                        MeshVertex {
                            pos: [(x + 1.0) * 2.0, h_a[3], (y + 1.0) * 2.0],
                            normal: [1.0, 0.0, 0.0],
                            texcoord: [tile.u1, tile.v1],
                            lightcoord: [u1, v1], // (l.u1, l.v1)
                            tilecoord: [0.0, 0.0],
                        }
                    ]);
                }
            }
        }

        mesh.shrink_to_fit();
        unsafe {
            println!("{:?}", std::mem::transmute::<_, &[f32]>(&mesh[0..100]));
        }
        water.shrink_to_fit();

        let mesh_vert_count = mesh.len() / 12;
        let water_vert_count = water.len() / 5;
        let lightmap_image = Gnd::create_lightmap_image(&lightmaps);
        let tiles_color_image = Gnd::create_tiles_color_image(
            width as usize,
            height as usize,
            &surfaces,
            &tiles,
        );
        let shadowmap_image = Gnd::create_shadowmap_image(
            width as usize,
            height as usize,
            &surfaces,
            &tiles,
            &lightmaps,
        );

        Gnd {
            version,
            width,
            height,
            zoom,
            texture_names,
            texture_indices,
            lightmaps,
            tiles,
            surfaces,
            mesh,
            mesh_vert_count,
            water_vert_count,
            water_mesh: water,
            tiles_color_image,
            shadowmap_image,
            lightmap_image,
            shadow_map: vec![],
        }
    }

    fn lightmap_atlas(i: u16,
                      l_count_w: f32,
                      l_count_h: f32,
                      l_width: f32,
                      l_height: f32) -> (f32, f32, f32, f32) /*u1, u2, v1, v2*/ {
        (
            (((i % l_count_w as u16) as f32 + 0.125) / l_count_w) * ((l_count_w * 8.0) / l_width),
            (((i % l_count_w as u16) as f32 + 0.875) / l_count_w) * ((l_count_w * 8.0) / l_width),
            ((i.checked_div(l_count_w as u16).unwrap_or(0) as f32 + 0.125) / l_count_h) * ((l_count_h * 8.0) / l_height),
            ((i.checked_div(l_count_w as u16).unwrap_or(0) as f32 + 0.875) / l_count_h) * ((l_count_h * 8.0) / l_height)
        )
    }

    fn load_surfaces(buf: &mut BinaryReader, width: u32, height: u32) -> Vec<Surface> {
        (0..width * height).map(|i| {
            Surface {
                height: [buf.next_f32() / 5f32, buf.next_f32() / 5f32, buf.next_f32() / 5f32, buf.next_f32() / 5f32],
                tile_up: buf.next_i32() as isize,
                tile_front: buf.next_i32() as isize,
                tile_right: buf.next_i32() as isize,
            }
        }).collect()
    }

    fn load_tiles(buf: &mut BinaryReader, texture_count: usize, texture_indices: &Vec<usize>) -> Vec<Tile> {
        let count = buf.next_u32();
        // Texture atlas stuff
        let atlas_cols: f32 = (texture_count as f32).sqrt().round();
        let atlas_rows: f32 = (texture_count as f32).sqrt().ceil();
        let atlas_width: f32 = 2f32.powf((atlas_cols * 258f32).log2().ceil());
        let atlas_height: f32 = 2f32.powf((atlas_rows * 258f32).log2().ceil());
        let atlas_factor_u: f32 = (atlas_cols * 258f32) / atlas_width;
        let atlas_factor_v: f32 = (atlas_rows * 258f32) / atlas_height;
        let atlas_px_u: f32 = 1f32 / 258f32;
        let atlas_px_v: f32 = 1f32 / 258f32;

        (0..count).map(|i| {
            let u1 = buf.next_f32();
            let u2 = buf.next_f32();
            let u3 = buf.next_f32();
            let u4 = buf.next_f32();
            let v1 = buf.next_f32();
            let v2 = buf.next_f32();
            let v3 = buf.next_f32();
            let v4 = buf.next_f32();
            let texture = texture_indices[buf.next_u16() as usize];

            let u = (texture % atlas_cols as usize) as f32;
            let v = (texture as f32 / atlas_cols as f32).floor();

            Tile {
                u1: (u + u1 * (1f32 - atlas_px_u * 2f32) + atlas_px_u) * atlas_factor_u / atlas_cols,
                u2: (u + u2 * (1f32 - atlas_px_u * 2f32) + atlas_px_u) * atlas_factor_u / atlas_cols,
                u3: (u + u3 * (1f32 - atlas_px_u * 2f32) + atlas_px_u) * atlas_factor_u / atlas_cols,
                u4: (u + u4 * (1f32 - atlas_px_u * 2f32) + atlas_px_u) * atlas_factor_u / atlas_cols,
                v1: (v + v1 * (1f32 - atlas_px_v * 2f32) + atlas_px_v) * atlas_factor_v / atlas_rows,
                v2: (v + v2 * (1f32 - atlas_px_v * 2f32) + atlas_px_v) * atlas_factor_v / atlas_rows,
                v3: (v + v3 * (1f32 - atlas_px_v * 2f32) + atlas_px_v) * atlas_factor_v / atlas_rows,
                v4: (v + v4 * (1f32 - atlas_px_v * 2f32) + atlas_px_v) * atlas_factor_v / atlas_rows,
                texture,
                light: buf.next_u16(),
                color: [buf.next_u8(), buf.next_u8(), buf.next_u8(), buf.next_u8()],
            }
        }).collect()
    }

    fn create_lightmap_image(lightmap: &LightmapData) -> Vec<u8> {
        let width = (lightmap.count as f32).sqrt().round() as usize;
        let height = (lightmap.count as f32).sqrt().ceil() as usize;
        let _width = 2f32.powi((width as f32 * 8f32).log2().ceil() as i32) as usize;
        let _height = 2f32.powi((height as f32 * 8f32).log2().ceil() as i32) as usize;
        let mut out = vec![0; (_width * _height * 4) as usize];

        for i in 0..(lightmap.count as usize) {
            let per_cell = lightmap.per_cell as usize;
            let pos = i * 4 * per_cell;
            let x = (i % width) * 8;
            let y = i.checked_div(width).unwrap_or(0) * 8;
            for _x in 0..8 {
                for _y in 0..8 {
                    let idx = (((x + _x) + (y + _y) * _width) * 4) as usize;
                    out[idx + 0] = lightmap.data[pos + per_cell + (_x + _y * 8) * 3 + 0] >> 4 << 4; // Posterisation
                    out[idx + 1] = lightmap.data[pos + per_cell + (_x + _y * 8) * 3 + 1] >> 4 << 4; // Posterisation
                    out[idx + 2] = lightmap.data[pos + per_cell + (_x + _y * 8) * 3 + 2] >> 4 << 4; // Posterisation
                    out[idx + 3] = lightmap.data[pos + (_x + _y * 8)];
                }
            }
        }
        return out;
    }

    fn create_shadowmap_image(width: usize,
                              height: usize,
                              surfaces: &Vec<Surface>,
                              tiles: &Vec<Tile>,
                              lightmap: &LightmapData) -> Vec<u8> {
        let per_cell = lightmap.per_cell as usize;
        let data = &lightmap.data;
        let mut out = vec![0; width * 8 * height * 8];

        for y in 0..height {
            for x in 0..width {
                let cell = &surfaces[y * width + x];
                if cell.tile_up > -1 {
                    let index = tiles[cell.tile_up as usize].light as usize * 4 * per_cell;

                    for i in 0..8 {
                        for j in 0..8 {
                            out[(x * 8 + i) + (y * 8 + j) * (width * 8)] = data[index + i + j * 8];
                        }
                    }
                } else {
                    // If no ground, shadow should be 1.0
                    for i in 0..8 {
                        for j in 0..8 {
                            out[(x * 8 + i) + (y * 8 + j) * (width * 8)] = 255;
                        }
                    }
                }
            }
        }

        return out;
    }

    fn create_tiles_color_image(width: usize,
                                height: usize,
                                surfaces: &Vec<Surface>,
                                tiles: &Vec<Tile>) -> Vec<u8> {
        let mut data = vec![0; width * height * 4];
        for y in 0..height {
            for x in 0..width {
                let cell = &surfaces[y * width + x];
                if cell.tile_up > -1 {
                    let color = tiles[cell.tile_up as usize].color;
                    let from = (y * width + x) as usize * 4;
                    let to = from + 4;
                    data[from..to].copy_from_slice(&color);
                }
            }
        }

        return data;
    }

    fn smooth_normal(width: usize,
                     height: usize,
                     surfaces: &Vec<Surface>,
                     tiles: &Vec<Tile>) -> Vec<[Vector3<f32>; 4]> {
        // Calculate normal for each cells
        let mut tmp: Vec<Vector3<f32>> = vec![Vector3::zeros(); width * height];
        let mut normals: Vec<[Vector3<f32>; 4]> = vec![
            [Vector3::zeros(), Vector3::zeros(), Vector3::zeros(), Vector3::zeros()];
            (width * height) as usize
        ];
        for y in 0..height {
            for x in 0..width {
                let cell = &surfaces[(y * width + x) as usize];
                if cell.tile_up > -1 {
                    let a: Vector3<f32> = Vector3::new(((x + 0) * 2) as f32, cell.height[0], ((y + 0) * 2) as f32);
                    let b: Vector3<f32> = Vector3::new(((x + 1) * 2) as f32, cell.height[1], ((y + 0) * 2) as f32);
                    let c: Vector3<f32> = Vector3::new(((x + 1) * 2) as f32, cell.height[3], ((y + 1) * 2) as f32);
                    let d: Vector3<f32> = Vector3::new(((x + 0) * 2) as f32, cell.height[2], ((y + 1) * 2) as f32);
                    let t1 = triangle_normal(&a, &b, &c);
                    let t2 = triangle_normal(&c, &d, &a);
                    tmp[(y * width + x) as usize] = (t1 + t2).normalize();
                }
            }
        }

        // Smooth normals
        let width = width as isize;
        let height = height as isize;

        fn or(tmp: &Vec<Vector3<f32>>, x: isize, y: isize, width: isize) -> Vector3<f32> {
            let i = (y * width + x) as usize;
            if x < 0 || y < 0 || tmp.len() <= i {
                Vector3::zeros()
            } else {
                tmp[(y * width + x) as usize]
            }
        }

        for y in 0..height {
            for x in 0..width {
                let mut n = &mut normals[(y * width + x) as usize];
                // Up Left
                n[0] = n[0] + tmp[((x + 0) + (y + 0) * width) as usize];
                n[0] = n[0] + or(&tmp, x - 1, y + 0, width);
                n[0] = n[0] + or(&tmp, (x - 1), (y - 1), width);
                n[0] = n[0] + or(&tmp, (x + 0), (y - 1), width);
                n[0].normalize_mut();

                // Up Right
                n[1] = n[1] + tmp[((x + 0) + (y + 0) * width) as usize];
                n[1] = n[1] + or(&tmp, (x + 1), (y + 0), width);
                n[1] = n[1] + or(&tmp, (x + 1), (y - 1), width);
                n[1] = n[1] + or(&tmp, (x + 0), (y - 1), width);
                n[1].normalize_mut();

                // Bottom Right
                n[2] = n[2] + tmp[((x + 0) + (y + 0) * width) as usize];
                n[2] = n[2] + or(&tmp, (x + 1), (y + 0), width);
                n[2] = n[2] + or(&tmp, (x + 1), (y + 1), width);
                n[2] = n[2] + or(&tmp, (x + 0), (y + 1), width);
                n[2].normalize_mut();

                // Bottom Left
                n[3] = n[3] + tmp[((x + 0) + (y + 0) * width) as usize];
                n[3] = n[3] + or(&tmp, (x - 1), (y + 0), width);
                n[3] = n[3] + or(&tmp, (x - 1), (y + 1), width);
                n[3] = n[3] + or(&tmp, (x + 0), (y + 1), width);
                n[3].normalize_mut();
            }
        }
        return normals;
    }

    fn load_lightmaps(buf: &mut BinaryReader) -> LightmapData {
        let count = buf.next_u32();
        let per_cell_x = buf.next_u32();
        let per_cell_y = buf.next_u32();
        let size_cell = buf.next_u32();
        let per_cell = per_cell_x * per_cell_y * size_cell;

        LightmapData {
            per_cell,
            count,
            data: buf.next(count * per_cell * 4),
        }
    }

    fn load_textures(buf: &mut BinaryReader) -> (Vec<String>, Vec<usize>) {
        let count = buf.next_u32();
        let len = buf.next_u32();

        let mut texture_index_map: BTreeMap<String, usize> = BTreeMap::new();
        let mut texture_indices: Vec<usize> = Vec::new();
        for i in 0..count {
            let name = buf.string(len);
            let current_texture_count = texture_index_map.len();
            let index = texture_index_map.entry(name.clone()).or_insert(current_texture_count);
            texture_indices.push(*index);
        }


        let texture_names: Vec<String> = texture_index_map
            .into_iter()
            .map(|e| e.0)
            .collect();

        (texture_names, texture_indices)
    }

    pub fn create_gl_texture_atlas(texture_names: &Vec<String>) -> GlTexture {
        let texture_surfaces: Vec<sdl2::surface::Surface> = texture_names.iter().map(|texture_name| {
                use sdl2::image::LoadSurface;
                let path = format!("d:\\Games\\TalonRO\\grf\\data\\texture\\{}", texture_name);
                sdl2::surface::Surface::from_file(path.clone()).unwrap_or_else(|_| {
                    println!("Missing: {}", path);
                    let mut missing_texture = sdl2::surface::Surface::new(256, 256, PixelFormatEnum::RGB888).unwrap();
                    let rect = missing_texture.rect();
                    missing_texture.fill_rect(rect, Color::RGB(255, 0, 255));
                    missing_texture
                })
            })
            .collect();
        let surface_atlas = Gnd::create_texture_atlas(&texture_surfaces);

        surface_atlas.save_bmp("shitaka.bmp");
        GlTexture::from_surface(surface_atlas)
    }

    fn create_texture_atlas(texture_surfaces: &Vec<sdl2::surface::Surface>) -> sdl2::surface::Surface<'static> {
        let _width = (texture_surfaces.len() as f32).sqrt().round() as i32;
        let width = ((_width * 258) as u32).next_power_of_two();
        let height = ((texture_surfaces.len() as f32).sqrt().ceil() as u32 * 258).next_power_of_two();
        let mut surface_atlas = sdl2::surface::Surface::new(width, height, PixelFormatEnum::RGB888).unwrap();
        for (i, texture_surface) in texture_surfaces.iter().enumerate() {
            let x = (i as i32 % _width) * 258;
            let y = ((i as i32 / _width) as f32).floor() as i32 * 258;
            texture_surface.blit_scaled(texture_surface.rect(),
                                        &mut surface_atlas,
                                        Rect::new(x, y, 258, 258),
            );
            texture_surface.blit_scaled(texture_surface.rect(),
                                        &mut surface_atlas,
                                        Rect::new(x + 1, y + 1, 256, 256),
            );
        }
        surface_atlas
    }
}

#[cfg(test)]
mod tests {
    use crate::rsw::Rsw;
    use crate::gat::Gat;
    use crate::common::BinaryReader;
    use crate::gnd::Gnd;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn it_adds_two() {
        let world = Rsw::load(BinaryReader::new(format!("d:\\Games\\TalonRO\\grf\\data\\{}.rsw", "new_zone01")));
        let altitude = Gat::load(BinaryReader::new(format!("d:\\Games\\TalonRO\\grf\\data\\{}.gat", "new_zone01")));
        let ground = Gnd::load(BinaryReader::new(format!("d:\\Games\\TalonRO\\grf\\data\\{}.gnd", "new_zone01")),
                               world.water.level,
                               world.water.wave_height);
        let mut content = String::with_capacity(8 * 1024 * 1024);
        File::open("tests/mesh.bin").unwrap().read_to_string(&mut content).unwrap();
        let floats: Vec<f32> = content.split(",").map(|line| {
            line.trim().parse::<f32>().unwrap()
        }).collect();

        let mesh_floats: Vec<f32> = unsafe {
            std::mem::transmute(ground.mesh)
        };
        floats.iter().zip(mesh_floats).enumerate().for_each(|(index, (a, b))| {
            assert!(*a - b < 0.000001, "{}.: {} != {}, ({} - {} = {})", index, a, b, a, b, *a - b);
        })
    }
}