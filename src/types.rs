mod position
{
	#[derive(Clone, Copy, Default, Debug)]
	pub(crate) struct Position
	{
		pub(crate) x: i32,
		pub(crate) y: i32,
	}

	impl Position
	{
		pub(crate) fn new(x: i32, y: i32) -> Self
		{
			Position { x, y }
		}
	}
}

pub(crate) mod tree
{
	use super::position::Position;
	use rand::{thread_rng as rng, Rng};
	use raylib::color::Color;

	const MAGIC_NUMBER: i32 = 10_000;

	#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
	pub(crate) enum TreeState
	{
		Alive,
		Burning(i32),
		Dead(i32),
		Lightning(i32),
		#[default]
		None,
	}

	#[derive(Clone, Copy, Default, Debug)]
	pub(crate) struct Tree
	{
		pub(crate) age: i32,
		pub(crate) position: Position,
		pub(crate) state: TreeState,
	}

	impl TreeState
	{
		pub(crate) fn _random() -> Self
		{
			let num = rng().gen_range(0..=3);
			match num {
				0 => TreeState::Alive,
				1 => TreeState::Burning(0),
				2 => TreeState::Dead(0),
				3 => TreeState::None,
				_ => panic!(),
			}
		}
	}

	impl Tree
	{
		pub(crate) fn _new() -> Self
		{
			Tree {
				age: 0,
				position: Position::new(0, 0),
				state: TreeState::None,
			}
		}

		pub(crate) fn is_ignited(&self) -> bool
		{
			matches!(self.state, TreeState::Burning(_))
		}

		pub(crate) fn draw(&self) -> raylib::color::Color
		{
			match self.state {
				TreeState::Alive => match self.age {
					0..10_000 => Color::new(25, 160, 40, 255),
					10_000..20_000 => Color::new(35, 110, 40, 255),
					20_000..30_000 => Color::new(55, 80, 40, 255),
					_ => Color::new(65, 75, 25, 255),
				},
				TreeState::Burning(_) => Color::new(180, 120, 70, 255),
				TreeState::Dead(_) => Color::new(40, 40, 35, 255),
				TreeState::None => Color::new(15, 30, 15, 255),
				TreeState::Lightning(_) => Color::BLUE,
			}
		}

		pub(crate) fn update(mut self, neighbors: &[&Tree]) -> Self
		{
			match &mut self.state {
				TreeState::Lightning(v) => {
					if *v >= 3 {
						self.state = TreeState::Burning(0)
					} else {
						*v += 1
					}
				}
				TreeState::Burning(v) => {
					if *v >= 5 {
						self.state = TreeState::Dead(0);
					} else {
						*v += 1;
					}
				}
				TreeState::Dead(d) => {
					if rng().gen_range(0..10000) < 5 {
						self.state = TreeState::None;
						return self;
					}
					if *d > 500 {
						let rng = rng().gen::<f64>();
						let age: f64 = self.age.into();
						let regrowth_change = (age / 100f64).ln();

						if rng > regrowth_change {
							self.state = TreeState::Alive;
							self.age = 0;
						}
					} else {
						*d += 1;
					}
				}
				TreeState::Alive => {
					let lightning = neighbors
						.iter()
						.any(|tree| matches!(tree.state, TreeState::Lightning(_)));
					if lightning {
						self.state = TreeState::Burning(0);
						return self;
					}
					let burning_neighbors =
						neighbors.iter().filter(|tree| tree.is_ignited()).count() as i32;
					if burning_neighbors > 1 {
						let burn_percentage: i32 = ((100.0 / 8.0)
							* (burning_neighbors + (self.age / MAGIC_NUMBER * 2)) as f64)
							as i32;
						if rng().gen_range(0..100) < burn_percentage {
							self.state = TreeState::Burning(0);
						}
					} else if self.age >= 2000 {
						let r = rng().gen::<f64>();
						let age: f64 = self.age.into();
						let death_chance = (age / 100f64).ln();

						if r > death_chance {
							self.state = TreeState::Dead(0);
						}
					}
					self.age += 1
				}
				TreeState::None => {
					if rng().gen_range(0..10000) < 5 {
						self.state = TreeState::Alive;
					}
				}
			}
			self
		}
	}
}

pub(crate) mod tree_builder
{
	use super::position::Position;
	use super::tree::{Tree, TreeState};

	pub(crate) struct TreeBuilder
	{
		age: i32,
		position: Position,
		state: TreeState,
	}

	impl TreeBuilder
	{
		#[inline(always)]
		pub(crate) fn new() -> Self
		{
			TreeBuilder {
				age: 0,
				position: Position::new(0, 0),
				state: TreeState::None,
			}
		}

