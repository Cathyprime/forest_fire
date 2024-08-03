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

	#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
	pub(crate) enum TreeState
	{
		Alive,
		Burning(i32),
		Dead(i32),
		#[default]
		None,
	}

	#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
	pub(crate) enum Burning
	{
		Yes(i32),
		#[default]
		No,
	}

	impl Burning
	{
		pub(crate) fn new(yes: bool, state: i32) -> Self
		{
			if yes {
				Self::Yes(state)
			} else {
				Self::No
			}
		}
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
				TreeState::Alive => Color::GREEN,
				TreeState::Burning(_) => Color::ORANGE,
				TreeState::Dead(_) => Color::BROWN,
				TreeState::None => Color::WHITE,
			}
		}

		pub(crate) fn should_spread(&self) -> Burning
		{
			let r = rng().gen_range(1..=10);
			if let TreeState::Burning(x) = self.state {
				match x {
					1..4 => Burning::No,
					4..6 => Burning::new(r > 8, x+1),
					6..8 => Burning::new(r > 6, x+2),
					8..10 => Burning::new(r > 4, x+3),
					10..=20 => Burning::new(r > 2, x+4),
					_ => Burning::Yes(x+5),
				}
			} else {
				Burning::No
			}
		}

		pub(crate) fn ignite(mut self, burn_time: i32) -> Self
		{
			if self.state == TreeState::Alive {
				let rng = rng().gen::<f64>();
				let age: f64 = self.age as f64;
				let resistance = (age / 1000f64).ln();

				if rng > resistance {
					self.state = TreeState::Burning(burn_time);
				}
			}
			self
		}

		pub(crate) fn update(&mut self)
		{
			self.age += 1;
			match &mut self.state {
				TreeState::Burning(v) => {
					if *v >= 40 {
						self.state = TreeState::Dead(0);
					} else {
						*v += 1;
					}
				}
				TreeState::Dead(d) => {
					if *d > 300 {
						let rng = rng().gen::<f64>();
						let age: f64 = self.age as f64;
						let regrowth_change = (age / 100f64).ln();

						if rng > regrowth_change {
							self.state = TreeState::Alive;
							self.age = 0;
						}
					} else {
						*d += 1;
					}
				}
				_ => (),
			}
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
		pub(crate) trees: Vec<tree::Tree>,
		pub(crate) width: i32,
		pub(crate) height: i32,
	}

	impl Forest
	{
		pub(crate) fn new(width: i32, height: i32) -> Self
		{
			Forest {
				trees: Vec::with_capacity(width as usize * height as usize),
				width,
				height,
			}
		}

		pub(crate) fn add_tree(&mut self, tree: tree::Tree)
		{
			self.trees.push(tree);
		}

		pub(crate) fn ignite_random_tree(&mut self)
		{
			let mut binding = self
				.trees
				.iter_mut()
				.filter(|t| match t.state {
					tree::TreeState::Alive => true,
					tree::TreeState::Burning(_) => false,
					tree::TreeState::Dead(_) => true,
					tree::TreeState::None => true,
				})
				.collect::<Vec<_>>();

			let tree = binding.choose_mut(&mut rng());

			match tree {
				Some(t) => t.state = TreeState::Burning(0),
				None => todo!(),
			}
		}

		pub(crate) fn draw(&self) -> Box<[u8]>
		{
			let mut colors: Vec<u8> = vec![0; (self.width * self.height * 4) as usize];

			for tree in &self.trees {
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
			let mut changes = Vec::new();

			for idx in 0..self.trees.len() {
				if let tree::Burning::Yes(x) = self.trees[idx].should_spread() {
					for neighbor_index in vec![
						idx.checked_sub(width - 1),
						idx.checked_sub(width),
						idx.checked_sub(width + 1),
						idx.checked_sub(1),
						idx.checked_add(1),
						idx.checked_add(width - 1),
						idx.checked_add(width),
						idx.checked_add(1 + width),
					]
					.into_iter()
					.flatten()
					{
						if let Some(tree) = self.trees.get(neighbor_index) {
							if !tree.is_ignited() {
								changes.push((neighbor_index, x));
							}
						}
					}
				}
			}

			for (index, burn_time) in changes {
				if let Some(tree) = self.trees.get(index) {
					self.trees[index] = (*tree).ignite(burn_time);
				}
			}

			self.trees.iter_mut().for_each(|tree| tree.update());
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
