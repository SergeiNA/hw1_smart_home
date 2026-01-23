#!/usr/bin/env python3
"""
Integration tests for Smart Home API server.
Run with: pytest test_api.py -v
"""

import requests
import pytest

BASE_URL = "http://localhost:8888/api/v1"


class TestRooms:
    """Tests for room endpoints."""

    def test_add_room(self):
        """Test adding a new room."""
        response = requests.post(
            f"{BASE_URL}/room/add_room",
            json={"name": "Test Room 1"}
        )
        assert response.status_code == 200

    def test_list_rooms(self):
        """Test listing all rooms."""
        # First add a room
        requests.post(
            f"{BASE_URL}/room/add_room",
            json={"name": "List Test Room"}
        )

        response = requests.get(f"{BASE_URL}/room/room_list")
        assert response.status_code == 200

        data = response.json()
        assert "rooms" in data
        assert isinstance(data["rooms"], list)

    def test_get_room(self):
        """Test getting a specific room."""
        room_name = "Get Room Test"

        # Create room first
        requests.post(
            f"{BASE_URL}/room/add_room",
            json={"name": room_name}
        )

        response = requests.get(f"{BASE_URL}/room/{room_name}")
        assert response.status_code == 200

    def test_get_room_not_found(self):
        """Test getting a non-existent room."""
        response = requests.get(f"{BASE_URL}/room/NonExistentRoom12345")
        assert response.status_code == 404

    def test_delete_room(self):
        """Test deleting a room."""
        room_name = "Room To Delete"

        # Create room first
        requests.post(
            f"{BASE_URL}/room/add_room",
            json={"name": room_name}
        )

        # Delete room
        response = requests.delete(f"{BASE_URL}/room/{room_name}")
        assert response.status_code == 200

        # Verify room is deleted
        response = requests.get(f"{BASE_URL}/room/{room_name}")
        assert response.status_code == 404


class TestDevices:
    """Tests for device endpoints."""

    def test_add_outlet_device(self):
        """Test adding an outlet device to a room."""
        room_name = "Device Test Room 1"

        # Create room first
        requests.post(
            f"{BASE_URL}/room/add_room",
            json={"name": room_name}
        )

        # Add outlet device
        response = requests.post(
            f"{BASE_URL}/room/{room_name}/device",
            json={
                "name": "Test Outlet",
                "device_type": {
                    "type": "outlet",
                    "params": {"power": 120}
                }
            }
        )
        assert response.status_code == 200

    def test_add_thermometer_device(self):
        """Test adding a thermometer device to a room."""
        room_name = "Device Test Room 2"

        # Create room first
        requests.post(
            f"{BASE_URL}/room/add_room",
            json={"name": room_name}
        )

        # Add thermometer device
        response = requests.post(
            f"{BASE_URL}/room/{room_name}/device",
            json={
                "name": "Test Thermometer",
                "device_type": {
                    "type": "thermometer",
                    "params": {"temperature": 22.5}
                }
            }
        )
        assert response.status_code == 200

    def test_list_devices(self):
        """Test listing devices in a room."""
        room_name = "Device List Test Room"

        # Create room
        requests.post(
            f"{BASE_URL}/room/add_room",
            json={"name": room_name}
        )

        # Add a device
        requests.post(
            f"{BASE_URL}/room/{room_name}/device",
            json={
                "name": "List Test Device",
                "device_type": {
                    "type": "outlet",
                    "params": {"power": 100}
                }
            }
        )

        # List devices
        response = requests.get(f"{BASE_URL}/room/{room_name}/devices")
        assert response.status_code == 200

        data = response.json()
        assert "devices" in data
        assert isinstance(data["devices"], list)

    def test_get_device_report(self):
        """Test getting a device report."""
        room_name = "Device Report Room"
        device_name = "Report Test Device"

        # Create room
        requests.post(
            f"{BASE_URL}/room/add_room",
            json={"name": room_name}
        )

        # Add device
        requests.post(
            f"{BASE_URL}/room/{room_name}/device",
            json={
                "name": device_name,
                "device_type": {
                    "type": "outlet",
                    "params": {"power": 150}
                }
            }
        )

        # Get device report
        response = requests.get(
            f"{BASE_URL}/room/{room_name}/device/{device_name}/report"
        )
        assert response.status_code == 200

    def test_delete_device(self):
        """Test deleting a device from a room."""
        room_name = "Device Delete Room"
        device_name = "Device To Delete"

        # Create room
        requests.post(
            f"{BASE_URL}/room/add_room",
            json={"name": room_name}
        )

        # Add device
        requests.post(
            f"{BASE_URL}/room/{room_name}/device",
            json={
                "name": device_name,
                "device_type": {
                    "type": "outlet",
                    "params": {"power": 100}
                }
            }
        )

        # Delete device
        response = requests.delete(
            f"{BASE_URL}/room/{room_name}/device/{device_name}"
        )
        assert response.status_code == 200

    def test_add_device_to_nonexistent_room(self):
        """Test adding a device to a room that doesn't exist."""
        response = requests.post(
            f"{BASE_URL}/room/NonExistentRoom99999/device",
            json={
                "name": "Test Device",
                "device_type": {
                    "type": "outlet",
                    "params": {"power": 100}
                }
            }
        )
        assert response.status_code == 404


class TestReports:
    """Tests for report endpoints."""

    def test_home_report(self):
        """Test getting the home report."""
        response = requests.get(f"{BASE_URL}/home/report")
        assert response.status_code == 200

        data = response.json()
        assert "report" in data
        assert isinstance(data["report"], str)

    def test_room_report(self):
        """Test getting a room report."""
        room_name = "Room Report Test"

        # Create room
        requests.post(
            f"{BASE_URL}/room/add_room",
            json={"name": room_name}
        )

        # Get room report
        response = requests.get(f"{BASE_URL}/room/{room_name}/report")
        assert response.status_code == 200

        data = response.json()
        assert "report" in data


class TestEdgeCases:
    """Tests for edge cases and error handling."""

    def test_add_duplicate_room(self):
        """Test adding a room with the same name twice."""
        room_name = "Duplicate Room Test"

        # Add room first time
        requests.post(
            f"{BASE_URL}/room/add_room",
            json={"name": room_name}
        )

        # Add room second time (should fail or handle gracefully)
        response = requests.post(
            f"{BASE_URL}/room/add_room",
            json={"name": room_name}
        )
        # Expecting either 409 Conflict or 400 Bad Request
        assert response.status_code in [400, 409]

    def test_invalid_device_type(self):
        """Test adding a device with invalid type."""
        room_name = "Invalid Device Type Room"

        # Create room
        requests.post(
            f"{BASE_URL}/room/add_room",
            json={"name": room_name}
        )

        # Try to add device with invalid type
        response = requests.post(
            f"{BASE_URL}/room/{room_name}/device",
            json={
                "name": "Invalid Device",
                "device_type": {
                    "type": "invalid_type",
                    "params": {}
                }
            }
        )
        assert response.status_code == 422  # Unprocessable Entity

#     def test_empty_room_name(self):
#         """Test adding a room with empty name."""
#         response = requests.post(
#             f"{BASE_URL}/room/add_room",
#             json={"name": ""}
#         )
#         # Should fail validation
#         assert response.status_code in [400, 422]


if __name__ == "__main__":
    pytest.main([__file__, "-v"])