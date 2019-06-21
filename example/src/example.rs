/// This is Makepad, a work-in-progress livecoding IDE for 2D Design.
// This application is nearly 100% Wasm running on webGL. NO HTML AT ALL.
// The vision is to build a livecoding / design hybrid program,
// here procedural design and code are fused in one environment.
// If you have missed 'learnable programming' please check this out:
// http://worrydream.com/LearnableProgramming/
// Makepad aims to fulfill (some) of these ideas using a completely
// from-scratch renderstack built on the GPU and Rust/wasm.
// It will be like an IDE meets a vector designtool, and had offspring.
// Direct manipulation of the vectors modifies the code, the code modifies the vectors.
// And the result can be lasercut, embroidered or drawn using a plotter.
// This means our first goal is 2D path CNC with booleans (hence the CAD),
// and not dropshadowy-gradients.

// Find the repo and more explanation at github.com/makepad/makepad.
// We are developing the UI kit and code-editor as MIT, but the full stack
// will be a cloud/native app product in a few months.

// However to get to this amazing mixed-mode code editing-designtool,
// we first have to build an actually nice code editor (what you are looking at)
// And a vector stack with booleans (in progress)
// Up next will be full multiplatform support and more visual UI.
// All of the code is written in Rust, and it compiles to native and Wasm
// Its built on a novel immediate-mode UI architecture
// The styling is done with shaders written in Rust, transpiled to metal/glsl

// for now enjoy how smooth a full GPU editor can scroll (try select scrolling the edge)
// Also the tree fold-animates and the docking panel system works.
// Multi cursor/grid cursor also works with ctrl+click / ctrl+shift+click
// press alt or escape for animated codefolding outline view!

use render::*;
use widget::*;
pub mod cap_winmf;
pub use crate::cap_winmf::*;

struct App {
    view: View<ScrollBar>,
    window: DesktopWindow,
    buttons: Elements<u64, Button, Button>,
    text: Text,
    quad: Quad,
    blit: Blit,
    cap:CapWinMF,
    clickety: u64
}

main_app!(App, "Example");

impl Style for App {
    fn style(cx: &mut Cx) -> Self {
        set_dark_style(cx);
        Self {
            cap:CapWinMF::default(),
            window:DesktopWindow::style(cx),
            view: View {
                scroll_h: Some(ScrollBar::style(cx)),
                scroll_v: Some(ScrollBar::style(cx)),
                ..Style::style(cx)
            },
            quad: Quad {
                shader: cx.add_shader(App::def_quad_shader(), "App.quad"),
                ..Style::style(cx)
            },
            text: Text ::style(cx),
            blit: Blit::style(cx),
            buttons: Elements::new(Button {
                ..Style::style(cx)
            }),
            clickety: 0
        }
    }
}

impl App {
    fn def_quad_shader() -> ShaderGen {
        Quad::def_quad_shader().compose(shader_ast!({
            fn pixel() -> vec4 {
                df_viewport(pos * vec2(w, h));
                df_rect(0., 0., w, h);
                df_fill(vec4(pos.x, pos.y, sin(pos.x), 1.));
                df_move_to(0., 0.);
                df_line_to(w, h);
                return df_stroke(color, 4.);
            }
        }))
    }
    
    fn handle_app(&mut self, cx: &mut Cx, event: &mut Event) {
        
        if let Event::Construct = event{
            self.cap.init(cx, 0, 3840, 2160, 30.0);
        }
        
        self.window.handle_desktop_window(cx, event);
        
        self.view.handle_scroll_bars(cx, event);
        
        if let CapEvent::NewImage =  self.cap.handle_signal(cx, event){
            self.view.redraw_view_area(cx);
        }
        /*
        for btn in self.buttons.iter() {
            match btn.handle_button(cx, event) {
                ButtonEvent::Clicked => {
                    // boop
                    self.clickety += 1;
                    cx.redraw_child_area(Area::All);
                },
                _ => ()
            }
        }*/
    }
    
    fn draw_app(&mut self, cx: &mut Cx) {
        
        let _ = self.window.begin_desktop_window(cx);
        
        let _ =  self.view.begin_view(cx, Layout {
            padding: Padding {l: 0., t: 0., r: 0., b: 0.},
            ..Default::default()
        });
        self.blit.draw_blit_walk(cx, &self.cap.texture, Bounds::Fill, Bounds::Fill, Margin::zero());
        /*
        for i in 0..100 {
            
            self.buttons.get_draw(cx, i, | _cx, templ | {templ.clone()})
                .draw_button_with_label(cx, &format!("Btn {}", i));
            
            if i % 10 == 9 {
                cx.turtle_new_line()
            }
        }*/
        
        //cx.turtle_new_line();
        //self.text.draw_text(cx, &format!("It works {}", self.clickety));
        //self.quad.draw_quad_walk(cx, Bounds::Fix(100.), Bounds::Fix(100.), Margin {l: 15., t: 0., r: 0., b: 0.});
        
        self.view.end_view(cx);
        
        self.window.end_desktop_window(cx);
    }
}