/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state

enum Cell {
    Empty,          // Opened, no mine
    Number(u8),     // Opened, close to mine
    Flag,           // Marked as mine
    Mine,           // Found mine, game over
    Closed,         // Not opened yet
}
impl Cell {
    fn is_clickable(&self) -> bool {
        match *self {
            Self::Flag | Self::Mine | Self::Empty => false,
            _ => true,
        }
    }
    fn is_number(&self) -> bool {
        match *self {
            Self::Number(_) => true,
            _ => false,
        }
    }
    fn is_secondary_clickable(&self) -> bool {
        match *self {
            Self::Flag | Self::Closed => true,
            _ => false,
        }
    }
}

pub struct TemplateApp {
    // Example stuff:
    board_dimensions: (usize, usize, usize, usize),
    mines_num: u16,

    mines_positions: Vec<(usize, usize, usize, usize)>,
    back_board: Vec<Vec<Vec<Vec<u8>>>>,
    board: Vec<Vec<Vec<Vec<Cell>>>>,
    current_hover: Option<(usize, usize, usize, usize)>,
    alive: bool,
    current_mines: u16,
    total_mines: u16,
    spreading: bool,

    inverted_mines_placing: bool
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            board_dimensions: (4, 4, 4, 4),
            mines_num: 10,

            mines_positions: vec![],
            back_board: vec![],
            board: vec![],
            current_hover:  None,
            alive: true,
            current_mines: 0,
            total_mines: 10,
            spreading: true,
            inverted_mines_placing: false,
        }
    }
}

// PASTE START

fn init_game(board: &mut Vec<Vec<Vec<Vec<Cell>>>>, back_board: &mut Vec<Vec<Vec<Vec<u8>>>>, dimensions: &(usize, usize, usize, usize), mines: &mut Vec<(usize, usize, usize, usize)>, alive: &mut bool, mines_num: &u16, current_mines: &mut u16, total_mines: &mut u16) {
    *alive = true;
    *current_mines = *mines_num;
    *total_mines = 0;

    use rand::Rng;

    // Initialization of an empty board
    let mut b = vec![];
    let mut bb = vec![]; // init backboard
    for _ in 0..dimensions.0 {
        let mut blocal = vec![];
        let mut local = vec![];
        for _ in 0..dimensions.1 {
            let mut blocall = vec![];
            let mut locall = vec![];
            for _ in 0..dimensions.2 {
                let mut localll = vec![];
                let mut blocalll = vec![];
                for _ in 0..dimensions.3 {
                    localll.push(Cell::Closed);
                    blocalll.push(0);
                }
                locall.push(localll);
                blocall.push(blocalll);
            }
            local.push(locall);
            blocal.push(blocall);
        }
        b.push(local);
        bb.push(blocal);
    }
    *board = b;
    *back_board = bb;

    // Initialization of the mines mines_positions
    let mut rng = rand::thread_rng();
    *mines = vec![];
    for _ in 0..*mines_num {
        let mut pos: (usize, usize, usize, usize) = (rng.gen_range(0..dimensions.0), rng.gen_range(0..dimensions.1), rng.gen_range(0..dimensions.2), rng.gen_range(0..dimensions.3));
        while mines.contains(&pos) { 
            pos = (rng.gen_range(0..dimensions.0), rng.gen_range(0..dimensions.1), rng.gen_range(0..dimensions.2), rng.gen_range(0..dimensions.3));
        } 
        mines.push(pos); 
    }
    
    // Calculating back-board numbers
    for (x, first) in back_board.iter_mut().enumerate() {
        let x: isize = x.try_into().unwrap();
        for (y, second) in first.iter_mut().enumerate() {
            let y: isize = y.try_into().unwrap();
            for (z, third) in second.iter_mut().enumerate() {
                let z: isize = z.try_into().unwrap();
                for (w, cell) in third.iter_mut().enumerate() {
                    let w: isize = w.try_into().unwrap();
                    let mut total: u8 = 0;
                    for dx in [-1, 0, 1] {
                        if x + dx < 0 || x + dx == dimensions.0.try_into().unwrap() { continue; }
                        for dy in [-1, 0, 1] {
                            if y + dy < 0 || y + dy == dimensions.1.try_into().unwrap() { continue; }
                            for dz in [-1, 0, 1] {
                                if z + dz < 0 || z + dz == dimensions.2.try_into().unwrap() { continue; }
                                for dw in [-1, 0, 1] {
                                    if w + dw < 0 || w + dw == dimensions.3.try_into().unwrap() { continue; }
                                    if dx == dy && dw == dz && dx == dz && dz == 0 { continue; }
                                    
                                    // These should never fail
                                    let newx: usize = (x + dx).try_into().unwrap();
                                    let newy: usize = (y + dy).try_into().unwrap();
                                    let newz: usize = (z + dz).try_into().unwrap();
                                    let neww: usize = (w + dw).try_into().unwrap();
                                    if mines.contains(&(newx, newy, newz, neww)) { total += 1; }
                                }
                            }
                        }
                    }
                    *cell = total;
                }
            }
        }
    }
}

