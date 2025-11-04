// use crate::smart_devices::Device;
// use crate::smart_room::{AccessDevice, SmartRoom};
// use crate::traits::Information;
// use crate::smart_home::SmartHome;
// use std::fmt::Display;
//
// pub struct Report<'a> {
//     rooms: Vec<&'a SmartRoom>,
//     devices: Vec<&'a Device>,
// }
//
// impl Report {
//     pub fn new() -> Self {
//         Report {
//             rooms: Vec::new(),
//             devices: Vec::new(),
//         }
//     }
//     pub fn add_room<'a>(mut self, room: &'a SmartRoom) -> Self {
//         self.rooms.push(room);
//         self
//
//     }
//
//     pub fn add_device<'a>(mut self, device: &'a Device) -> Self{
//         self.devices.push(device);
//         self
//     }
// }
