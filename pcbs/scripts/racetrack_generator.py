from enum import Enum

from pcb_definitions import BoardData, Track, Layer
import json
from pcb_definitions import BoardData, Unit, Layer, Point, Track
import tkinter as tk
from tkinter import filedialog
from json_plot import plot_tracks
import math


class Direction(Enum):
    CW = "CW"
    CCW = "CCW"


class Layer2Racetrack:
    def __init__(self, center, length, od, id, width, spacing, layers, tilt, directions: [Direction]):
        self.racetracks = []
        self.racetracks.append(Racetrack(center, length, od, id, width, spacing, layers[0], tilt, directions[0]))
        self.racetracks.append(Racetrack(center, length, od, id, width, spacing, layers[1], tilt, directions[1]))

    def add2board(self, board):
        for racetrack in self.racetracks:
            racetrack.add2board(board)


class Racetrack:
    def __init__(self, center, length, od, id, width, spacing, layer, tilt, direction: Direction):
        self.center = center
        self.length = length
        self.od = od
        self.id = id
        self.width = width
        self.spacing = spacing
        self.layer = layer
        self.tilt = tilt
        self.direction = direction

    def add2board(self, board):
        tilt_radians = math.radians(self.tilt)
        sensing_length = self.length - self.od
        degree_step = 5.0
        if self.direction is Direction.CW:
            angle_step = math.radians(degree_step)
        if self.direction is Direction.CCW:
            angle_step = -math.radians(degree_step)
        angle = 0.0
        radius_step = degree_step / 360 * (self.width + self.spacing)
        radius = self.od / 2.0
        last_point = translate(rotate(get_racetrack_point(angle, radius, sensing_length), tilt_radians), self.center)
        while radius > self.id / 2.0:
            radius -= radius_step
            angle += angle_step
            next_point = translate(rotate(get_racetrack_point(angle, radius, sensing_length), tilt_radians),
                                   self.center)
            board.tracks.append(Track(start=last_point, end=next_point, width=self.width, layer=self.layer))
            last_point = next_point

        return


def rotate(point, angle_radians):
    x_rotated = point.x * math.cos(angle_radians) - point.y * math.sin(angle_radians)
    y_rotated = point.x * math.sin(angle_radians) + point.y * math.cos(angle_radians)
    return Point(x_rotated, y_rotated)


def translate(point, offset):
    return Point(point.x + offset.x, point.y + offset.y)


def get_racetrack_point(angle, radius, sensing_length):
    offset = sensing_length / 2.0
    if math.cos(angle) <= 0.0:
        offset *= -1.0
    result = Point(radius * math.cos(angle) + offset, radius * math.sin(angle))
    return result


def save_to_json(board_data, file_path):
    data = {
        "unit": board_data.unit.value,
        "tracks": [
            {
                "start": {"x": track.start.x, "y": track.start.y},
                "end": {"x": track.end.x, "y": track.end.y},
                "width": track.width,
                "layer": track.layer.value
            }
            for track in board_data.tracks
        ]
    }

    with open(file_path, 'w') as file:
        json.dump(data, file, indent=2)


def open_file_dialog():
    root = tk.Tk()
    root.withdraw()

    file_path = filedialog.asksaveasfilename(title="Save As", defaultextension=".json",
                                             filetypes=[("JSON files", "*.json")])

    return file_path


