// Cool setup:
// Adds 5+6 to survive rules, starting with a filled 32x32 grid.


#[macro_use]
extern crate conrod;
use conrod::{widget, position, Borderable, Colorable, Labelable, Positionable, Sizeable, Widget};
use conrod::backend::glium::glium;
use conrod::backend::glium::glium::{DisplayBuild, Surface};
use std::num::Wrapping;

// Conway rules (see https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life for fun board patterns):
// - Cell survives if it has 2 or 3 neighbor cells alive
// - Cell is born if it has 3 neighbor cells alive
// - Otherwise cell dies

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
	survive_rules: Vec<usize>,
	born_rules: Vec<usize>,
}

impl Board {
	fn new( width: usize, height: usize, survives: &Vec<usize>, borns: &Vec<usize> ) -> Board {
		Board {
			width: width, 
			height: height,
			data: vec![true; width* height],
			survives_by_count: Board::compile_survive_rules(&survives),
			borns_by_count: Board::compile_born_rules(&borns),
			survive_rules: survives.clone(),
			born_rules: borns.clone(),
		}
	}
	
	fn compile_survive_rules( survives: &Vec<usize> ) -> [bool; 10] {
		let mut survives_by_count = [false; 10];
		for count in survives.iter() {
			survives_by_count[count+1] = true; // center cell is included in the count, hence the +1
		}
		survives_by_count
	}
	
	fn compile_born_rules( borns: &Vec<usize> ) -> [bool; 9] {
		let mut borns_by_count = [false; 9];
		for count in borns.iter() {
			borns_by_count[count+0] = true;
		}
		borns_by_count
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

	fn update_survive_rules( &mut self, new_survives: &mut Vec<usize> ) {
		self.survive_rules = new_survives.clone();
		self.survives_by_count = Board::compile_survive_rules( new_survives );
	}

	fn update_born_rules( &mut self, new_borns: &mut Vec<usize> ) {
		self.born_rules = new_borns.clone();
		self.borns_by_count = Board::compile_born_rules( new_borns );
	}
	
	fn fill_board(&mut self, value: bool) {
		self.data = vec![value; self.data.len()];
	}
}


#[derive(Debug)]
struct AppState {
	board: Board,
	simulating: bool,
	simulation_step_duration: std::time::Duration,
	last_simulation_update:  std::time::Instant,
}

impl AppState {
	fn new( simulation_step_duration: std::time::Duration ) -> AppState {
		const BOARD_WIDTH: usize = 32;
		const BOARD_HEIGHT: usize = 32;
		
		AppState {
			board: Board::new(BOARD_WIDTH, BOARD_HEIGHT, &conway_survives(), &conway_borns()),
			simulating: false,
			simulation_step_duration: simulation_step_duration,
			last_simulation_update: std::time::Instant::now()
		}
	}
	
	fn advance_simulation(&mut self) -> bool {
		let now = std::time::Instant::now();
		let duration_since_last_update = now.duration_since(self.last_simulation_update);
		if duration_since_last_update >= self.simulation_step_duration {
			self.last_simulation_update = now;
			self.board.advance_simulation();
			true
		} else {
			false
		}
	}
}


// Generate the widget identifiers.
widget_ids!(struct Ids { 
	title,
	board,
	start_stop_button,
	fill_button,
	clear_button,
	survive_label,
	survive_label2,
	survive_rules,
	born_label,
	born_label2,
	born_rules,
});


// UI sizes
const RULE_SIZE: u32 = 16;
const NB_RULE_CELLS: u32 = 9;
const RULE_BORDER: u32 = 1;
const BOARD_CELL_SIZE: u32 = 10;


fn main() {
	let simulation_step_duration_ms: std::time::Duration = std::time::Duration::from_millis(160);
	
	let mut app_state = AppState::new(simulation_step_duration_ms);
//	println!("Initial app_state={:?}", app_state);

	// Build the window.
	let ui_width = std::cmp::max(400, BOARD_CELL_SIZE * app_state.board.width as u32);
	let ui_height = 30*2 + (RULE_SIZE + 4) * 2 + BOARD_CELL_SIZE * app_state.board.height as u32;
	let display = glium::glutin::WindowBuilder::new()
		.with_vsync()
		.with_dimensions(ui_width, ui_height)
		.with_title("Game of Life")
		.with_multisampling(4)
		.build_glium()
		.unwrap();

	// construct our `Ui`.
	let mut ui = conrod::UiBuilder::new([ui_width as f64, ui_height as f64]).build();

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
			app_state.advance_simulation();
			ui_needs_update = true;
		}

