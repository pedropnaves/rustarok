use crate::asset::str::{KeyFrameType, StrLayer};
use crate::components::BrowserClient;
use crate::systems::render::render_command::{
    EffectFrameCacheKey, RenderCommandCollectorComponent,
};
use crate::systems::render_sys::DamageRenderSystem;
use crate::systems::{SystemFrameDurations, SystemVariables};
use crate::video::{GlTexture, VertexArray, VertexAttribDefinition, Video, TEXTURE_0};
use nalgebra::{Matrix4, Rotation3, Vector3};
use sdl2::ttf::Sdl2TtfContext;
use specs::prelude::*;
use std::collections::HashMap;

pub struct OpenGlRenderSystem<'a, 'b> {
    centered_rectangle_vao: VertexArray,
    circle_vao: VertexArray,
    // damage rendering
    single_digit_u_coord: f32,
    texture_u_coords: [f32; 10],

    str_effect_cache: HashMap<EffectFrameCacheKey, Option<EffectFrameCache>>,
    text_cache: HashMap<String, GlTexture>,
    fonts: Fonts<'a, 'b>,
}

pub struct Fonts<'a, 'b> {
    small_font: sdl2::ttf::Font<'a, 'b>,
    normal_font: sdl2::ttf::Font<'a, 'b>,
    big_font: sdl2::ttf::Font<'a, 'b>,
    small_bold_font: sdl2::ttf::Font<'a, 'b>,
    normal_bold_font: sdl2::ttf::Font<'a, 'b>,
    big_bold_font: sdl2::ttf::Font<'a, 'b>,

    small_font_outline: sdl2::ttf::Font<'a, 'b>,
    normal_font_outline: sdl2::ttf::Font<'a, 'b>,
    big_font_outline: sdl2::ttf::Font<'a, 'b>,
    small_bold_font_outline: sdl2::ttf::Font<'a, 'b>,
    normal_bold_font_outline: sdl2::ttf::Font<'a, 'b>,
    big_bold_font_outline: sdl2::ttf::Font<'a, 'b>,
}

pub const SMALL_FONT_SIZE: i32 = 14;
pub const NORMAL_FONT_H: i32 = 20;
pub const NORMAL_FONT_W: i32 = 10;
pub const BIG_FONT_SIZE: i32 = 32;

impl<'a, 'b> Fonts<'a, 'b> {
    pub fn new(ttf_context: &'a Sdl2TtfContext) -> Fonts {
        let small_font = Video::load_font(
            &ttf_context,
            "assets/fonts/UbuntuMono-R.ttf",
            SMALL_FONT_SIZE as u16,
        )
        .unwrap();
        let mut small_font_outline = Video::load_font(
            &ttf_context,
            "assets/fonts/UbuntuMono-R.ttf",
            SMALL_FONT_SIZE as u16,
        )
        .unwrap();
        small_font_outline.set_outline_width(2);

        let normal_font = Video::load_font(
            &ttf_context,
            "assets/fonts/UbuntuMono-R.ttf",
            NORMAL_FONT_H as u16,
        )
        .unwrap();
        let mut normal_font_outline = Video::load_font(
            &ttf_context,
            "assets/fonts/UbuntuMono-R.ttf",
            NORMAL_FONT_H as u16,
        )
        .unwrap();
        normal_font_outline.set_outline_width(2);

        let big_font = Video::load_font(&ttf_context, "assets/fonts/UbuntuMono-R.ttf", 32).unwrap();
        let mut big_font_outline =
            Video::load_font(&ttf_context, "assets/fonts/UbuntuMono-R.ttf", 32).unwrap();
        big_font_outline.set_outline_width(2);

        let small_bold_font = Video::load_font(
            &ttf_context,
            "assets/fonts/UbuntuMono-B.ttf",
            SMALL_FONT_SIZE as u16,
        )
        .unwrap();
        let mut small_bold_font_outline = Video::load_font(
            &ttf_context,
            "assets/fonts/UbuntuMono-B.ttf",
            SMALL_FONT_SIZE as u16,
        )
        .unwrap();
        small_bold_font_outline.set_outline_width(2);

        let normal_bold_font = Video::load_font(
            &ttf_context,
            "assets/fonts/UbuntuMono-B.ttf",
            NORMAL_FONT_H as u16,
        )
        .unwrap();
        let mut normal_bold_font_outline = Video::load_font(
            &ttf_context,
            "assets/fonts/UbuntuMono-B.ttf",
            NORMAL_FONT_H as u16,
        )
        .unwrap();
        normal_bold_font_outline.set_outline_width(2);

        let big_bold_font = Video::load_font(
            &ttf_context,
            "assets/fonts/UbuntuMono-B.ttf",
            BIG_FONT_SIZE as u16,
        )
        .unwrap();
        let mut big_bold_font_outline = Video::load_font(
            &ttf_context,
            "assets/fonts/UbuntuMono-B.ttf",
            BIG_FONT_SIZE as u16,
        )
        .unwrap();
        big_bold_font_outline.set_outline_width(2);

        Fonts {
            small_font,
            normal_font,
            big_font,
            small_bold_font,
            normal_bold_font,
            big_bold_font,
            small_font_outline,
            normal_font_outline,
            big_font_outline,
            small_bold_font_outline,
            normal_bold_font_outline,
            big_bold_font_outline,
        }
    }
}

