#[macro_use]
extern crate conrod;
use conrod::{widget, Borderable, Colorable, Labelable, Positionable, Sizeable, Widget};
use conrod::backend::glium::glium;
use conrod::backend::glium::glium::{DisplayBuild, Surface};
use std::num::Wrapping;

// nb_live = number of live neighbour among the 8 surrounding cells
// Conway rules:
// * Cell survives if:
// < 2 => die
// == 2 => survive
// > 3 => die
// == 3 => survive 
//
// * Cell is born if:
// == 3 become alive

fn conway_survives() -> Vec<usize> {
	vec![2,3]
}

fn conway_borns() -> Vec<usize> {
	vec![3]
}

#[derive(Debug)]
struct Board {
	width: usize,
	height: usize,
	data: Vec<bool>,
	survives_by_count: [bool; 10], // note that the center cell is included in the count, so index is #neighbor+1
	borns_by_count: [bool; 9],
}

impl Board {
	fn new( width: usize, height: usize, survives: &Vec<usize>, borns: &Vec<usize> ) -> Board {
		let mut survives_by_count = [false; 10];
		for count in survives.iter() {
			survives_by_count[count+1] = true; // center cell is included in the count, hence the +1
		}
		let mut borns_by_count = [false; 9];
		for count in borns.iter() {
			borns_by_count[count+0] = true;
		}
		
		Board {
			width: width, 
			height: height,
			data: vec![true; width* height],
			survives_by_count: survives_by_count,
			borns_by_count: borns_by_count,
		}
	}
	
	fn get(&self, x: usize, y: usize) -> bool {
		assert!( x <= self.width  &&  y <= self.height );
		self.data[ y * self.width + x ]
	}
	
	fn set(&mut self, x: usize, y: usize, new_value: bool) {
		assert!( x <= self.width  &&  y <= self.height );
		self.data[ y * self.width + x ] = new_value;
	}
	
	fn advance_simulation(&mut self) {
//		println!("Begin simulation board={:?}", self);
		let mut next_board: Vec<bool> = Vec::with_capacity( self.data.len() );
		for y_test in 0..self.height {
			for x_test in 0..self.width {
				next_board.push( self.cell_next_state(x_test, y_test) );
			}
		}
		assert!( next_board.len() == self.data.len() );
		self.data = next_board;
//		println!("End simulation board={:?}", self);
	}
	
	fn cell_next_state(&self, x: usize, y: usize) -> bool {
		let nb_neighbor = self.count_neighbor(x,y);
		if self.get(x, y) {
			self.survives_by_count[nb_neighbor + 1]
		} else {
			self.borns_by_count[nb_neighbor]
		}
	}
	
	fn cell_alive(&self, x: usize, y: usize) -> usize {
		((x < self.width  &&  y < self.height)  &&  self.data[x + y*self.width]) as usize
	}
	
	fn count_neighbor( &self, x: usize, y: usize ) -> usize {
		let Wrapping(x_minus_1) = Wrapping(x) - Wrapping(1);
		let Wrapping(y_minus_1) = Wrapping(y) - Wrapping(1);
		self.cell_alive(x_minus_1, y_minus_1) + self.cell_alive(x, y_minus_1) + self.cell_alive(x+1, y_minus_1) + 
		self.cell_alive(x_minus_1, y) + self.cell_alive(x+1, y) +
		self.cell_alive(x_minus_1, y+1) + self.cell_alive(x, y+1) + self.cell_alive(x+1, y+1)
	}
	
}

#[derive(Debug)]
struct AppState {
	board: Board,
	simulating: bool,
}

impl AppState {
	fn new() -> AppState {
		const BOARD_WIDTH: usize = 8;
		const BOARD_HEIGHT: usize = 8;
		
		AppState {
			board: Board::new(BOARD_WIDTH, BOARD_HEIGHT, &conway_survives(), &conway_borns()),
			simulating: false,
		}
	}
}


