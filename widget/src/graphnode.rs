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
    fn draw_node_core(&mut self, cx: &mut Cx, renderable_area: &mut Rect) {}

    fn handle_node_core(&mut self, cx: &mut Cx, event: &mut Event) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphNodeCoreType {
    #[serde(rename = "none ")]
    None,
    #[serde(rename = "debug")]
    Debug(DebugNode),
}

impl GraphNodeCore for GraphNodeCoreType {
    fn draw_node_core(&mut self, cx: &mut Cx, renderable_area: &mut Rect) {
        match self {
            GraphNodeCoreType::Debug(n) => n.draw_node_core(cx, renderable_area),
            _ => (),
        }
    }

    fn handle_node_core(&mut self, cx: &mut Cx, event: &mut Event) {
        match self {
            GraphNodeCoreType::Debug(n) => n.handle_node_core(cx, event),
            _ => (),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoopNode {}
impl GraphNodeCore for NoopNode {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugNode {}
impl GraphNodeCore for DebugNode {}

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
            core: GraphNodeCoreType::None,
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