struct EffectFrameCache {
    pub pos_vao: VertexArray,
    pub offset: [f32; 2],
    pub color: [f32; 4],
    pub rotation_matrix: Matrix4<f32>,
    pub src_alpha: u32,
    pub dst_alpha: u32,
    pub texture_index: usize,
}

impl<'a, 'b> OpenGlRenderSystem<'a, 'b> {
    pub fn new(ttf_context: &Sdl2TtfContext) -> OpenGlRenderSystem {
        let single_digit_width = 10.0;
        let texture_width = single_digit_width * 10.0;
        let single_digit_u_coord = single_digit_width / texture_width;

        OpenGlRenderSystem {
            fonts: Fonts::new(ttf_context),
            single_digit_u_coord,
            texture_u_coords: [
                single_digit_u_coord * 0.0,
                single_digit_u_coord * 1.0,
                single_digit_u_coord * 2.0,
                single_digit_u_coord * 3.0,
                single_digit_u_coord * 4.0,
                single_digit_u_coord * 5.0,
                single_digit_u_coord * 6.0,
                single_digit_u_coord * 7.0,
                single_digit_u_coord * 8.0,
                single_digit_u_coord * 9.0,
            ],
            centered_rectangle_vao: {
                let bottom_left = v3!(-0.5, 0.0, -0.5);
                let top_left = v3!(-0.5, 0.0, 0.5);
                let top_right = v3!(0.5, 0.0, 0.5);
                let bottom_right = v3!(0.5, 0.0, -0.5);
                VertexArray::new(
                    gl::LINE_LOOP,
                    &[bottom_left, top_left, top_right, bottom_right],
                    4,
                    vec![VertexAttribDefinition {
                        number_of_components: 3,
                        offset_of_first_element: 0,
                    }],
                )
            },
            circle_vao: {
                let capsule_mesh = ncollide2d::procedural::circle(&1.0, 32);
                let coords: Vec<[f32; 3]> = capsule_mesh
                    .coords()
                    .iter()
                    .map(|it| [it.x, 0.0, it.y])
                    .collect();
                VertexArray::new(
                    gl::LINE_LOOP,
                    coords.as_slice(),
                    coords.len(),
                    vec![VertexAttribDefinition {
                        number_of_components: 3,
                        offset_of_first_element: 0,
                    }],
                )
            },
            str_effect_cache: HashMap::new(),
            text_cache: HashMap::with_capacity(1024),
        }
    }