def main():
    mil2mm = 0.0254
    min_width = 5 * mil2mm
    min_spacing = 5 * mil2mm
    min_id = 0.6
    # Example BoardData
    unit = Unit.MM
    tracks = [
    ]

    board_data = BoardData(unit=unit, tracks=tracks)
    rt111 = Layer2Racetrack(center=Point(-20.0, 0.0),
                          length=80.5,
                          od=20.0,
                          id=min_id,
                          tilt=90.0,
                          width=min_width,
                          spacing=min_spacing,
                          layers=[Layer.TOP_COPPER, Layer.INNER_COPPER_2],
                          directions=[Direction.CW, Direction.CCW]
                          )

    rt112 = Layer2Racetrack(center=Point(0.0, 0.0),
                          length=80.5,
                          od=20.0,
                          id=min_id,
                          tilt=90.0,
                          width=min_width,
                          spacing=min_spacing,
                          layers=[Layer.TOP_COPPER, Layer.INNER_COPPER_2],
                          directions=[Direction.CW, Direction.CCW]
                          )
    rt113 = Layer2Racetrack(center=Point(20.0, 0.0),
                           length=80.5,
                           od=20.0,
                           id=min_id,
                           tilt=90.0,
                           width=min_width,
                           spacing=min_spacing,
                           layers=[Layer.TOP_COPPER, Layer.INNER_COPPER_2],
                           directions=[Direction.CW, Direction.CCW]
                           )
    rt121 = Layer2Racetrack(center=Point(0.0, -15.0),
                          length=80.5,
                          od=20.0,
                          id=min_id,
                          tilt=0.0,
                          width=min_width,
                          spacing=min_spacing,
                          layers=[Layer.INNER_COPPER_1, Layer.BOTTOM_COPPER],
                          directions=[Direction.CW, Direction.CCW]
                          )

    rt122 = Layer2Racetrack(center=Point(00.0, 15.0),
                          length=80.5,
                          od=20.0,
                          id=min_id,
                          tilt=0.0,
                          width=min_width,
                          spacing=min_spacing,
                          layers=[Layer.INNER_COPPER_1, Layer.BOTTOM_COPPER],
                          directions=[Direction.CW, Direction.CCW]
                          )
    rt111.add2board(board_data)
    rt112.add2board(board_data)
    rt113.add2board(board_data)
    rt121.add2board(board_data)
    rt122.add2board(board_data)
    cluster2_centre = Point(90.0, 0.0)
    rt211 = Layer2Racetrack(center=cluster2_centre + Point(-20, 0),
                           length=80.5,
                           od=20.0,
                           id=4,
                           tilt=90.0,
                           width=min_width,
                           spacing=min_spacing,
                           layers=[Layer.TOP_COPPER, Layer.INNER_COPPER_2],
                           directions=[Direction.CW, Direction.CCW]
                           )

    rt212 = Layer2Racetrack(center=cluster2_centre + Point(0, 0),
                           length=80.5,
                           od=20.0,
                           id=4,
                           tilt=90.0,
                           width=min_width,
                           spacing=min_spacing,
                           layers=[Layer.TOP_COPPER, Layer.INNER_COPPER_2],
                           directions=[Direction.CW, Direction.CCW]
                           )
    rt213 = Layer2Racetrack(center=cluster2_centre + Point(20.0, 0.0),
                           length=80.5,
                           od=20.0,
                           id=4,
                           tilt=90.0,
                           width=min_width,
                           spacing=min_spacing,
                           layers=[Layer.TOP_COPPER, Layer.INNER_COPPER_2],
                           directions=[Direction.CW, Direction.CCW]
                           )
    rt221 = Layer2Racetrack(center=cluster2_centre + Point(00.0, -15.0),
                           length=80.5,
                           od=20.0,
                           id=4,
                           tilt=0.0,
                           width=min_width,
                           spacing=min_spacing,
                           layers=[Layer.INNER_COPPER_1, Layer.BOTTOM_COPPER],
                           directions=[Direction.CW, Direction.CCW]
                           )

    rt222 = Layer2Racetrack(center=cluster2_centre + Point(00.0, 15.0),
                           length=80.5,
                           od=20.0,
                           id=4,
                           tilt=0.0,
                           width=min_width,
                           spacing=min_spacing,
                           layers=[Layer.INNER_COPPER_1, Layer.BOTTOM_COPPER],
                           directions=[Direction.CW, Direction.CCW]
                           )
    rt211.add2board(board_data)
    rt212.add2board(board_data)
    rt213.add2board(board_data)
    rt221.add2board(board_data)
    rt222.add2board(board_data)
    plot_tracks(board_data)
    file_path = open_file_dialog()

    if not file_path:
        print("No file selected. Exiting.")
        return

    save_to_json(board_data, file_path)
    print(f"BoardData saved to {file_path}")


if __name__ == "__main__":
    main()
