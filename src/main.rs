use std::str::FromStr;

use drawing_rs::{
	color::Rgba,
	frame::{self, Frame, FrameBuilder},
	pixels::SurfaceTexture,
	winit::{
		event::{Event, WindowEvent},
		event_loop::ControlFlow,
	},
};

fn draw(frame: &mut Frame, index: usize)
{
	frame.set_by_index(
		index + (frame.width() * index as u32) as usize,
		Rgba::from_str("#ff0000").unwrap(),
	);
}

fn main()
{
	let event_loop = frame::default_event_loop();
	let window = frame::default_window(800, 600, "Hello, World", &event_loop);
	let surface = SurfaceTexture::new(
		window.inner_size().width,
		window.inner_size().width,
		&window,
	);

	let mut frame = match FrameBuilder::new(800, 600)
		.with_event_loop(&event_loop)
		.with_surface(surface)
		.with_background(Rgba::from_str("#fff").unwrap())
		.build()
	{
		Ok(v) => v,
		Err(e) => panic!("{}", e),
	};

	let mut index: usize = 0;

	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Poll;

		match event {
			Event::WindowEvent { event: e, .. } => match e {
				WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
				WindowEvent::Resized(new_size) => frame
					.resize_surface(new_size.width, new_size.height)
					.unwrap(),
				_ => {}
			},
			Event::RedrawRequested(_) => {
				draw(&mut frame, index);
				index += 1;
				if frame.render().is_err() {
					*control_flow = ControlFlow::Exit;
				}
			}
			_ => {}
		};

		window.request_redraw();
	});
}