    pub fn create_number_vertex_array(&self, number: u32) -> VertexArray {
        let digits = DamageRenderSystem::get_digits(number);
        // create vbo based on the numbers
        let mut width = 0.0;
        let mut vertices = vec![];
        digits.iter().for_each(|&digit| {
            let digit = digit as usize;
            vertices.push([width - 0.5, 0.5, self.texture_u_coords[digit], 0.0]);
            vertices.push([
                width + 0.5,
                0.5,
                self.texture_u_coords[digit] + self.single_digit_u_coord,
                0.0,
            ]);
            vertices.push([width - 0.5, -0.5, self.texture_u_coords[digit], 1.0]);
            vertices.push([
                width + 0.5,
                0.5,
                self.texture_u_coords[digit] + self.single_digit_u_coord,
                0.0,
            ]);
            vertices.push([width - 0.5, -0.5, self.texture_u_coords[digit], 1.0]);
            vertices.push([
                width + 0.5,
                -0.5,
                self.texture_u_coords[digit] + self.single_digit_u_coord,
                1.0,
            ]);
            width += 1.0;
        });
        return VertexArray::new(
            gl::TRIANGLES,
            &vertices,
            vertices.len(),
            vec![
                VertexAttribDefinition {
                    number_of_components: 2,
                    offset_of_first_element: 0,
                },
                VertexAttribDefinition {
                    // uv
                    number_of_components: 2,
                    offset_of_first_element: 2,
                },
            ],
        );
    }

    fn prepare_effect(layer: &StrLayer, key_index: i32) -> Option<EffectFrameCache> {
        let mut from_id = None;
        let mut to_id = None;
        let mut last_source_id = 0;
        let mut last_frame_id = 0;
        for (i, key_frame) in layer.key_frames.iter().enumerate() {
            if key_frame.frame <= key_index {
                match key_frame.typ {
                    KeyFrameType::Start => from_id = Some(i),
                    KeyFrameType::End => to_id = Some(i),
                };
            }
            last_frame_id = last_frame_id.max(key_frame.frame);
            if key_frame.typ == KeyFrameType::Start {
                last_source_id = last_source_id.max(key_frame.frame);
            }
        }
        if from_id.is_none() || to_id.is_none() || last_frame_id < key_index {
            return None;
        }
        let from_id = from_id.unwrap();
        let to_id = to_id.unwrap();
        if from_id >= layer.key_frames.len() || to_id >= layer.key_frames.len() {
            return None;
        }
        let from_frame = &layer.key_frames[from_id];
        let to_frame = &layer.key_frames[to_id];

        let (color, pos, xy, angle) = if to_id != from_id + 1 || to_frame.frame != from_frame.frame
        {
            // no other source
            if last_source_id <= from_frame.frame {
                return None;
            }
            (
                from_frame.color,
                from_frame.pos,
                from_frame.xy,
                from_frame.angle,
            )
        } else {
            let delta = (key_index - from_frame.frame) as f32;
            // morphing
            let color = [
                from_frame.color[0] + to_frame.color[0] * delta,
                from_frame.color[1] + to_frame.color[1] * delta,
                from_frame.color[2] + to_frame.color[2] * delta,
                from_frame.color[3] + to_frame.color[3] * delta,
            ];
            let xy = [
                from_frame.xy[0] + to_frame.xy[0] * delta,
                from_frame.xy[1] + to_frame.xy[1] * delta,
                from_frame.xy[2] + to_frame.xy[2] * delta,
                from_frame.xy[3] + to_frame.xy[3] * delta,
                from_frame.xy[4] + to_frame.xy[4] * delta,
                from_frame.xy[5] + to_frame.xy[5] * delta,
                from_frame.xy[6] + to_frame.xy[6] * delta,
                from_frame.xy[7] + to_frame.xy[7] * delta,
            ];
            let angle = from_frame.angle + to_frame.angle * delta;
            let pos = [
                from_frame.pos[0] + to_frame.pos[0] * delta,
                from_frame.pos[1] + to_frame.pos[1] * delta,
            ];
            (color, pos, xy, angle)
        };

        let offset = [pos[0] - 320.0, pos[1] - 320.0];

        return Some(EffectFrameCache {
            pos_vao: VertexArray::new(
                gl::TRIANGLE_STRIP,
                &[
                    [xy[0], xy[4], 0.0, 0.0],
                    [xy[1], xy[5], 1.0, 0.0],
                    [xy[3], xy[7], 0.0, 1.0],
                    [xy[2], xy[6], 1.0, 1.0],
                ],
                4,
                vec![
                    VertexAttribDefinition {
                        // xy
                        number_of_components: 2,
                        offset_of_first_element: 0,
                    },
                    VertexAttribDefinition {
                        // uv
                        number_of_components: 2,
                        offset_of_first_element: 2,
                    },
                ],
            ),
            offset,
            color,
            rotation_matrix: Rotation3::from_axis_angle(
                &nalgebra::Unit::new_normalize(Vector3::z()),
                -angle,
            )
            .to_homogeneous(),
            src_alpha: from_frame.src_alpha,
            dst_alpha: from_frame.dst_alpha,
            texture_index: from_frame.texture_index,
        });
        //        unsafe {
        //            gl::BlendFunc(from_frame.src_alpha, from_frame.dst_alpha);
        //        }
        //        //        str_file.textures[from_frame.texture_index].bind(TEXTURE_0);
        //        //        system_vars.str_effect_vao.bind().draw();
        //        unsafe {
        //            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        //            gl::Enable(gl::DEPTH_TEST);
        //        }
    }
}

