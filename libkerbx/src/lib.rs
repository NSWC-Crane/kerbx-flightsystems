// KRPC Mars Generated Services
pub mod space_center;
pub mod drawing;
pub mod infernal_robotics;
pub mod kerbal_alarm_clock;
pub mod remote_tech;
pub mod ui;

// Library Modules
use nalgebra::{Vector3};
use krpc_mars::{RPCClient};
use std::{error::Error, thread, time};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

/*

    // Connect to KSP via krpc-rs
    let server_address = format!("{}:{}", matches.value_of("ip").unwrap(), matches.value_of("port").unwrap());
    let client = RPCClient::connect("Name of Application", server_address).expect("Could not connect to KRPC Server.");

    // Main loop for handling information from ksp
    loop {

        let vessel = client.mk_call(&space_center::get_active_vessel())?;

        let surf_ref_frame = client.mk_call(&vessel.get_surface_reference_frame())?;
        let vessel_ref_frame = client.mk_call(&vessel.get_reference_frame())?;

        // Get values for calculating lat/lon
        let orb_ref_frame = client.mk_call(&vessel.get_orbital_reference_frame())?;
        let orbit = client.mk_call(&vessel.get_orbit())?;
        let planet = client.mk_call(&orbit.get_body())?;

        // Get current vessel direction
        let direction = client.mk_call(&vessel.direction(&surf_ref_frame))?;
        let direction = Vector3::new(direction.0, direction.1, direction.2);

        // Constants leveraged for the final calculation.
        let horizon = Vector3::new(0.0, direction[1], direction[2]);
        let north = Vector3::new(0.0, 1.0, 0.0);
        let up = Vector3::new(1.0, 0.0, 0.0);

        // This line is causing Protobuf(WireError(UnecpectedEof)) errors.
        let vessel_up = client.mk_call(&space_center::transform_direction((0.0, 0.0, -1.0), &vessel_ref_frame, &surf_ref_frame))?;

        // Calculate pitch
        let pitch = if direction[0] < 0.0 {
            -to_degrees(horizon.angle(&direction))
        } else {
            to_degrees(horizon.angle(&direction))
        };

        // Calculate heading
        let heading = if horizon[2] < 0.0 {
            360.0 - to_degrees(horizon.angle(&north))
        } else {
            to_degrees(horizon.angle(&north))
        };

        let vessel_up = Vector3::new(vessel_up.0, vessel_up.1, vessel_up.2);
        let plane_normal = direction.cross(&up);
        let roll = to_degrees(vessel_up.angle(&plane_normal));
        let roll = if vessel_up[0] > 0.0 {
            roll * -1.0
        } else {
            if roll < 0.0 {
                roll + 180.0
            } else {
                roll - 180.0
            }
        };


        // Get current vessel position
        let position = client.mk_call(&vessel.position(&orb_ref_frame))?;
        let lat = client.mk_call(&planet.latitude_at_position(position, &orb_ref_frame))?;
        let lon = client.mk_call(&planet.longitude_at_position(position, &orb_ref_frame))?;
        let alt = client.mk_call(&planet.altitude_at_position(position, &orb_ref_frame))?;


        // Quit on recieving q
        let b = stdin.next();
        if let Some(Ok(b'q')) = b {
            break;
        }

        mvaddstr(&mut stdout, 3, 3, format!("Yaw: {}", heading).as_str());
        mvaddstr(&mut stdout, 3, 4, format!("Pitch: {}", pitch).as_str());
        mvaddstr(&mut stdout, 3, 5, format!("Roll: {}", roll).as_str());
        mvaddstr(&mut stdout, 3, 6, format!("Lat: {}", lat).as_str());
        mvaddstr(&mut stdout, 3, 7, format!("Lon: {}", lon).as_str());
        mvaddstr(&mut stdout, 3, 8, format!("Alt: {}", alt).as_str());

        thread::sleep(time::Duration::from_millis(100));
    }

    // Cleanup after ourselves
    write!(stdout, "{}", clear::All).unwrap();

    Ok(())
}

fn to_degrees(val: f64) -> f64 {
    val * (180.0 / std::f64::consts::PI)
}
 */