mod types;

use raylib::color::Color;
use raylib::prelude::*;
use raylib::texture::Image;

use types::forest;
use types::tree;
use types::tree_builder;

use std::time;

use rand::{thread_rng as rng, Rng};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

fn main()
{
	let (mut rl, thread) = raylib::init().size(WIDTH, HEIGHT).title("Arson").build();

	rl.set_target_fps(144);

	let mut forest = forest::Forest::new(WIDTH, HEIGHT);

	for index in 0..WIDTH * HEIGHT {
		let x = index % WIDTH;
		let y = index / WIDTH;

		forest += tree_builder::TreeBuilder::new()
			.with_position(x, y)
			.with_state(tree::TreeState::Alive)
			.with_age(rng().gen_range(0..3))
			.build()
	}

	let mut forest_image = Image::gen_image_color(WIDTH, HEIGHT, Color::WHITE);
	let mut texture = rl
		.load_texture_from_image(&thread, &forest_image)
		.expect("Failed to create texture from image");

	let mut time_keeper = time::Duration::from_secs(9);
	while !rl.window_should_close() {
		let now = time::Instant::now();
		forest.update();

		time_keeper += now.elapsed();

		if time_keeper.ge(&time::Duration::from_secs(10)) {
			time_keeper = time::Duration::ZERO;
			forest.ignite_random_tree();
		}

		let pixels = forest.draw();

		let screen_width = rl.get_screen_width();
		let screen_height = rl.get_screen_height();

		let scale_x = screen_width as f32 / WIDTH as f32;
		let scale_y = screen_height as f32 / HEIGHT as f32;

		forest_image.resize(screen_width, screen_height);

		texture.update_texture(&pixels);

		let mut d = rl.begin_drawing(&thread);
		d.clear_background(Color::WHITE);
		d.draw_texture_ex(
			&texture,
			Vector2::new(0.0, 0.0),
			0.0,
			scale_x.min(scale_y),
			Color::WHITE,
		);
		d.draw_fps(0, 0);
	}
}