impl<'a> specs::System<'a> for OpenGlRenderSystem<'_, '_> {
    type SystemData = (
        specs::ReadStorage<'a, RenderCommandCollectorComponent>,
        specs::ReadStorage<'a, BrowserClient>,
        specs::WriteExpect<'a, SystemFrameDurations>,
        specs::ReadExpect<'a, SystemVariables>,
    );

    fn run(
        &mut self,
        (render_commands_storage, browser_client_storage,mut system_benchmark, system_vars): Self::SystemData,
    ) {
        let _stopwatch = system_benchmark.start_measurement("OpenGlRenderSystem");

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        for (render_commands, _not_browser) in
            (&render_commands_storage, !&browser_client_storage).join()
        {
            let render_commands: &RenderCommandCollectorComponent = render_commands;

            /////////////////////////////////
            // 2D Trimesh
            /////////////////////////////////
            {
                let shader = system_vars.assets.shaders.trimesh2d_shader.gl_use();
                shader.set_mat4("projection", &system_vars.matrices.ortho);
                for trimesh_2d in &render_commands.trimesh_2d_commands {
                    // TODO: move bind out of the loop by grouping same vaos
                    shader.set_mat4("model", &trimesh_2d.matrix);
                    shader.set_vec4("color", &trimesh_2d.color);
                    shader.set_vec2("size", &trimesh_2d.size);
                    shader.set_f32("z", 0.01 * trimesh_2d.layer as usize as f32);
                    trimesh_2d.vao.bind().draw();
                }
            }

            /////////////////////////////////
            // 2D Texture
            /////////////////////////////////
            {
                let shader = system_vars.assets.shaders.sprite2d_shader.gl_use();
                shader.set_mat4("projection", &system_vars.matrices.ortho);
                shader.set_int("model_texture", 0);
                let vertex_array_bind = system_vars.map_render_data.sprite_vertex_array.bind();
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                }
                for command in &render_commands.texture_2d_commands {
                    let width = command.texture_width as f32;
                    let height = command.texture_height as f32;
                    unsafe {
                        gl::BindTexture(gl::TEXTURE_2D, command.texture.0);
                    }
                    shader.set_mat4("model", &command.matrix);
                    shader.set_f32("z", 0.01 * command.layer as usize as f32);
                    shader.set_vec2("offset", &command.offset);
                    shader.set_vec2("size", &[width * command.size, height * command.size]);
                    shader.set_vec4("color", &command.color);
                    vertex_array_bind.draw();
                }
            }

            /////////////////////////////////
            // 2D Rectangle
            /////////////////////////////////
            {
                let vertex_array_bind = system_vars.map_render_data.sprite_vertex_array.bind();
                let shader = system_vars.assets.shaders.trimesh2d_shader.gl_use();
                shader.set_mat4("projection", &system_vars.matrices.ortho);
                for command in &render_commands.rectangle_2d_commands {
                    shader.set_vec4("color", &command.color);
                    shader.set_mat4("model", &command.matrix);
                    shader.set_vec2("size", &command.size);
                    shader.set_f32("z", 0.01 * command.layer as usize as f32);
                    vertex_array_bind.draw();
                }
            }

            /////////////////////////////////
            // 2D Text
            /////////////////////////////////
            {
                let shader = system_vars.assets.shaders.sprite2d_shader.gl_use();
                shader.set_mat4("projection", &system_vars.matrices.ortho);
                shader.set_int("model_texture", 0);
                let vertex_array_bind = system_vars.map_render_data.sprite_vertex_array.bind();
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                }
                for command in &render_commands.text_2d_commands {
                    let texture = self
                        .text_cache
                        .entry(command.text.clone()) // TODO: why clone ?
                        .or_insert(Video::create_text_texture(
                            &self.fonts.normal_font,
                            &command.text,
                        ));

                    let width = texture.width as f32;
                    let height = texture.height as f32;
                    unsafe {
                        gl::BindTexture(gl::TEXTURE_2D, texture.id().0);
                    }
                    shader.set_mat4("model", &command.matrix);
                    shader.set_f32("z", 0.01 * command.layer as usize as f32);
                    shader.set_vec2("offset", &[0.0, 0.0]);
                    shader.set_vec2("size", &[width * command.size, height * command.size]);
                    shader.set_vec4("color", &command.color);
                    vertex_array_bind.draw();
                }
            }

