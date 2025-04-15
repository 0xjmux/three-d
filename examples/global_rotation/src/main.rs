// attempting to rotate an object in 3d space

// Entry point for non-wasm
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run().await;
}

// use three_d::egui::*;
use cgmath::{self, Euler};
use three_d::*;

#[derive(Debug, Default)]
pub struct Appstate {
    pitch_deg: f32,
    roll_deg: f32,
}

pub async fn run() {
    let window = Window::new(WindowSettings {
        title: "Global Rotation Demo".to_string(),
        max_size: Some((800, 500)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.125, 0.0, -0.5),
        // vec3(0.125, -0.25, -0.5),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.01,
        100.0,
    );
    let mut control = OrbitControl::new(camera.target(), 0.1, 3.0);

    // Load point cloud .pcd file
    // let mut loaded = three_d_asset::io::load_async(&["hand.pcd"])
    //     .await
    //     .unwrap();
    // let cpu_point_cloud: PointCloud = loaded.deserialize("hand.pcd").unwrap();

    let mut point_mesh = CpuMesh::sphere(4);
    point_mesh.transform(Mat4::from_scale(0.001)).unwrap();

    let axes = Axes::new(&context, 0.01, 0.1);
    let c = -axes.aabb().center();
    let mut axes_mesh = Gm {
        // geometry: Mesh::new(axes.into(), &point_mesh),
        geometry: axes,
        material: ColorMaterial::default(),
    };

    // find translation between view center and axes location
    axes_mesh.set_transformation(Mat4::from_translation(c));

    let mut app = Appstate::default();

    let mut gui = three_d::GUI::new(&context);

    // these axes don't move (maybe they'd be better in a different color)
    let static_axes = Axes::new(&context, 0.01, 0.1);
    let mut static_axes_mesh = Gm {
        geometry: static_axes,
        material: ColorMaterial::default(),
    };
    static_axes_mesh.set_transformation(Mat4::from_translation(c));

    // main loop
    window.render_loop(move |mut frame_input| {
        const VPRT_BOUND: f32 = 90.0;
        let mut panel_width = 0.0;
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                use three_d::egui::*;
                SidePanel::left("side_panel").show(gui_context, |ui| {
                    ui.heading("Debug Panel");

                    ui.add(
                        Slider::new(&mut app.pitch_deg, -VPRT_BOUND..=VPRT_BOUND).text("Pitch (°)"),
                    );
                    ui.add(
                        Slider::new(&mut app.roll_deg, -VPRT_BOUND..=VPRT_BOUND).text("Roll (°)"),
                    );

                    ui.label(format!("Camera View: {:?}", camera.position()));
                    ui.end_row();

                    // ui.label(format!("pitch={:?}", pitch_matrix));

                    ui.separator();
                });
                panel_width = gui_context.used_rect().width();
            },
        );

        // camera.position()

        // where camera goes
        let viewport = Viewport {
            x: (panel_width * frame_input.device_pixel_ratio) as i32,
            y: 0,
            width: frame_input.viewport.width
                - (panel_width * frame_input.device_pixel_ratio) as u32,
            height: frame_input.viewport.height,
        };

        camera.set_viewport(viewport);

        // camera drag control
        control.handle_events(&mut camera, &mut frame_input.events);

        let rotation = Euler {
            x: Deg(app.pitch_deg),
            y: Deg::zero(),
            z: Deg(app.roll_deg),
        };

        let combined_rotation = Quaternion::from(rotation);

        // my different attempts at getting this working
        // let combined_rotation = (pitch_matrix* roll_matrix).normalize();
        // let transform = Mat4::from(combined_rotation) * Mat4::from_translation(c);
        // let transform = Matrix4::from(combined_rotation);
        axes_mesh.set_transformation(combined_rotation.into());

        // Main view
        frame_input
            .screen()
            .clear(ClearState::color_and_depth(1.0, 1.0, 1.0, 1.0, 1.0))
            .clear_partially(
                viewport.into(),
                ClearState::color_and_depth(1.0, 1.0, 1.0, 1.0, 1.0),
            )
            .render_partially(
                viewport.into(),
                &camera,
                // &axes_mesh,
                axes_mesh.into_iter().chain(&static_axes_mesh),
                &[],
            )
            .write(|| gui.render())
            .unwrap();

        FrameOutput::default()
    })
}
