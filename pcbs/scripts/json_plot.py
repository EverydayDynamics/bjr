import json
import tkinter as tk
from tkinter import filedialog
import matplotlib.pyplot as plt
from pcb_definitions import BoardData, Unit, Layer, Point, Track
import itertools


def open_file_dialog():
    root = tk.Tk()
    root.withdraw()

    file_path = filedialog.askopenfilename(title="Select JSON File", filetypes=[("JSON files", "*.json")])

    return file_path


def plot_tracks(board_data):
    mm2pts = (72.0/25.4)
    plt.figure()
    plt.gca().set_aspect('equal', adjustable='box')
    # Create an iterator for cycling through colors
    colors = itertools.cycle(plt.rcParams['axes.prop_cycle'].by_key()['color'])

    layer_color_mapping = {}  # To store layer-color associations

    for track in board_data.tracks:
        layer = track.layer

        # Assign a color to the layer if not already assigned
        if layer not in layer_color_mapping:
            layer_color_mapping[layer] = next(colors)

        color = layer_color_mapping[layer]

        x_values = [track.start.x, track.end.x]
        y_values = [track.start.y, track.end.y]

        plt.plot(x_values, y_values, color=color)

    plt.title("PCB Tracks")
    plt.xlabel("X (mm)")
    plt.ylabel("Y (mm)")
    # plt.legend()
    plt.grid(True)
    # plt.savefig("pcb_tracks.png")
    plt.show()


def main():
    file_path = open_file_dialog()

    if not file_path:
        print("No file selected. Exiting.")
        return

    with open(file_path, 'r') as file:
        json_data = file.read()

    json_dict = json.loads(json_data)

    unit = Unit(json_dict['unit'])
    tracks = [
        Track(
            start=Point(track['start']['x'], track['start']['y']),
            end=Point(track['end']['x'], track['end']['y']),
            width=track['width'],
            layer=Layer(track['layer'])
        )
        for track in json_dict['tracks']
    ]

    board_data = BoardData(unit=unit, tracks=tracks)

    plot_tracks(board_data)


if __name__ == "__main__":
    main()