		#[inline(always)]
		pub(crate) fn with_age(mut self, age: i32) -> Self
		{
			self.age = age;
			self
		}

		#[inline(always)]
		pub(crate) fn with_position(mut self, x: i32, y: i32) -> Self
		{
			self.position = Position::new(x, y);
			self
		}

		#[inline(always)]
		pub(crate) fn with_state(mut self, state: TreeState) -> Self
		{
			self.state = state;
			self
		}

		#[inline(always)]
		pub(crate) fn build(self) -> Tree
		{
			Tree {
				age: self.age,
				position: self.position,
				state: self.state,
			}
		}
	}
}

pub(crate) mod forest
{
	use super::tree::{self, TreeState};
	use rand::seq::SliceRandom;
	use rand::thread_rng as rng;

	pub(crate) struct Forest
	{
		ping_pong: bool,
		trees_one: Vec<tree::Tree>,
		trees_two: Vec<tree::Tree>,
		pub(crate) width: i32,
		pub(crate) height: i32,
	}

	#[inline(always)]
	fn get_neighbors(width: usize, idx: usize) -> Vec<usize>
	{
		let row = idx / width;
		let col = idx % width;

		let mut neighbors = Vec::with_capacity(8);

		let offsets = [
			(-1, -1),
			(-1, 0),
			(-1, 1),
			(0, -1),
			(0, 1),
			(1, -1),
			(1, 0),
			(1, 1),
		];

		for &(dr, dc) in &offsets {
			let new_row = row as isize + dr;
			let new_col = col as isize + dc;

			if new_row >= 0 && new_row < width as isize && new_col >= 0 && new_col < width as isize
			{
				let new_idx = (new_row * width as isize + new_col) as usize;
				if new_idx != idx {
					neighbors.push(new_idx);
				}
			}
		}

		neighbors
	}

	pub(crate) fn neighboring_trees(
		trees: &[tree::Tree],
		idx: usize,
		width: usize,
	) -> Vec<&tree::Tree>
	{
		get_neighbors(width, idx)
			.into_iter()
			.flat_map(|tree_index| trees.get(tree_index))
			.collect()
	}

	impl Forest
	{
		pub(crate) fn new(width: i32, height: i32) -> Self
		{
			Forest {
				ping_pong: false,
				trees_one: Vec::with_capacity((width * height) as usize),
				trees_two: Vec::with_capacity((width * height) as usize),
				width,
				height,
			}
		}

		pub(crate) fn trees_and_buffer(&mut self) -> (&[tree::Tree], &mut [tree::Tree])
		{
			if !self.ping_pong {
				(&self.trees_one, &mut self.trees_two)
			} else {
				(&self.trees_two, &mut self.trees_one)
			}
		}

		pub(crate) fn trees_mut(&mut self) -> &mut [tree::Tree]
		{
			if !self.ping_pong {
				&mut self.trees_one
			} else {
				&mut self.trees_two
			}
		}

		pub(crate) fn add_tree(&mut self, tree: tree::Tree)
		{
			self.trees_one.push(tree);
			self.trees_two.push(tree);
		}

		pub(crate) fn ignite_random_tree(&mut self)
		{
			let mut binding = self
				.trees_mut()
				.iter_mut()
				.filter(|t| matches!(t.state, tree::TreeState::Alive))
				.collect::<Vec<_>>();

			let tree = binding.choose_mut(&mut rng());

			match tree {
				Some(t) => t.state = TreeState::Lightning(0),
				None => todo!(),
			}
		}

		pub(crate) fn draw(&self) -> Box<[u8]>
		{
			let mut colors: Vec<u8> = vec![0; (self.width * self.height * 4) as usize];

			for tree in &self.trees_one {
				let color = tree.draw();
				let index = (tree.position.y * self.width + tree.position.x) as usize * 4;
				if index + 3 < colors.len() {
					colors[index] = color.r;
					colors[index + 1] = color.g;
					colors[index + 2] = color.b;
					colors[index + 3] = color.a;
				}
			}

			colors.into_boxed_slice()
		}

		pub(crate) fn update(&mut self)
		{
			let width = self.width as usize;
			let (trees, buffer) = self.trees_and_buffer();
			for (idx, tree) in trees.iter().enumerate() {
				let updated_tree = tree.update(neighboring_trees(trees, idx, width).as_slice());
				buffer[idx] = updated_tree;
			}
			self.ping_pong = !self.ping_pong
		}
	}

	impl std::ops::AddAssign<tree::Tree> for Forest
	{
		#[inline(always)]
		fn add_assign(&mut self, rhs: tree::Tree)
		{
			self.add_tree(rhs);
		}
	}
}
