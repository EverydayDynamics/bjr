import json
from pcb_definitions import BoardData, Unit, Layer, Point, Track
import tkinter as tk
from tkinter import filedialog

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

    file_path = filedialog.asksaveasfilename(title="Save As", defaultextension=".json", filetypes=[("JSON files", "*.json")])

    return file_path

def main():
    # Example BoardData
    unit = Unit.MM
    tracks = [
        Track(start=Point(0.0, 0.0), end=Point(10.0, 5.0), width=0.2, layer=Layer.TOP_COPPER),
        Track(start=Point(5.0, 5.0), end=Point(15.0, 10.0), width=0.3, layer=Layer.BOTTOM_COPPER)
    ]

    board_data = BoardData(unit=unit, tracks=tracks)

    file_path = open_file_dialog()

    if not file_path:
        print("No file selected. Exiting.")
        return

    save_to_json(board_data, file_path)
    print(f"BoardData saved to {file_path}")

if __name__ == "__main__":
    main()