fn mark_flags(board: &mut Vec<Vec<Vec<Vec<Cell>>>>, position: &Option<(usize, usize, usize, usize)>, flag_count: &mut u16) {
    if let Some((x, y, z, w)) = *position {
        let is_flagged = if let Cell::Flag = board[x][y][z][w] { true } else { false };
        if is_flagged { *flag_count -= 1; } else { *flag_count += 1; }
        board[x][y][z][w] = if is_flagged { Cell::Closed } else { Cell::Flag };
    }
}

fn spread_cells(board: &mut Vec<Vec<Vec<Vec<Cell>>>>, back_board: &Vec<Vec<Vec<Vec<u8>>>>, mines: &Vec<(usize, usize, usize, usize)>, position: &Option<(usize, usize, usize, usize)>, alive: &mut bool) {
    if let Some((x, y, z, w)) = *position {
        // println!("IN {:?} {:?}", board[x][y][z][w], (x, y, z, w));
        if let Cell::Empty = board[x][y][z][w] {

            let x: isize = x.try_into().unwrap();
            let y: isize = y.try_into().unwrap();
            let z: isize = z.try_into().unwrap();
            let w: isize = w.try_into().unwrap();

            let dimensions = (board.len(), board[0].len(), board[0][0].len(), board[0][0][0].len());
            for dx in -1..=1 {
                let newx = x + dx;
                if newx < 0 || newx == dimensions.0.try_into().unwrap() { continue; }
                let newx: usize = newx.try_into().unwrap();
                for dy in -1..=1 {
                    let newy = y + dy;
                    if newy < 0 || newy == dimensions.1.try_into().unwrap() { continue; }
                    let newy: usize = newy.try_into().unwrap();
                    for dz in -1..=1 {
                        let newz = z + dz;
                        if newz < 0 || newz == dimensions.2.try_into().unwrap() { continue; }
                        let newz: usize = newz.try_into().unwrap();
                        for dw in -1..=1 {
                            let neww = w + dw;
                            if neww < 0 || neww == dimensions.3.try_into().unwrap() { continue; }
                            let neww: usize = neww.try_into().unwrap();
                            if let Cell::Closed = board[newx][newy][newz][neww] {
                                // println!("It is indeed closed");
                                reveal_cell(board, back_board, mines, &Some((newx, newy, newz, neww)), alive, &true);
                            }
                        }
                    }
                }
            }   
        }
    }
}