		// Instantiate all widgets in the GUI.
		{
			let ui = &mut ui.set_widgets();
			
			widget::Text::new("Game of Life") // Somehow button disappears if title is removed!
				.top_left_of(ui.window)
				.color(conrod::color::WHITE)
				.font_size(16)
				.set(ids.title, ui);

			let start_stop_label = if app_state.simulating { "Pause simulation" } else { "Start simulation"};
			let start_stop_button = widget::Button::new()
				.label(start_stop_label)
				.down_from(ids.title, 8.0)
				.label_font_size(12) // Seems to be ignored and use title font size instead?!?
				.set(ids.start_stop_button, ui);
			for _press in start_stop_button {
				app_state.simulating = !app_state.simulating;
			}
			
			for _press in widget::Button::new()
				.label("Fill board")
				.right_from(ids.start_stop_button, 4.0)
				.label_font_size(12)
				.set(ids.fill_button, ui) {
				app_state.board.fill_board(true);
			}
			
			for _press in widget::Button::new()
				.label("Clear board")
				.right_from(ids.fill_button, 4.0)
				.label_font_size(12)
				.set(ids.clear_button, ui) {
				app_state.board.fill_board(false);
			}
			
			make_rules_ui( &mut app_state, &ids, ui );

			make_board_ui( &mut app_state, &ids, ui );
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

fn make_rules_ui<'a, 'b, 'c>( app_state: &'a mut AppState, ids: &Ids, ui: &'b mut conrod::UiCell<'c>) {
	make_survive_rules_ui( app_state, ids, ui );
	make_born_rules_ui( app_state, ids, ui );
}

fn make_survive_rules_ui<'a, 'b, 'c>( app_state: &'a mut AppState, ids: &Ids, ui: &'b mut conrod::UiCell<'c>) {
	widget::Text::new("Cells with ")
				.down_from(ids.start_stop_button, 8.0)
				.x_align_to(ui.window, position::Align::Start)
				.color(conrod::color::WHITE)
				.font_size(RULE_SIZE)
				.set(ids.survive_label, ui);
	let mut elements = widget::Matrix::new(NB_RULE_CELLS as usize, 1)
		.right(4.0)
		.w_h(NB_RULE_CELLS as f64 * RULE_SIZE as f64, RULE_SIZE as f64)
		.set(ids.survive_rules, ui);
	while let Some(elem) = elements.next(ui) {
		let (col, _row) = (elem.col, elem.row);
		let enabled = app_state.board.survive_rules.contains(&col);
		let label = col.to_string();
		let toggle = widget::Toggle::new(enabled)
			.rgba(0.3215686274509804, 0.7098039215686275, 0.5607843137254902, 1.0)
			.border(RULE_BORDER as f64)
			.label(&label)
			.label_rgba(1.0,1.0,1.0,1.0)
			.label_font_size(RULE_SIZE - 2*(RULE_BORDER+1));
		if let Some(new_value) = elem.set(toggle, ui).last() {
			let mut new_survive_rules = updated_rules( &app_state.board.survive_rules, col, new_value );
			app_state.board.update_survive_rules( &mut new_survive_rules );
		}
	}
	widget::Text::new("neighbor(s) survive")
				.right(4.0)
				.align_top_of(ids.survive_label)
				.color(conrod::color::WHITE)
				.font_size(RULE_SIZE)
				.set(ids.survive_label2, ui);
}

fn make_born_rules_ui<'a, 'b, 'c>( app_state: &'a mut AppState, ids: &Ids, ui: &'b mut conrod::UiCell<'c>) {
	widget::Text::new("Cells with ")
				.down_from(ids.survive_label, 4.0)
				.color(conrod::color::WHITE)
				.font_size(RULE_SIZE)
				.set(ids.born_label, ui);
	let mut elements = widget::Matrix::new(NB_RULE_CELLS as usize, 1)
		.right(4.0)
		.w_h(NB_RULE_CELLS as f64 * RULE_SIZE as f64, RULE_SIZE as f64)
		.set(ids.born_rules, ui);
	while let Some(elem) = elements.next(ui) {
		let (col, _row) = (elem.col, elem.row);
		let enabled = app_state.board.born_rules.contains(&col);
		let label = col.to_string();
		let toggle = widget::Toggle::new(enabled)
			.rgba(0.4745098039215686, 0.23529411764705882, 0.07450980392156863, 1.0)
			.border(RULE_BORDER as f64)
			.label(&label)
			.label_rgba(1.0,1.0,1.0,1.0)
			.label_font_size(RULE_SIZE - 2*(RULE_BORDER+1));
		if let Some(new_value) = elem.set(toggle, ui).last() {
			let mut new_born_rules = updated_rules( &app_state.board.born_rules, col, new_value );
			app_state.board.update_born_rules( &mut new_born_rules );
		}
	}
	widget::Text::new("neighbor(s) are born")
				.right(4.0)
				.align_top_of(ids.born_label)
				.color(conrod::color::WHITE)
				.font_size(RULE_SIZE)
				.set(ids.born_label2, ui);
}

fn updated_rules(rules: &Vec<usize>, neighbor_count: usize, alive: bool) -> Vec<usize> {
	let mut new_rules = rules.clone();
	if !alive {
		new_rules.retain( |count| *count != neighbor_count );
	} else {
		if !new_rules.contains( &neighbor_count ) {
			new_rules.push( neighbor_count );
		}
	}
	new_rules
}



fn make_board_ui<'a, 'b, 'c>( app_state: &'a mut AppState, ids: &Ids, ui: &'b mut conrod::UiCell<'c>) -> &'b mut conrod::UiCell<'c> /*(&'a mut AppState, &'b mut conrod::UiCell<'c>)*/ {
	// Each cell of the board is a Toggle widget. Layout is done using a Matrix widget.
	let (cols, rows) = (app_state.board.width, app_state.board.height);
	let mut elements = widget::Matrix::new(cols, rows)
		.down_from(ids.born_label, 8.0)
		.w_h(BOARD_CELL_SIZE as f64 * cols as f64, BOARD_CELL_SIZE as f64 * rows as f64)
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
	
	ui
}