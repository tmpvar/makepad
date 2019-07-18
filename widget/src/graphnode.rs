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
                println!("draw_node_core!");
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
            color: color("#FFF"),
            shader: cx.add_dynamic_shader("some junk", CxDynamicShader {
                needs_rebuild: true,
                source: self.source.clone(),
                ..Default::default()
            })
        });
    }

     fn draw_node_core(&mut self, cx: &mut Cx, renderable_area: &mut Rect) {
         if let Some(bg) = &mut self.bg {
             println!("DRAWING DEBUG CORE!!!");
             bg.draw_quad_abs(cx, *renderable_area);
         }
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