fn chord_cell(board: &mut Vec<Vec<Vec<Vec<Cell>>>>, back_board: &Vec<Vec<Vec<Vec<u8>>>>, mines: &Vec<(usize, usize, usize, usize)>, position: &Option<(usize, usize, usize, usize)>, alive: &mut bool, do_spread: &bool) {
    if let Some((x, y, z, w)) = *position {
        let dimensions = (board.len(), board[0].len(), board[0][0].len(), board[0][0][0].len());
        let mut total_flags = 0;
        let current_number = if let Cell::Number(num) = board[x][y][z][w] { num } else { 0 };

        let x: isize = x.try_into().unwrap();
        let y: isize = y.try_into().unwrap();
        let z: isize = z.try_into().unwrap();
        let w: isize = w.try_into().unwrap();

        for dx in [-1, 0, 1] {
            if x + dx < 0 || x + dx == dimensions.0.try_into().unwrap() { continue; }
            for dy in [-1, 0, 1] {
                if y + dy < 0 || y + dy == dimensions.1.try_into().unwrap() { continue; }
                for dz in [-1, 0, 1] {
                    if z + dz < 0 || z + dz == dimensions.2.try_into().unwrap() { continue; }
                    for dw in [-1, 0, 1] {
                        if w + dw < 0 || w + dw == dimensions.3.try_into().unwrap() { continue; }
                        if dx == dy && dw == dz && dx == dz && dz == 0 { continue; }
                        
                        // These should never fail
                        let newx: usize = (x + dx).try_into().unwrap();
                        let newy: usize = (y + dy).try_into().unwrap();
                        let newz: usize = (z + dz).try_into().unwrap();
                        let neww: usize = (w + dw).try_into().unwrap();
                        if let Cell::Flag = board[newx][newy][newz][neww] {
                            total_flags += 1;
                        }
                    }
                }
            }
        }
        if total_flags == current_number {
            for dx in [-1, 0, 1] {
                if x + dx < 0 || x + dx == dimensions.0.try_into().unwrap() { continue; }
                for dy in [-1, 0, 1] {
                    if y + dy < 0 || y + dy == dimensions.1.try_into().unwrap() { continue; }
                    for dz in [-1, 0, 1] {
                        if z + dz < 0 || z + dz == dimensions.2.try_into().unwrap() { continue; }
                        for dw in [-1, 0, 1] {
                            if w + dw < 0 || w + dw == dimensions.3.try_into().unwrap() { continue; }
                            if dx == dy && dw == dz && dx == dz && dz == 0 { continue; }
                            
                            // These should never fail
                            let newx: usize = (x + dx).try_into().unwrap();
                            let newy: usize = (y + dy).try_into().unwrap();
                            let newz: usize = (z + dz).try_into().unwrap();
                            let neww: usize = (w + dw).try_into().unwrap();
                            
                            let is_flag = if let Cell::Flag = board[newx][newy][newz][neww] { true } else { false };
                            if !is_flag { reveal_cell(board, back_board, mines, &Some((newx, newy, newz, neww)), alive, do_spread); }
                        }
                    }
                }
            }
        }
    }
}

fn reveal_cell(board: &mut Vec<Vec<Vec<Vec<Cell>>>>, back_board: &Vec<Vec<Vec<Vec<u8>>>>, mines: &Vec<(usize, usize, usize, usize)>, position: &Option<(usize, usize, usize, usize)>, alive: &mut bool, do_spread: &bool) {
    let mut spread = false;
    if let Some((x, y, z, w)) = *position {
        if mines.contains(&(x, y, z, w)) {
            board[x][y][z][w] = Cell::Mine;
            *alive = false;
        }
        else {
            board[x][y][z][w] = if back_board[x][y][z][w] > 0 {
                Cell::Number(back_board[x][y][z][w])
            }
            else {
                spread = true;
                Cell::Empty
            };
        }
        if spread && *do_spread { spread_cells(board, back_board, mines, position, alive); }
    }
}

fn is_neighbor(pos1: &(usize, usize, usize, usize), pos2: &(usize, usize, usize, usize)) -> bool {
    let (x1, y1, z1, w1) = *pos1;
    let (x2, y2, z2, w2) = *pos2;
    let x1: isize = x1.try_into().unwrap();
    let x2: isize = x2.try_into().unwrap();
    let y1: isize = y1.try_into().unwrap();
    let y2: isize = y2.try_into().unwrap();
    let z1: isize = z1.try_into().unwrap();
    let z2: isize = z2.try_into().unwrap();
    let w1: isize = w1.try_into().unwrap();
    let w2: isize = w2.try_into().unwrap();

    if (x1 - x2 == 1 || x2 - x1 == 1 || x1 == x2) && 
       (y1 - y2 == 1 || y2 - y1 == 1 || y1 == y2) && 
       (z1 - z2 == 1 || z2 - z1 == 1 || z1 == z2) && 
       (w1 - w2 == 1 || w2 - w1 == 1 || w1 == w2) {
        true
    }
    else {
        false
    }

}

