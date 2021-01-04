//We need to implement this for rust: 
//"conn = krpc.connect(name='Pitch/Heading/Roll', address='172.26.144.1')"
//"vessel = conn.space_center.active_vessel"

use std::f64::consts::PI as PI;


#[derive(Debug)]
struct Vector {
    x: f64,
    y: f64,
    z: f64,
}

impl Vector {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Vector {
            x: x,
            y: y,
            z: z,
        }
    }
 
    fn dot_product(&self, other: &Vector) -> f64 {
        (self.x * other.x) + (self.y * other.y) + (self.z * other.z)
    }
 
    fn cross_product(&self, other: &Vector) -> Vector {
        Vector::new(self.y * other.z - self.z * other.y,
                    self.z * other.x - self.x * other.z,
                    self.x * other.y - self.y * other.x)
    }
    
    fn magnitude(&self) -> f64 {
    	(self.dot_product(&self)).sqrt()
    }
    
    fn angle_between(&self, other: &Vector) -> f64 {
        let dp = self.dot_product(&other);
        if dp == 0.0 {
        	return 0.0;
        }
        let um = self.magnitude();
        let vm = other.magnitude();
       return ((dp / (um*vm)).acos()) * (180.0/PI);
    }
    	
} 

extern crate ncurses;


fn main() {
    println!("Hello, world!");
    let a = Vector::new(3.0, 4.0, 5.0);
    let b = Vector::new(4.0, 3.0, 5.0);
 
    println!("a . b = {}", a.dot_product(&b));
    println!("a x b = {:?}", a.cross_product(&b));
    println!("|a| = {}", a.magnitude());
    println!("angle between a and b = {}", a.angle_between(&b));
    
    let mut stdscr = ncurses::initscr();
    ncurses::nocbreak();
    ncurses::noecho();
    ncurses::curs_set(0);

    if ncurses::has_colors() {
    	ncurses::start_color();
    }
    ncurses::init_pair(1, ncurses::COLOR_RED, ncurses::COLOR_BLACK);
ncurses::init_pair(2, ncurses::COLOR_GREEN, ncurses::COLOR_BLACK);
ncurses::init_pair(3, ncurses::COLOR_BLUE, ncurses::COLOR_BLACK);

stdscr.addstr(&format!("{:?}", "Kerbal Fire Control", ncurses::A_REVERSE);
//ncurses::chgat(-1, ncurses::A_REVERSE);

/*

stdscr.addstr(curses.LINES-1, 0, "Press 'Q' to quit")
stdscr.chgat(curses.LINES-1,1,1, curses.A_BOLD | curses.color_pair(1))

output_window = curses.newwin(curses.LINES-2, curses.COLS, 1,0)
output_text_window = output_window.subwin(curses.LINES-6,curses.COLS-4, 3, 2)
output_window.box()

# Non-blocking delay for getch()
output_window.timeout(0)

stdscr.noutrefresh()
output_window.noutrefresh()

curses.doupdate()
*/

	loop {
		/*
		    vessel_direction = vessel.direction(vessel.surface_reference_frame)

                    # Get the direction of the vessel in the horizon plane
    		    horizon_direction = (0, vessel_direction[1], vessel_direction[2])
    		*/
    		let pitch = vessel_direction.angle_between(&horizon_direction);
    		if vessel_direction[0] < 0 {
    			pitch = -pitch;
    		}
    		let north = (0,1,0);
    		let heading = north.angle_between(&horizon_direction);
    		if horizon_direction[2] < 0 {
    			heading = 360 - heading;
    		}
    		let up = (1,0,0);
    		let plane_normal = vessel_direction.cross_product(&up);
    		//vessel_up = conn.space_center.transform_direction((0, 0, -1), vessel.reference_frame, vessel.surface_reference_frame)
    		let roll = vessel_up.angle_between(&plane_normal);
    		if vessel_up[0] > 0 {
    			roll = -roll;
    		}
    		else if roll < 0 {
    			roll = roll + 180;
    		}
    		else {
    			roll = roll - 180;
    		}
    		/*
    		    pos = vessel.position(vessel.orbit.body.reference_frame)
    lat = vessel.orbit.body.latitude_at_position(pos, vessel.orbit.body.reference_frame)
    lon = vessel.orbit.body.longitude_at_position(pos, vessel.orbit.body.reference_frame)
    alt = vessel.orbit.body.altitude_at_position(pos, vessel.orbit.body.reference_frame)
    
        # Write our 6dof info
    sixdof_str = """ 
        acceleration: %5.1f\n
        pitch: %5.1f\n
        roll: %5.1f\n
        heading: %5.1f\n
        lat: %5.1f\n
        lon: %5.1f\n
        alt: %5.1f\n
        """ % (vessel_accel, pitch, roll, heading, lat, lon, alt)

    output_text_window.erase()
    output_text_window.addstr(sixdof_str, curses.color_pair(3))
    output_text_window.refresh()

    # Refresh windows from the bottom up
    stdscr.noutrefresh()
    output_window.noutrefresh()
    output_text_window.noutrefresh()
    curses.doupdate()

    # Handle loop control
    c = output_window.getch()
    if c == ord('q') or c == ord('Q'):
        break

    time.sleep(1/500.)   
#END While

curses.nocbreak()
curses.echo()
curses.curs_set(1)
curses.endwin()
    		*/
    		
   } 		
}
