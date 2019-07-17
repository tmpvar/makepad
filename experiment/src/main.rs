//use syn::Type;
use render::*;
use widget::*;
mod style;

mod graph;
use crate::graph::Graph;

use std::collections::HashMap;
use serde::*;
use uuid::Uuid;

#[derive(Clone)]
struct AppWindow {
    desktop_window: DesktopWindow,
    graph: Graph,
}

struct AppGlobal {
    state: AppState,
    index_file_read: FileRead,
    app_state_file_read: FileRead,
}

struct App {
    app_window_state_template: AppWindowState,
    app_window_template: AppWindow,
    app_global: AppGlobal,
    windows: Vec<AppWindow>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AppWindowState {
    window_position: Vec2,
    window_inner_size: Vec2,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct AppState {
    windows: Vec<AppWindowState>
}

main_app!(App, "HALLO WORLD");

impl Style for AppWindow {
    fn style(cx: &mut Cx) -> Self {
        Self {
            desktop_window: DesktopWindow::style(cx),
            graph: Graph{
                ..Style::style(cx)
            }
        }
    }
}

impl Style for App {

    fn style(cx: &mut Cx) -> Self {
        style::set_experiment_style(cx);
        Self {
            app_window_template: AppWindow::style(cx),
            app_window_state_template: AppWindowState {
                window_inner_size: Vec2::zero(),
                window_position: Vec2::zero(),
            },
            windows: vec![],
            app_global: AppGlobal {
                index_file_read: FileRead::default(),
                app_state_file_read: FileRead::default(),
                state: AppState::default()
            }
        }
    }
}

impl AppWindow {
    fn handle_app_window(&mut self, cx: &mut Cx, event: &mut Event, window_index: usize, app_global: &mut AppGlobal) {
        self.graph.handle_graph(cx, event);
        match self.desktop_window.handle_desktop_window(cx, event) {
            DesktopWindowEvent::EventForOtherWindow => {
                return
            },
            DesktopWindowEvent::WindowClosed => {
                return
            },
            DesktopWindowEvent::WindowGeomChange(wc) => {
                println!("new pos {:?}", wc.new_geom.position);
                if !app_global.app_state_file_read.is_pending() {
                    // store our new window geom
                    app_global.state.windows[window_index].window_position = wc.new_geom.position;
                    app_global.state.windows[window_index].window_inner_size = wc.new_geom.inner_size;
                    app_global.save_state(cx);
                }
            },
            _ => ()
        }


    }

    fn draw_app_window(&mut self, cx: &mut Cx, window_index: usize, app_global: &mut AppGlobal) {
        if let Err(()) = self.desktop_window.begin_desktop_window(cx) {
            return
        }

        // self.dock.draw_dock(cx);
        self.graph.draw_graph(cx);
        self.desktop_window.end_desktop_window(cx);
    }
}

impl AppGlobal {
    fn handle_construct(&mut self, cx: &mut Cx) {
        // add a test shader that needs to be rebuilt
        cx.dynamic_shader_map.insert(Uuid::new_v4(), CxDynamicShader {
            name: String::from("some junk"),
            needs_rebuild: true,
            source: String::from("
                #include <metal_stdlib>
                using namespace metal;
                struct _Geom{
                    packed_float2 geom;
                };

                struct _Inst{
                    float x;
                    float y;
                    float w;
                    float h;
                    packed_float4 color;
                    packed_float2 start;
                    packed_float2 end;
                };

                struct _UniCx{
                    float4x4 camera_projection;
                    float dpi_factor;
                    float dpi_dilate;
                };

                struct _UniVw{
                    float2 view_scroll;
                    float4 view_clip;
                };

                struct _UniDr{
                    float view_do_scroll;
                };

                struct _Loc{
                    float2 df_pos;
                    float4 df_result;
                    float2 df_last_pos;
                    float2 df_start_pos;
                    float df_shape;
                    float df_clip;
                    float df_has_clip;
                    float df_old_shape;
                    float df_blur;
                    float df_aa;
                    float df_scale;
                    float df_field;
                };

                struct _Tex{
                };

                #define  PI (3.141592653589793)
                #define  E (2.718281828459045)
                #define  LN2 (0.6931471805599453)
                #define  LN10 (2.302585092994046)
                #define  LOG2E (1.4426950408889634)
                #define  LOG10E (0.4342944819032518)
                #define  SQRT1_2 (0.7071067811865476)
                #define  TORAD (0.017453292519943295)
                #define  GOLDEN (1.618033988749895)
                struct _Vary{
                    float4 mtl_position [[position]];
                    float2 pos;
                    float w;
                    float h;
                    float2 start;
                    float2 end;
                    float4 color;
                };

                //Vertex shader
                float4 _vertex(_Tex _tex, thread _Loc &_loc, thread _Vary &_vary, thread _Geom &_geom, thread _Inst &_inst, device _UniCx &_uni_cx, device _UniVw &_uni_vw, device _UniDr &_uni_dr){
                    float2 shift = -_uni_vw.view_scroll*_uni_dr.view_do_scroll;
                    float2 clipped = clamp(float2(_geom.geom)*float2(float(_inst.w), float(_inst.h))+float2(float(_inst.x), float(_inst.y))+shift, _uni_vw.view_clip.xy, _uni_vw.view_clip.zw);
                    _vary.pos = (clipped-shift-float2(float(_inst.x), float(_inst.y)))/float2(float(_inst.w), float(_inst.h));
                    return float4(clipped.x, clipped.y, 0.0, 1.0)*_uni_cx.camera_projection;
                }

                vertex _Vary _vertex_shader(_Tex _tex, device _Geom *in_geometries [[buffer(0)]], device _Inst *in_instances [[buffer(1)]],
                device _UniCx &_uni_cx [[buffer(2)]], device _UniVw &_uni_vw [[buffer(3)]], device _UniDr &_uni_dr [[buffer(4)]],
                uint vtx_id [[vertex_id]], uint inst_id [[instance_id]]){
                    _Loc _loc;
                    _Vary _vary;
                    _Geom _geom = in_geometries[vtx_id];
                    _Inst _inst = in_instances[inst_id];
                    _vary.mtl_position = _vertex(_tex, _loc, _vary, _geom, _inst, _uni_cx, _uni_vw, _uni_dr);

                    _vary.w = _inst.w;
                    _vary.h = _inst.h;
                    _vary.start = _inst.start;
                    _vary.end = _inst.end;
                    _vary.color = _inst.color;
                    return _vary;
                };
                fragment float4 _fragment_shader(){
                    return float4(1, 0, 1, 1);
                };                
                
                "
            ),
            ..Default::default()
        });

    }

    fn save_state(&mut self, cx: &mut Cx) {
        println!("SAVE STATE");
        let json = serde_json::to_string(&self.state).unwrap();
        cx.file_write(&format!("{}experiment_state.json", "./".to_string()), json.as_bytes());
    }
}

impl App {
    fn handle_app(&mut self, cx: &mut Cx, event: &mut Event) {
        match event {
            Event::Construct => {
                self.app_global.handle_construct(cx);
                self.app_global.app_state_file_read = cx.file_read(&format!("{}experiment_state.json", "./".to_string()));
            },
            Event::FileRead(fr) => {
                if let Some(utf8_data) = self.app_global.app_state_file_read.resolve_utf8(fr) {
                    if let Ok(utf8_data) = utf8_data {
                        if let Ok(state) = serde_json::from_str(&utf8_data) {
                            self.app_global.state = state;

                            // create our windows with the serialized positions/size
                            for window_state in &self.app_global.state.windows {
                                let mut size = window_state.window_inner_size;

                                if size.x <= 10. {
                                    size.x = 800.;
                                }
                                if size.y <= 10. {
                                    size.y = 600.;
                                }
                                let last_pos = window_state.window_position;
                                let create_pos = Some(last_pos);
                                self.windows.push(AppWindow {
                                    desktop_window: DesktopWindow {window: Window {
                                        create_inner_size: Some(size),
                                        create_position: create_pos,
                                        ..Style::style(cx)
                                    }, ..Style::style(cx)},
                                    ..self.app_window_template.clone()
                                })
                            }
                            cx.redraw_child_area(Area::All);
                        }
                    }
                    else { // load default window
                        println!("DOING DEFAULT");
                        self.app_global.state.windows = vec![self.app_window_state_template.clone()];
                        self.windows = vec![self.app_window_template.clone()];

                        cx.redraw_child_area(Area::All);
                    }
                    for (window_index, window) in self.windows.iter_mut().enumerate() {
                        window.handle_app_window(cx, &mut Event::Construct, window_index, &mut self.app_global);
                    }
                }
            },
            _ => (),
        }

        for (window_index, window) in self.windows.iter_mut().enumerate() {
            window.handle_app_window(cx, event, window_index, &mut self.app_global);
        }
    }

    fn draw_app(&mut self, cx: &mut Cx) {
        for (window_index, window) in self.windows.iter_mut().enumerate() {
            window.draw_app_window(cx, window_index, &mut self.app_global);
        }
    }
}