            /////////////////////////////////
            // 3D Rectangle
            /////////////////////////////////
            {
                let centered_rectangle_vao_bind = self.centered_rectangle_vao.bind();
                let shader = system_vars.assets.shaders.trimesh_shader.gl_use();
                shader.set_mat4("projection", &system_vars.matrices.projection);
                shader.set_mat4("view", &render_commands.view_matrix);
                for command in &render_commands.rectangle_3d_commands {
                    shader.set_vec4("color", &command.common.color);
                    shader.set_mat4("model", &command.common.matrix);
                    shader.set_vec2("size", &command.size);
                    centered_rectangle_vao_bind.draw();
                }
            }

            /////////////////////////////////
            // 3D Circles
            /////////////////////////////////
            {
                let vao_bind = self.circle_vao.bind();
                let shader = system_vars.assets.shaders.trimesh_shader.gl_use();
                shader.set_mat4("projection", &system_vars.matrices.projection);
                shader.set_mat4("view", &render_commands.view_matrix);
                for command in &render_commands.circle_3d_commands {
                    shader.set_vec4("color", &command.common.color);
                    shader.set_mat4("model", &command.common.matrix);
                    shader.set_vec2(
                        "size",
                        &[command.common.size * 2.0, command.common.size * 2.0],
                    );
                    vao_bind.draw();
                }
            }

