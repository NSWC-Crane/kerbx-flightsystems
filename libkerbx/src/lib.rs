/*
* =================================================================================================
*
*                                      PUBLIC DOMAIN NOTICE
*                           Naval Surface Warfare Center - Crane Division
*
*  This software is a "United States Government Work" under the terms of the United States
*  Copyright Act. It was written as part of the author's official duties as a United States
*  Government employee and thus cannot be copyrighted. This software/database is freely available
*  to the public for use. Naval Surface Warfare Center - Crane Division (NSWC-CD) and the U.S.
*  Government have not places any restriction on its use or reproduction.
*
*  Although all reasonable efforts have been taken to ensure the accuracy and reliability of the
*  software and data, NSWC-CD and the U.S. Government do not and cannot warrant the performance or
*  results that may be obtained by using this software or data. NSWC-CD and the U.S. Government
*  disclaim all warranties, express or implied, including warranties of performance,
*  merchantability or fitness for any particular purpose.
*
*  Please cite the author in any work or product based on this material.
*
* =================================================================================================
*/

// KRPC Mars Generated Services
pub mod drawing;
pub mod infernal_robotics;
pub mod kerbal_alarm_clock;
pub mod remote_tech;
pub mod space_center;
pub mod ui;

// Proto generated library
pub mod kerbx;

// Bespoke code
pub mod flightplan;

// Library Modules
use crate::kerbx::Time;
use contracts::*;
use krpc_mars::{error::Error, RPCClient};
use nalgebra::Vector3;
use space_center::{CelestialBody, ReferenceFrame, Vessel};
use std::time::SystemTime;

pub fn time() -> Result<Time, std::time::SystemTimeError> {
    let mut time = Time::new();
    time.seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    Ok(time)
}

/// Abstraction of our KerbX Vessel within the KSP Simulator
pub struct KerbxTransport {
    sim_feed: RPCClient,
    vessel_obj: Vessel,
    vessel_ref_frame: ReferenceFrame,
    surf_ref_frame: ReferenceFrame,
    /// The ID of the planet the transport is orbiting
    orbiting_obj: CelestialBody,
    orb_ref_frame: ReferenceFrame,
    north: Vector3<f64>,
    up: Vector3<f64>,
}

impl KerbxTransport {
    pub fn new(connection: RPCClient) -> Result<KerbxTransport, Error> {
        let vessel = connection.mk_call(&space_center::get_active_vessel())?;

        // Given travel is terrestrial only on Kerbin, we are assuming these reference frames are
        // constant across the life of the vehicle
        let surf_ref_frame = connection.mk_call(&vessel.get_surface_reference_frame())?;
        let vessel_ref_frame = connection.mk_call(&vessel.get_reference_frame())?;
        let orb_ref_frame = connection.mk_call(&vessel.get_orbital_reference_frame())?;

        let orbit = connection.mk_call(&vessel.get_orbit())?;
        let planet = connection.mk_call(&orbit.get_body())?;

        Ok(KerbxTransport {
            sim_feed: connection,
            vessel_obj: vessel,
            vessel_ref_frame,
            surf_ref_frame,
            orbiting_obj: planet,
            orb_ref_frame,
            north: Vector3::new(0.0, 1.0, 0.0),
            up: Vector3::new(1.0, 0.0, 0.0),
        })
    }
    pub fn get_direction(&self) -> Result<Vector3<f64>, Error> {
        // Get current vessel direction
        let direction = self
            .sim_feed
            .mk_call(&self.vessel_obj.direction(&self.surf_ref_frame))?;

        Ok(Vector3::new(direction.0, direction.1, direction.2))
    }

    pub fn get_horizon(&self) -> Result<Vector3<f64>, Error> {
        let direction = self.get_direction()?;
        Ok(Vector3::new(0.0, direction[1], direction[2]))
    }

    /// m/s in reference to the nearest orbiting object
    /// NOTE: Currently bugged and will always return 0 for some reason.
    pub fn get_velocity(&self) -> Result<f64, Error> {
        // velocity rpc call is returning 0
        let flight = self
            .sim_feed
            .mk_call(&self.vessel_obj.flight(&self.surf_ref_frame))?;

        // True Air Speed using the surface reference frame provides velocity in m/s based on
        // the surface
        Ok(self.sim_feed.mk_call(&flight.get_true_air_speed())?.into())

        // Returning true air speed as the following rpc calls fail to return data:
        // flight.get_speed(), flight.get_velocity, vessel.get_velocity
        // If you can ever get velocity to return a three-tuple, you can use the following to calculate
        // the linear velocity:
        //Ok((velocity.0.powi(2) + velocity.1.powi(2) + velocity.2.powi(2)).sqrt())
    }

