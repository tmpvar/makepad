use crate::graphnodeport::*;
use render::*;
use serde::*;
use uuid::Uuid;

#[derive(Clone, PartialEq)]
pub enum GraphNodeEvent {
    None,
    DragMove {
        fe: FingerMoveEvent,
    },
    DragEnd {
        fe: FingerUpEvent,
    },
    DragOut,
    PortDragMove {
        port_id: Uuid,
        port_dir: PortDirection,
        fe: FingerMoveEvent,
    },
    PortDrop,
    PortDropHit {
        port_id: Uuid,
        port_dir: PortDirection,
    },
    PortDropMiss,
}

/*
  .....................
  .       A NODE      .
  .....................
 (IN) --        -- (OUT)
  .      \    /       .
 (IN) -- (CORE)       .
  .      /    \       .
 (IN) --        -- (OUT)
  ......................
*/

pub trait GraphNodeCore {
    fn construct(&mut self, cx: &mut Cx);
    fn draw_node_core(&mut self, cx: &mut Cx, renderable_area: &mut Rect) {
        println!("default draw_node_core");
    }

    fn handle_node_core(&mut self, cx: &mut Cx, event: &mut Event) {}
}

#[derive(Clone, Serialize, Deserialize)]
pub enum GraphNodeCoreType {
    #[serde(rename = "none ")]
    None,
    #[serde(rename = "debug")]
    Debug(DebugNode),
}

impl GraphNodeCore for GraphNodeCoreType {
    fn draw_node_core(&mut self, cx: &mut Cx, renderable_area: &mut Rect) {
        match self {
            GraphNodeCoreType::Debug(n) => {
                n.draw_node_core(cx, renderable_area);
            }
            _ =>  println!("draw_node_core missed"),
        }
    }

    fn handle_node_core(&mut self, cx: &mut Cx, event: &mut Event) {
        match self {
            GraphNodeCoreType::Debug(n) => n.handle_node_core(cx, event),
            _ => (),
        }
    }

