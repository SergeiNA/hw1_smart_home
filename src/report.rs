use crate::traits::Information;

pub struct Reporter<'a, T = ()> {
    item: T,
    _marker: std::marker::PhantomData<&'a ()>, // holds lifetime
}

impl<'a> Default for Reporter<'a, ()> {
    fn default() -> Self {
        Reporter {
            item: (),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> Reporter<'a, T> {
    pub fn append<U: Information + 'a>(self, item: &'a U) -> Reporter<'a, (T, &'a U)> {
        Reporter {
            item: (self.item, item),
            _marker: std::marker::PhantomData,
        }
    }
}

pub trait ReportAll {
    fn collect_all(&self) -> Vec<String>;
}

impl ReportAll for () {
    fn collect_all(&self) -> Vec<String> {
        vec![]
    }
}

impl<'a, Tail: ReportAll, Head: Information + 'a> ReportAll for (Tail, &'a Head) {
    fn collect_all(&self) -> Vec<String> {
        let mut reports = self.0.collect_all();
        reports.push(self.1.info());
        reports
    }
}

impl<'a, T: ReportAll> Reporter<'a, T> {
    pub fn report(&self) -> Vec<String> {
        self.item.collect_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_devices::{Celsius, Device, OutletState, Watt};
    use crate::smart_room::SmartRoom;
    use std::collections::HashMap;
    #[test]
    fn reporter_test_base() {
        let thermometer =
            Device::new_thermometer("Living Room Thermometer".to_string(), 22.5 as Celsius);
        let outlet = Device::new_outlet(
            "Living Room Outlet".to_string(),
            OutletState::On,
            150 as Watt,
        );
        let room_thermometer =
            Device::new_thermometer("Room Thermometer".to_string(), 21.0 as Celsius);
        let mut room = SmartRoom::new("Living Room".to_string(), HashMap::new());
        room.add_device("Thermometer".to_string(), room_thermometer);
        let reporter = Reporter::default()
            .append(&thermometer)
            .append(&outlet)
            .append(&room)
            .report();

        let expected = r#"Thermometer: Living Room Thermometer - Current Temperature: 22.50°C
Smart Outlet: Living Room Outlet - Current State: On, Power Usage: 150 Watt

Smart Room: Living Room:
 Total devices: 1
  [0]: Thermometer: Room Thermometer - Current Temperature: 21.00°C"#;

        assert_eq!(reporter.join("\n"), expected);
    }

    #[test]
    fn reporter_test_only_devices() {
        let thermometer_living_room =
            Device::new_thermometer("Living Room Thermometer".to_string(), 22.5 as Celsius);
        let thermometer_bedroom_room =
            Device::new_thermometer("Bed Room Thermometer".to_string(), 20.5 as Celsius);
        let outlet_living_room = Device::new_outlet(
            "Living Room Outlet".to_string(),
            OutletState::On,
            150 as Watt,
        );
        let outlet_bed_room =
            Device::new_outlet("Bed Room Outlet".to_string(), OutletState::On, 220 as Watt);
        let reporter = Reporter::default()
            .append(&thermometer_living_room)
            .append(&thermometer_bedroom_room)
            .append(&outlet_living_room)
            .append(&outlet_bed_room)
            .report();

        let expected = r#"Thermometer: Living Room Thermometer - Current Temperature: 22.50°C
Thermometer: Bed Room Thermometer - Current Temperature: 20.50°C
Smart Outlet: Living Room Outlet - Current State: On, Power Usage: 150 Watt
Smart Outlet: Bed Room Outlet - Current State: On, Power Usage: 220 Watt"#;
        assert_eq!(reporter.join("\n"), expected);
    }
}