fn count_closed_cells(board: &Vec<Vec<Vec<Vec<Cell>>>>) -> u16 {
    let mut total = 0;
    for a in board.iter() {
        for b in a.iter() {
            for c in b.iter() {
                for d in c.iter() {
                    match d {
                        Cell::Flag | Cell::Closed => total += 1,
                        _ => {}
                    };
                }
            }
        }
    }
    total
}

fn run_game(ctx: &egui::Context, board: &mut Vec<Vec<Vec<Vec<Cell>>>>, back_board: &Vec<Vec<Vec<Vec<u8>>>>, mines_positions: &Vec<(usize, usize, usize, usize)>, hover_display: &mut Option<(usize, usize, usize, usize)>, alive: &mut bool, total_mines: &u16, current_mines: &mut u16, do_spread: &mut bool, inverted_clicks: &bool) {
    let mut clicked_at: Option<(usize, usize, usize, usize)> = None;
    let mut secondary_at: Option<(usize, usize, usize, usize)> = None;
    let mut chord_at: Option<(usize, usize, usize, usize)> = None;
    let mut hover_at: Option<(usize, usize, usize, usize)> = None;

    let mut win = false;
    egui::TopBottomPanel::top("top-main-panel").show(ctx, |ui| {
        println!("HERE");
        ui.horizontal(|ui| {
            let closed_cells = count_closed_cells(board);
            if *total_mines == closed_cells {
                *alive = false;
                win = true;
                ui.label(egui::RichText::new("You won!").heading());
            }
            else {
                let mines_to_display = if *current_mines >= *total_mines { 0 } else { *total_mines - *current_mines };
                ui.label(egui::RichText::new(format!("Mines: {}", mines_to_display)).heading());
                if !*alive {
                    ui.label(egui::RichText::new("You lost!").heading());
                }
            }
        });
    });
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::Grid::new("large-board").spacing(egui::vec2(10.0, 10.0)).show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            for (fidx, first) in board.iter().enumerate() {
                for (sidx, second) in first.iter().enumerate() {
                    ui.group(|ui|{
                        ui.vertical(|ui| {
                            for (tidx, third) in second.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    for (idx, elem) in third.iter().enumerate() {
                                        let elem = if *alive || !mines_positions.contains(&(fidx, sidx, tidx, idx)) { elem } else { 
                                            if let Cell::Flag = elem { elem } else { if !win { &Cell::Mine } else { &Cell::Flag } }
                                        }; 
                                        let btn = egui::Button::new(
                                            match elem {
                                                Cell::Flag => egui::RichText::new("F").color(egui::Color32::BLACK),
                                                Cell::Mine => egui::RichText::new("X").color(egui::Color32::WHITE),
                                                Cell::Number(num) => egui::RichText::new(num.to_string()),
                                                _ => egui::RichText::new(" ".to_owned()),
                                            }
                                        ).min_size(egui::vec2(20.0, 20.0));
                                        let btn = match elem {
                                            Cell::Flag => if *alive || mines_positions.contains(&(fidx, sidx, tidx, idx)) { 
                                                btn.fill(egui::Color32::WHITE) 
                                            } 
                                            else {
                                                btn.fill(egui::Color32::LIGHT_RED)
                                            },
                                            Cell::Mine => btn.fill(egui::Color32::RED),
                                            _ => btn,
                                        };
                                        
                                        let btn = if let Some(pos) = *hover_display {
                                            if is_neighbor(&pos, &(fidx, sidx, tidx, idx)) && elem.is_clickable() {
                                                btn.fill(egui::Color32::DARK_GRAY)
                                            }
                                            else {
                                                btn
                                            }
                                        } else { btn };

                                        // Adding either enabled or disabled, depending on the cell
                                        let btn = if let Cell::Empty = elem {
                                            ui.add_enabled(false, btn)
                                        }
                                        else {
                                            ui.add(btn)
                                        };
                                        
                                        // Button neighbor highlighting
                                        if btn.hovered() {
                                            hover_at = Some((fidx, sidx, tidx, idx));
                                        } 

                                        // Passing the reaction further if the button was pressed
                                        if elem.is_clickable() {
                                            if btn.clicked() {
                                                if !elem.is_number() {
                                                    clicked_at = Some((fidx, sidx, tidx, idx));
                                                }
                                                else {
                                                    chord_at = Some((fidx, sidx, tidx, idx));
                                                }
                                            }
                                        }
                                        if elem.is_secondary_clickable() {
                                            if btn.secondary_clicked() {
                                                    secondary_at = Some((fidx, sidx, tidx, idx));
                                            }
                                        }
                                    }
                                ui.end_row();
                                });
                            }
                        });
                    });
                }
                ui.end_row();
            }
        });
    });
    
    if *alive {
        if *inverted_clicks {
            (secondary_at, clicked_at) = (clicked_at, secondary_at);
        }
        mark_flags(board, &secondary_at, current_mines);
        reveal_cell(board, back_board, mines_positions, &clicked_at, alive, do_spread);
        chord_cell(board, back_board, mines_positions, &chord_at, alive, do_spread);
    }
    *hover_display = hover_at;
}

