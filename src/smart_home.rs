use crate::smart_room::SmartRoom;

#[allow(dead_code)] // suppress warning
struct SmartHome {
    name: String,
    rooms: Vec<SmartRoom>,
}

#[allow(dead_code)] // suppress warning
impl SmartHome {
    pub fn new(name: String, rooms: Vec<SmartRoom>) -> Self {
        SmartHome { name, rooms }
    }

    pub fn home_name(&self) -> String {
        self.name.clone()
    }

    pub fn view_room(&self, index: usize) -> &SmartRoom {
        &self.rooms[index]
    }

    pub fn get_room(&mut self, index: usize) -> &mut SmartRoom {
        &mut self.rooms[index]
    }

    fn list_rooms(&self) -> Vec<String> {
        self.rooms.iter().map(|r| r.room_info()).collect()
    }

    pub fn home_info(&self) -> String {
        let room_info: Vec<String> = self.list_rooms();
        format!(
            "Smart Home: {}\n Total rooms:{}\n {}",
            self.name,
            room_info.len(),
            room_info.join("\n--------------------------------------\n")
        )
    }
}
