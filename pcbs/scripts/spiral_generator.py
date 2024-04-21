from enum import Enum
from typing import List

from outline_primitive import Polygon2D
import json
from pcb_definitions import BoardData, Unit, Layer, Track
import tkinter as tk
from tkinter import filedialog
from json_plot import plot_tracks
from shapely.geometry import Point
import math


class Direction(Enum):
    CW = "CW"
    CCW = "CCW"

class PolySpiral:
    def __init__(self, poly: Polygon2D, start_angle, turns, width, spacing, layer, direction: Direction):
        self.poly = poly
        self.width = width
        self.spacing = spacing
        self.layer = layer
        self.direction = direction
        self.turns = turns
        self.start_ange = start_angle

    def add2board(self, board):
        degree_step = 5.0
        angle_step = math.radians(degree_step)
        angle_end = self.turns*2*math.pi
        if self.direction is Direction.CCW:
            angle_step = -angle_step
            angle_end = -angle_end
        angle_end += math.radians(self.start_ange)
        angle = math.radians(self.start_ange)
        inwards_step = degree_step / 360 * (self.width + self.spacing)

        inwards = 0
        ray_length, ray_angle = self.poly.shoot_ray(angle)
        dist = ray_length - inwards/math.sin(ray_angle)
        last_point = Point(self.poly.center.x+math.cos(angle)*dist, self.poly.center.y+math.sin(angle)*dist)
        while (self.direction is Direction.CCW and angle > angle_end) or (self.direction is Direction.CW and angle < angle_end):

            angle += angle_step
            inwards += inwards_step

            ray_length, ray_angle = self.poly.shoot_ray(angle)
            dist = ray_length - inwards/math.sin(ray_angle)
            next_point = Point(self.poly.center.x + math.cos(angle) * dist, self.poly.center.y + math.sin(angle) * dist)
            board.tracks.append(Track(start=last_point, end=next_point, width=self.width, layer=self.layer))
            last_point = next_point
        return


def rotate(point, angle_radians):
    x_rotated = point.x * math.cos(angle_radians) - point.y * math.sin(angle_radians)
    y_rotated = point.x * math.sin(angle_radians) + point.y * math.cos(angle_radians)
    return Point(x_rotated, y_rotated)


def translate(point, offset):
    return Point(point.x + offset.x, point.y + offset.y)



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
    # Example BoardData
    unit = Unit.MM
    tracks = []
    board_data = BoardData(unit=unit, tracks=tracks)
    vertices_truncated_hex = [Point(57.000, 8.660),
                              Point(31.00000,53.69358),
                              Point(-31.00000,53.69358),
                              Point(-62.00000,0.0),
                              Point(-31.00000,-53.69358),
                              Point(31.00000,-53.69358),
                              Point(57.00000,-8.66025)]
    user_specified_center = Point(0.0, 0.0)
    square = Polygon2D(vertices_truncated_hex, user_specified_center)
    PsP = PolySpiral(square, 90 , 33, min_width, min_spacing, Layer.BOTTOM_COPPER, Direction.CCW)

    PsP.add2board(board_data)
    plot_tracks(board_data)
    file_path = open_file_dialog()

    if not file_path:
        print("No file selected. Exiting.")
        return

    save_to_json(board_data, file_path)
    print(f"BoardData saved to {file_path}")


if __name__ == "__main__":
    main()