    fn construct(&mut self, cx: &mut Cx) {
        match self {
            GraphNodeCoreType::Debug(n) => n.construct(cx),
            _ => (),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NoopNode {}
impl GraphNodeCore for NoopNode {
    fn construct(&mut self, cx: &mut Cx) {
        println!("construct noop");
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DebugNode {
    source: String,
    #[serde(
        skip_serializing,
        skip_deserializing,
    )]
    bg: Option<Quad>,
}

fn default_debug_source() -> String {
    String::from("
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
            //Pixel shader
            float _df_calc_blur(float w, _Tex _tex, thread _Loc &_loc, thread _Vary &_vary, device _UniCx &_uni_cx, device _UniVw &_uni_vw, device _UniDr &_uni_dr){
            float wa = clamp(-w*_loc.df_aa, 0.0, 1.0);
            float wb = 1.0;
            if(_loc.df_blur>0.001){
                wb = clamp(-w/_loc.df_blur, 0.0, 1.0);
            };
            return wa*wb;
            }
            float4 _df_fill_keep(float4 color, _Tex _tex, thread _Loc &_loc, thread _Vary &_vary, device _UniCx &_uni_cx, device _UniVw &_uni_vw, device _UniDr &_uni_dr){
            float f = _df_calc_blur(_loc.df_shape, _tex, _loc, _vary, _uni_cx, _uni_vw, _uni_dr);
            float4 source = float4(color.rgb*color.a, color.a);
            _loc.df_result = source*f+_loc.df_result*(1.0-source.a*f);
            if(_loc.df_has_clip>0.5){
                float f2 = 1.0-_df_calc_blur(-_loc.df_clip, _tex, _loc, _vary, _uni_cx, _uni_vw, _uni_dr);
                _loc.df_result = source*f2+_loc.df_result*(1.0-source.a*f2);
            };
            return _loc.df_result;
            }
            float4 _df_stroke_keep(float4 color, float width, _Tex _tex, thread _Loc &_loc, thread _Vary &_vary, device _UniCx &_uni_cx, device _UniVw &_uni_vw, device _UniDr &_uni_dr){
                float f = _df_calc_blur(abs(_loc.df_shape)-width/_loc.df_scale, _tex, _loc, _vary, _uni_cx, _uni_vw, _uni_dr);
                float4 source = float4(color.rgb*color.a, color.a);
                float4 dest = _loc.df_result;
                _loc.df_result = source*f+dest*(1.0-source.a*f);
                return _loc.df_result;
            }
            float _df_antialias(float2 p, _Tex _tex, thread _Loc &_loc, thread _Vary &_vary, device _UniCx &_uni_cx, device _UniVw &_uni_vw, device _UniDr &_uni_dr){
                return 1.0/length(float2(length(dfdx(p)), length(dfdy(p))));
            }
            float4 _df_fill(float4 color, _Tex _tex, thread _Loc &_loc, thread _Vary &_vary, device _UniCx &_uni_cx, device _UniVw &_uni_vw, device _UniDr &_uni_dr){
                _df_fill_keep(color, _tex, _loc, _vary, _uni_cx, _uni_vw, _uni_dr);
                _loc.df_old_shape = _loc.df_shape = 100000000000000000000.0;
                _loc.df_clip = -100000000000000000000.0;
                _loc.df_has_clip = 0.0;
                return _loc.df_result;
            }
            void _df_circle(float x, float y, float r, _Tex _tex, thread _Loc &_loc, thread _Vary &_vary, device _UniCx &_uni_cx, device _UniVw &_uni_vw, device _UniDr &_uni_dr){
            float2 c = _loc.df_pos-float2(x, y);
                _loc.df_field = (length(c.xy)-r)/_loc.df_scale;
                _loc.df_old_shape = _loc.df_shape;
                _loc.df_shape = min(_loc.df_shape, _loc.df_field);
            }
            float4 _df_stroke(float4 color, float width, _Tex _tex, thread _Loc &_loc, thread _Vary &_vary, device _UniCx &_uni_cx, device _UniVw &_uni_vw, device _UniDr &_uni_dr){
                _df_stroke_keep(color, width, _tex, _loc, _vary, _uni_cx, _uni_vw, _uni_dr);
                _loc.df_old_shape = _loc.df_shape = 100000000000000000000.0;
                _loc.df_clip = -100000000000000000000.0;
                _loc.df_has_clip = 0.0;
                return _loc.df_result;
            }
            void _df_line_to(float x, float y, _Tex _tex, thread _Loc &_loc, thread _Vary &_vary, device _UniCx &_uni_cx, device _UniVw &_uni_vw, device _UniDr &_uni_dr){
                float2 p = float2(x, y);
                float2 pa = _loc.df_pos-_loc.df_last_pos;
                float2 ba = p-_loc.df_last_pos;
                float h = clamp(dot(pa, ba)/dot(ba, ba), 0.0, 1.0);
                float s = sign(pa.x*ba.y-pa.y*ba.x);
                _loc.df_field = length(pa-ba*h)/_loc.df_scale;
                _loc.df_old_shape = _loc.df_shape;
                _loc.df_shape = min(_loc.df_shape, _loc.df_field);
                _loc.df_clip = max(_loc.df_clip, _loc.df_field*s);
                _loc.df_has_clip = 1.0;
                _loc.df_last_pos = p;
            }
            void _df_move_to(float x, float y, _Tex _tex, thread _Loc &_loc, thread _Vary &_vary, device _UniCx &_uni_cx, device _UniVw &_uni_vw, device _UniDr &_uni_dr){
                _loc.df_last_pos = _loc.df_start_pos = float2(x, y);
            }
            float2 _df_viewport(float2 pos, _Tex _tex, thread _Loc &_loc, thread _Vary &_vary, device _UniCx &_uni_cx, device _UniVw &_uni_vw, device _UniDr &_uni_dr){
                _loc.df_pos = pos;
                _loc.df_result = float4(0.0, 0.0, 0.0, 0.0);
                _loc.df_old_shape = _loc.df_shape = 100000000000000000000.0;
                _loc.df_clip = -100000000000000000000.0;
                _loc.df_blur = 0.00001;
                _loc.df_aa = _df_antialias(pos, _tex, _loc, _vary, _uni_cx, _uni_vw, _uni_dr);
                _loc.df_scale = 1.0;
                _loc.df_field = 0.0;
                _loc.df_clip = 0.0;
                _loc.df_has_clip = 0.0;
                return _loc.df_pos;
            }
            float4 _pixel(_Tex _tex, thread _Loc &_loc, thread _Vary &_vary, device _UniCx &_uni_cx, device _UniVw &_uni_vw, device _UniDr &_uni_dr){
                
                return float4(_vary.pos.x, 0, _vary.pos.y, 1);
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
            fragment float4 _fragment_shader(_Vary _vary[[stage_in]],_Tex _tex,
            device _UniCx &_uni_cx [[buffer(0)]], device _UniVw &_uni_vw [[buffer(1)]], device _UniDr &_uni_dr [[buffer(2)]]){
            _Loc _loc;
            return _pixel(_tex, _loc, _vary, _uni_cx, _uni_vw, _uni_dr);
            };
        "
    )
}

impl GraphNodeCore for DebugNode {
    fn construct(&mut self, cx: &mut Cx) {
        println!("construct debugnode");
        if self.source == "" {
            self.source = default_debug_source()
        }

        // add a test shader that needs to be rebuilt
        self.bg = Some(Quad {
            do_scroll: false,
            color: color("#F00"),
            shader: cx.add_dynamic_shader("some junk", CxDynamicShader {
                needs_rebuild: true,
                source: self.source.clone(),
                ..Default::default()
            })
        });        
    }

     fn draw_node_core(&mut self, cx: &mut Cx, renderable_area: &mut Rect) {
        // if let Err(()) = self.view.begin_view(cx, Layout::default()) {
        //     return
        // }
         
        if let Some(bg) = &mut self.bg {
             
            println!("DRAWING DEBUG CORE!!! {}", bg.shader.shader_id.unwrap());
            let rect = Rect {
                x: renderable_area.x + 7.,
                y: renderable_area.y + 7.,
                w: renderable_area.w - 14.,
                h: renderable_area.h - 14.,
            };

            let inst = bg.draw_quad_abs(cx, rect);
        }
        
        // self.view.end_view(cx);
        // self.view.redraw_view_area(cx);
        //cx.redraw_child_area(Area::All);
     }
}

impl Default for DebugNode {
    fn default() -> Self {
        Self {
            source: default_debug_source(),
            bg: None,
        }
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub aabb: Rect,
    pub id: Uuid,

    pub core: GraphNodeCoreType,

    pub inputs: Vec<GraphNodePort>,
    pub outputs: Vec<GraphNodePort>,

    #[serde(
        skip_serializing,
        skip_deserializing,
        default = "build_default_animator"
    )]
    pub animator: Animator,
}

impl Style for GraphNode {
    fn style(cx: &mut Cx) -> Self {
        Self {
            aabb: Rect {
                x: 100.0,
                y: 100.0,
                w: 200.0,
                h: 100.0,
            },
            id: Uuid::new_v4(),
            animator: Animator::new(Anim::empty()),
            core: GraphNodeCoreType::Debug(DebugNode::default()),
            inputs: vec![],
            outputs: vec![],
        }
    }
}

fn build_default_animator() -> Animator {
    Animator::new(Anim::empty())
}

impl GraphNode {
    pub fn get_port_address(
        &self,
        dir: PortDirection,
        index: usize,
    ) -> Option<GraphNodePortAddress> {
        let port_id: Uuid;

        // TODO: ensure the thing exists before blindly using it.
        match dir {
            PortDirection::Input => port_id = self.inputs[index].id,
            PortDirection::Output => port_id = self.outputs[index].id,
        }

        Some(GraphNodePortAddress {
            node: self.id.clone(),
            port: port_id,
            dir: dir,
        })
    }

    pub fn get_port_by_address(&self, addr: &GraphNodePortAddress) -> Option<&GraphNodePort> {
        match addr.dir {
            PortDirection::Input => {
                for input in &self.inputs {
                    if input.id == addr.port {
                        return Some(input);
                    }
                }
            }
            PortDirection::Output => {
                for output in &self.outputs {
                    if output.id == addr.port {
                        return Some(output);
                    }
                }
            }
        }
        None
    }

    pub fn draw_graph_node(&mut self, cx: &mut Cx, bg: &mut Quad, port_bg: &mut Quad) {
        let aabb = self.aabb;
        let mut core_aabb = aabb.clone();
        self.core.draw_node_core(cx, &mut core_aabb);

        let inst = bg.draw_quad_abs(cx, aabb);

        // TODO: eliminate all of these hardcoded offsets. maybe there is
        // value in defining sub views for inputs/outputs
        let mut y = 5.0;
        for input in &mut self.inputs {
            let rect = Rect {
                x: aabb.x - 10.0,
                y: aabb.y + y,
                w: 20.0,
                h: 20.0,
            };
            input.draw(cx, port_bg, rect);
            y += 20.0;
        }

        y = 5.0;
        for output in &mut self.outputs {
            let rect = Rect {
                x: aabb.w + aabb.x - 10.0,
                y: aabb.y + y,
                w: 20.0,
                h: 20.0,
            };
            output.draw(cx, port_bg, rect);
            y += 20.0;
        }

        self.animator.update_area_refs(cx, inst.clone().into_area());
    }

    pub fn construct_graph_node (&mut self, cx: &mut Cx) {
        self.core.construct(cx);
    }

    pub fn handle_graph_node(
        &mut self,
        cx: &mut Cx,
        event: &mut Event,
        skip: &Option<Uuid>,
    ) -> GraphNodeEvent {
        self.core.handle_node_core(cx, event);
        for input in &mut self.inputs {
            match input.handle(cx, event) {
                GraphNodePortEvent::DragMove { fe } => {
                    return GraphNodeEvent::PortDragMove {
                        port_id: input.id,
                        port_dir: PortDirection::Input,
                        fe: fe,
                    };
                }
                GraphNodePortEvent::DragEnd { fe } => {
                    return GraphNodeEvent::PortDrop;
                }
                GraphNodePortEvent::DropHit => {
                    return GraphNodeEvent::PortDropHit {
                        port_id: input.id,
                        port_dir: PortDirection::Input,
                    };
                }
                GraphNodePortEvent::DropMiss => {
                    return GraphNodeEvent::PortDropMiss;
                }
                _ => (),
            }
        }

        for output in &mut self.outputs {
            match output.handle(cx, event) {
                GraphNodePortEvent::DragMove { fe } => {
                    return GraphNodeEvent::PortDragMove {
                        port_id: output.id,
                        port_dir: PortDirection::Output,
                        fe: fe,
                    };
                }
                GraphNodePortEvent::DragEnd { fe } => {
                    return GraphNodeEvent::PortDrop;
                }
                GraphNodePortEvent::DropHit => {
                    return GraphNodeEvent::PortDropHit {
                        port_id: output.id,
                        port_dir: PortDirection::Output,
                    };
                }
                GraphNodePortEvent::DropMiss => {
                    return GraphNodeEvent::PortDropMiss;
                }
                _ => (),
            }
        }

        match event.hits(cx, self.animator.area, HitOpt::default()) {
            Event::Animate(ae) => {
                self.animator
                    .write_area(cx, self.animator.area, "bg.", ae.time);
            }
            Event::FingerUp(fe) => {
                return GraphNodeEvent::DragEnd { fe: fe.clone() };
            }
            Event::FingerMove(fe) => {
                self.aabb.x = fe.abs.x - fe.rel_start.x;
                self.aabb.y = fe.abs.y - fe.rel_start.y;

                return GraphNodeEvent::DragMove { fe: fe.clone() };
            }
            _ => (),
        }
        GraphNodeEvent::None
    }
}
