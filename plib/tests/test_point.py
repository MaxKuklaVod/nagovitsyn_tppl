import json
import pytest

from plib import Point

@pytest.fixture
def points():
    return Point(0, 0), Point(2, 2)

class TestPoint:

    def test_creation(self):
        p = Point(1, 2)
        assert p.x == 1 and p.y == 2

        with pytest.raises(TypeError):
            Point(1.5, 1.5)

    def test_add(self, points):
        p1, p2 = points
        assert p2 + p1 == Point(2, 2)
    
    def test_sub(self, points):
        p1, p2 = points
        assert p2 - p1 == Point(2, 2)
        assert p1 - p2 == -Point(2, 2)
    
    def test_distance_to(self):
        p1 = Point(0, 0)
        p2 = Point(2, 0)
        assert p1.to(p2) == 2

    @pytest.mark.parametrize(
            "p1, p2, distance",
            [(Point(0, 0), Point(0, 10), 10),
             (Point(0, 0), Point(10, 0), 10),
             (Point(0, 0), Point(1, 1), 1.414)]
    )
    def test_distance_all_axis(self, p1, p2, distance):
        assert p1.to(p2) == pytest.approx(distance, 0.001)

    def test_str_repr(self):
        p = Point(5, 8)
        assert str(p) == "Point(5, 8)"
        assert repr(p) == "Point(5, 8)"

    def test_is_center(self):
        assert Point(0, 0).is_center() is True
        assert Point(0, 1).is_center() is False
        assert Point(1, 0).is_center() is False

    def test_json_serialization(self):
        p = Point(10, -20)
        json_string = p.to_json()
        
        assert json.loads(json_string) == {"x": 10, "y": -20}

        p_from_json = Point.from_json(json_string)
        assert p == p_from_json