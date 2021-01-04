use nalgebra::{Vector3, Rotation3, Rotation};
use pancurses::{initscr, endwin, noecho, Input};

fn main() {
    let main_window = initscr();

    // Window constants used later for printing text    
    let lines = main_window.get_max_y();
    let cols = main_window.get_max_x();

    // Display our initial layout
    main_window.printw("Kerbel File Control");
    main_window.mvaddstr(lines - 1, 0, "Press 'q' to quit");
    main_window.refresh();
    noecho();

    // Connect to KSP via krpc-rs

    // Main loop for handling information from ksp
    loop {
        if let Some(Input::Character('q')) = main_window.getch() {
            break;
        }
    }

    // Cleanup after ourselves
    endwin();
}