    #[ensures(ret.is_ok() ->  (*ret.as_ref().unwrap() > -180.0 && *ret.as_ref().unwrap() <= 180.0), "Roll must be -180 < x <= +180 degrees." )]
    pub fn get_roll(&self) -> Result<f64, Error> {
        let vessel_up = self.sim_feed.mk_call(&space_center::transform_direction(
            (0.0, 0.0, -1.0),
            &self.vessel_ref_frame,
            &self.surf_ref_frame,
        ))?;
        let vessel_up = Vector3::new(vessel_up.0, vessel_up.1, vessel_up.2);

        let plane_normal = self.get_direction()?.cross(&self.up);
        let roll = self.to_degrees(vessel_up.angle(&plane_normal));
        let roll = if vessel_up[0] > 0.0 {
            roll * -1.0
        } else {
            if roll < 0.0 {
                roll + 180.0
            } else {
                roll - 180.0
            }
        };
        Ok(roll)
    }
    pub fn get_pitch(&self) -> Result<f64, Error> {
        // Calculate pitch
        let direction = self.get_direction()?;
        let pitch = if direction[0] < 0.0 {
            -self.to_degrees(self.get_horizon()?.angle(&direction))
        } else {
            self.to_degrees(self.get_horizon()?.angle(&direction))
        };
        Ok(pitch)
    }
    pub fn get_heading(&self) -> Result<f64, Error> {
        let horizon = self.get_horizon()?;
        let heading = if horizon[2] < 0.0 {
            360.0 - self.to_degrees(horizon.angle(&self.north))
        } else {
            self.to_degrees(horizon.angle(&self.north))
        };
        Ok(heading)
    }
    pub fn get_lat(&self) -> Result<f64, Error> {
        let position = self
            .sim_feed
            .mk_call(&self.vessel_obj.position(&self.orb_ref_frame))?;
        let lat = self.sim_feed.mk_call(
            &self
                .orbiting_obj
                .latitude_at_position(position, &self.orb_ref_frame),
        )?;
        Ok(lat)
    }
    pub fn get_lon(&self) -> Result<f64, Error> {
        let position = self
            .sim_feed
            .mk_call(&self.vessel_obj.position(&self.orb_ref_frame))?;
        let lon = self.sim_feed.mk_call(
            &self
                .orbiting_obj
                .longitude_at_position(position, &self.orb_ref_frame),
        )?;
        Ok(lon)
    }
    pub fn get_alt(&self) -> Result<f64, Error> {
        let position = self
            .sim_feed
            .mk_call(&self.vessel_obj.position(&self.orb_ref_frame))?;
        let alt = self.sim_feed.mk_call(
            &self
                .orbiting_obj
                .altitude_at_position(position, &self.orb_ref_frame),
        )?;
        Ok(alt)
    }

    // TODO: Error passing
    pub fn trigger_stage(&self) -> Result<(), Error> {
        let control = self.sim_feed.mk_call(&self.vessel_obj.get_control())?;
        let result = self.sim_feed.mk_call(&control.activate_next_stage())?;
        Ok(())
    }

    /// Returns the current stage of the craft as reflected by the in-game UI
    pub fn get_stage(&self) -> Result<i32, Error> {
        let control = self.sim_feed.mk_call(&self.vessel_obj.get_control())?;
        let stage = self.sim_feed.mk_call(&control.get_current_stage())?;
        Ok(stage)
    }

    /// percent must be a value between 0 and 1.
    #[requires(percent >= 0.0 && percent <= 1.0, "Throttle percent only valid between 0 and 1.")]
    pub fn set_throttle(&self, percent: f32) -> Result<(), Error> {
        let control = self.sim_feed.mk_call(&self.vessel_obj.get_control())?;
        self.sim_feed.mk_call(&control.set_throttle(percent))?;
        Ok(())
    }

    /// Turn on/off autopilot.
    /// setting: if true, turns on autopilot; if false, turns off autopilot
    pub fn set_auto_pilot(&self, setting: bool) -> Result<(), Error> {
        let autopilot = self.sim_feed.mk_call(&self.vessel_obj.get_auto_pilot())?;
        if setting {
            self.sim_feed.mk_call(&autopilot.engage())?;
        } else {
            self.sim_feed.mk_call(&autopilot.disengage())?;
        }
        Ok(())
    }

    /// Sets the direction the auto pilot should attempt to fly in -- autopilot automatically
    /// stabilizes flight.
    /// pitch: Wanted pitch
    /// heading: Wanted heading
    pub fn set_auto_pilot_direction(&self, pitch: f32, heading: f32) -> Result<(), Error> {
        let autopilot = self.sim_feed.mk_call(&self.vessel_obj.get_auto_pilot())?;
        self.sim_feed
            .mk_call(&autopilot.target_pitch_and_heading(pitch, heading))?;
        Ok(())
    }

    /// Turn on/off sas on craft.
    /// setting: if true, turns on sas; if false, turns off sas
    pub fn set_sas(&self, setting: bool) -> Result<(), Error> {
        let control = self.sim_feed.mk_call(&self.vessel_obj.get_control())?;
        self.sim_feed.mk_call(&control.set_sas(setting))?;
        Ok(())
    }

    /// Turn on/off rcs on craft.
    /// /// setting: if true, turns on rcs; if false, turns off rcs
    pub fn set_rcs(&self, setting: bool) -> Result<(), Error> {
        let control = self.sim_feed.mk_call(&self.vessel_obj.get_control())?;
        self.sim_feed.mk_call(&control.set_rcs(setting))?;
        Ok(())
    }

    fn to_degrees(&self, val: f64) -> f64 {
        val * (180.0 / std::f64::consts::PI)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
