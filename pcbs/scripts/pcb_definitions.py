from enum import Enum
from typing import List, NamedTuple
from dataclasses import dataclass, asdict
from enum import Enum
from typing import List


@dataclass
class Point:
    x: float
    y: float

    def __add__(self, other):
        return Point(self.x + other.x, self.y + other.y)


@dataclass
class Track:
    start: Point
    end: Point
    width: float
    layer: "Layer"


@dataclass
class BoardData:
    unit: "Unit"
    tracks: List[Track]


class Unit(Enum):
    MM = "mm"
    MIL = "mil"


class Layer(Enum):
    TOP_COPPER = "Tc"
    BOTTOM_COPPER = "Bc"
    INNER_COPPER_1 = "Ic1"
    INNER_COPPER_2 = "Ic2"
    INNER_COPPER_3 = "Ic3"
    INNER_COPPER_4 = "Ic4"
    INNER_COPPER_5 = "Ic5"
    INNER_COPPER_6 = "Ic6"
    INNER_COPPER_7 = "Ic7"
    INNER_COPPER_8 = "Ic7"