fn bottom_panel(ctx: &egui::Context, board_dimensions: &mut (usize, usize, usize, usize), mines_num: &mut u16, do_spread: &mut bool, inverted_mines: &mut bool) -> bool {
    let mut restart = false;
    egui::TopBottomPanel::bottom("settings").show(ctx, |ui| {
        ui.checkbox(inverted_mines, egui::RichText::new("Click to flag").heading());
        ui.horizontal(|ui| {
            ui.label("Dimensions: ");
            let (mut x, mut y, mut z, mut w) = *board_dimensions;
            let xdrag = egui::DragValue::new(&mut x).clamp_range(1..=128);
            let ydrag = egui::DragValue::new(&mut y).clamp_range(1..=128);
            let zdrag = egui::DragValue::new(&mut z).clamp_range(1..=128);
            let wdrag = egui::DragValue::new(&mut w).clamp_range(1..=128);
            ui.add(xdrag);
            ui.add(ydrag);
            ui.add(zdrag);
            ui.add(wdrag);
            let size = x * y * z * w;
            *board_dimensions = (x, y, z, w);
            ui.label(format!("Hypervolume: {}. ", size));
            if size >= 625 {
                ui.label("Careful, large board size might lead to crashes! Consider turning off automatic spreading.");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Mines: ");
            let drag = egui::DragValue::new(mines_num).clamp_range(1..=(board_dimensions.0 * board_dimensions.1 * board_dimensions.2 * board_dimensions.3));
            ui.add(drag);
            let numerator: f64 = (*mines_num).try_into().unwrap();
            let denominator: u32 = (board_dimensions.0 * board_dimensions.1 * board_dimensions.2 * board_dimensions.3).try_into().unwrap();
            let denominator: f64 = denominator.try_into().unwrap();
            ui.label(format!("Mine density: {:.3}", numerator/denominator));
        });
        if ui.button("Restart").clicked() {
            restart = true;
        }
        ui.checkbox(do_spread, "Automatic spreading");
    });
    restart
}

// PASTE END

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut res = Default::default();
        let TemplateApp { ref mut board_dimensions, ref mut mines_num, ref mut mines_positions, ref mut back_board, ref mut board, current_hover: _, ref mut alive, ref mut current_mines, ref mut total_mines, .. } = res;
        init_game(board, back_board, board_dimensions, mines_positions, alive, mines_num, current_mines, total_mines);
        *total_mines = 10;
        *current_mines = 0;
        res
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let TemplateApp { ref mut board_dimensions, ref mut mines_num, ref mut mines_positions, ref mut back_board, ref mut board, ref mut current_hover, ref mut alive, ref mut current_mines, ref mut total_mines, ref mut spreading, ref mut inverted_mines_placing } = self;
        let should_restart = bottom_panel(ctx, board_dimensions, mines_num, spreading, inverted_mines_placing);
        if should_restart {
            init_game(board, back_board, board_dimensions, mines_positions, alive, mines_num, total_mines, current_mines);
        }
        run_game(ctx, board, back_board, mines_positions, current_hover, alive, total_mines, current_mines, spreading, inverted_mines_placing);
    }
}