fn main() {
	const WIDTH: u32 = 400;
	const HEIGHT: u32 = 400;
	
	let mut app_state = AppState::new();
//	println!("Initial app_state={:?}", app_state);

	// Build the window.
	let display = glium::glutin::WindowBuilder::new()
		.with_vsync()
		.with_dimensions(WIDTH, HEIGHT)
		.with_title("Game of Life")
		.with_multisampling(4)
		.build_glium()
		.unwrap();

	// construct our `Ui`.
	let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

	// Generate the widget identifiers.
	widget_ids!(struct Ids { 
		title, 
		info,
		board,
		start_stop_button
	});
	let ids = Ids::new(ui.widget_id_generator());

	// Add a `Font` to the `Ui`'s `font::Map` from file.
	const FONT_PATH: &'static str =
		concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/NotoSans/NotoSans-Regular.ttf");
	ui.fonts.insert_from_file(FONT_PATH).unwrap();

	// A type used for converting `conrod::render::Primitives` into `Command`s that can be used
	// for drawing to the glium `Surface`.
	let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

	// The image map describing each of our widget->image mappings (in our case, none).
	let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

	// Poll events from the window.
	let mut last_update = std::time::Instant::now();
	let mut ui_needs_update = true;
	'main: loop {

		// We don't want to loop any faster than 60 FPS, so wait until it has been at least
		// 16ms since the last yield.
		let sixteen_ms = std::time::Duration::from_millis(16);
		let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
		if duration_since_last_update < sixteen_ms {
			std::thread::sleep(sixteen_ms - duration_since_last_update);
		}

		// Collect all pending events.
		let mut events: Vec<_> = display.poll_events().collect();

		// If there are no events and the `Ui` does not need updating, wait for the next event.
		if events.is_empty() && !ui_needs_update {
			events.extend(display.wait_events().next());
		}

		// Reset the needs_update flag and time this update.
		ui_needs_update = false;
		last_update = std::time::Instant::now();

		// Handle all events.
		for event in events {

			// Use the `winit` backend feature to convert the winit event to a conrod one.
			if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
				ui.handle_event(event);
				ui_needs_update = true;
			}

			match event {
				// Break from the loop upon `Escape`.
				glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::Escape)) |
				glium::glutin::Event::Closed =>
					break 'main,
				_ => {},
			}
		}
		
		if app_state.simulating {
			app_state.board.advance_simulation();
		}

		// Instantiate all widgets in the GUI.
		{
			let ui = &mut ui.set_widgets();

			// "Hello World!" in the middle of the screen.
			widget::Text::new("Game of Life")
				.mid_top_of(ui.window)
				.color(conrod::color::WHITE)
				.font_size(32)
				.set(ids.title, ui);

			let start_stop_label = if app_state.simulating { "Pause simulation" } else { "Start simulation"};
			let start_stop_button = widget::Button::new()
				.label(start_stop_label)
				.down_from(ids.title, 5.0)
				.set(ids.start_stop_button, ui);
			for _press in start_stop_button {
				app_state.simulating = !app_state.simulating;
			}


			// A demonstration using widget_matrix to easily draw a matrix of any kind of widget.
			let (cols, rows) = (8, 8);
			let mut elements = widget::Matrix::new(cols, rows)
				.down(20.0)
				.w_h(260.0, 260.0)
				.set(ids.board, ui);

			// The `Matrix` widget returns an `Elements`, which can be used similar to an `Iterator`.
			while let Some(elem) = elements.next(ui) {
				let (col, row) = (elem.col, elem.row);

				// Color effect for fun.
				let (r, g, b, a) = (
					0.5 + (elem.col as f32 / cols as f32) / 2.0,
					0.75,
					1.0 - (elem.row as f32 / rows as f32) / 2.0,
					1.0
				);

				// We can use `Element`s to instantiate any kind of widget we like.
				// The `Element` does all of the positioning and sizing work for us.
				// Here, we use the `Element` to `set` a `Toggle` widget for us.
				let toggle = widget::Toggle::new(app_state.board.get(col, row))
					.rgba(r, g, b, a)
					.border(1.0);
				if let Some(new_value) = elem.set(toggle, ui).last() {
					app_state.board.set(col, row, new_value);
				}
			}

			
		}

		// Render the `Ui` and then display it on the screen.
		if let Some(primitives) = ui.draw_if_changed() {
			renderer.fill(&display, primitives, &image_map);
			let mut target = display.draw();
			target.clear_color(0.0, 0.0, 0.0, 1.0);
			renderer.draw(&display, &mut target, &image_map).unwrap();
			target.finish().unwrap();
		}
	}
}