            /////////////////////////////////
            // 3D Sprites
            /////////////////////////////////
            {
                let shader = system_vars.assets.shaders.sprite_shader.gl_use();
                let vao_bind = system_vars
                    .map_render_data
                    .centered_sprite_vertex_array
                    .bind();
                shader.set_mat4("projection", &system_vars.matrices.projection);
                shader.set_mat4("view", &render_commands.view_matrix);
                shader.set_int("model_texture", 0);
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                }
                for command in &render_commands.billboard_commands {
                    shader.set_vec2("size", &[command.texture_width, command.texture_height]);
                    shader.set_mat4("model", &command.common.matrix);
                    shader.set_vec4("color", &command.common.color);
                    shader.set_vec2("offset", &command.common.offset);

                    unsafe {
                        gl::BindTexture(gl::TEXTURE_2D, command.texture.0);
                    }
                    vao_bind.draw();
                }
            }

            /////////////////////////////////
            // NUMBERS
            /////////////////////////////////
            {
                unsafe {
                    gl::Disable(gl::DEPTH_TEST);
                }
                let shader = system_vars.assets.shaders.sprite_shader.gl_use();
                shader.set_mat4("projection", &system_vars.matrices.projection);
                shader.set_mat4("view", &render_commands.view_matrix);
                shader.set_int("model_texture", 0);
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                }
                system_vars.assets.sprites.numbers.bind(TEXTURE_0);
                for command in &render_commands.number_3d_commands {
                    shader.set_vec2("size", &[command.common.size, command.common.size]);
                    shader.set_mat4("model", &command.common.matrix);
                    shader.set_vec4("color", &command.common.color);
                    shader.set_vec2("offset", &command.common.offset);

                    self.create_number_vertex_array(command.value).bind().draw();
                }
                unsafe {
                    gl::Enable(gl::DEPTH_TEST);
                }
            }

            /////////////////////////////////
            // EFFECTS
            /////////////////////////////////
            {
                unsafe {
                    gl::Disable(gl::DEPTH_TEST);
                }
                let shader = system_vars.assets.shaders.str_effect_shader.gl_use();
                shader.set_mat4("projection", &system_vars.matrices.projection);
                shader.set_mat4("view", &render_commands.view_matrix);
                shader.set_int("model_texture", 0);
                unsafe {
                    gl::ActiveTexture(gl::TEXTURE0);
                }

                for (command, commands) in &render_commands.effect_commands {
                    let cached_frame = self.str_effect_cache.get(&command);
                    let str_file = &system_vars.map_render_data.str_effects[&command.effect_name];
                    if let None = cached_frame {
                        let layer = &str_file.layers[command.layer_index];
                        let cached_effect_frame =
                            OpenGlRenderSystem::prepare_effect(layer, command.key_index);
                        self.str_effect_cache
                            .insert(command.clone(), cached_effect_frame);
                    } else if let Some(None) = cached_frame {
                        continue;
                    } else if let Some(Some(cached_frame)) = cached_frame {
                        shader.set_vec2("offset", &cached_frame.offset);
                        shader.set_vec4("color", &cached_frame.color);
                        unsafe {
                            gl::BlendFunc(cached_frame.src_alpha, cached_frame.dst_alpha);
                        }
                        str_file.textures[cached_frame.texture_index].bind(TEXTURE_0);
                        let bind = cached_frame.pos_vao.bind();
                        for matrix in commands {
                            shader.set_mat4("model", &(matrix * cached_frame.rotation_matrix));
                            bind.draw();
                        }
                    }
                }

                unsafe {
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                    gl::Enable(gl::DEPTH_TEST);
                }
            }

            /////////////////////////////////
            // MODELS
            /////////////////////////////////
            {
                let map_render_data = &system_vars.map_render_data;
                let shader = system_vars.assets.shaders.model_shader.gl_use();
                shader.set_mat4("projection", &system_vars.matrices.projection);
                shader.set_mat4("view", &render_commands.view_matrix);
                shader.set_mat3("normal_matrix", &render_commands.normal_matrix);
                shader.set_int("model_texture", 0);
                shader.set_vec3("light_dir", &map_render_data.rsw.light.direction);
                shader.set_vec3("light_ambient", &map_render_data.rsw.light.ambient);
                shader.set_vec3("light_diffuse", &map_render_data.rsw.light.diffuse);
                shader.set_f32("light_opacity", map_render_data.rsw.light.opacity);
                shader.set_int("use_lighting", map_render_data.use_lighting as i32);

                for render_command in &render_commands.model_commands {
                    shader.set_mat4("model", &render_command.matrix);
                    shader.set_f32("alpha", render_command.alpha);
                    let model_render_data = &map_render_data.models[&render_command.name];
                    for node_render_data in &model_render_data.model {
                        // TODO: optimize this
                        for face_render_data in node_render_data {
                            face_render_data.texture.bind(TEXTURE_0);
                            face_render_data.vao.bind().draw();
                        }
                    }
                }
            }
        }
    }
}
