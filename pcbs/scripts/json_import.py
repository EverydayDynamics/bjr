import pcbnew
import json
import wx
import math
from pcb_definitions import BoardData, Unit, Track, Point, Layer

LAYER_DEFINITIONS = {
    Layer.TOP_COPPER: pcbnew.F_Cu,
    Layer.BOTTOM_COPPER: pcbnew.B_Cu,
    Layer.INNER_COPPER_1: pcbnew.In1_Cu,
    Layer.INNER_COPPER_2: pcbnew.In2_Cu,
    Layer.INNER_COPPER_3: pcbnew.In3_Cu,
    Layer.INNER_COPPER_4: pcbnew.In4_Cu,
    Layer.INNER_COPPER_5: pcbnew.In5_Cu,
    Layer.INNER_COPPER_6: pcbnew.In6_Cu,
    Layer.INNER_COPPER_7: pcbnew.In7_Cu,
    Layer.INNER_COPPER_8: pcbnew.In8_Cu,
    # Add more layer definitions as needed
}


def create_mm_track(board, group, track):
    layer = LAYER_DEFINITIONS[track.layer]
    pcbtrack = pcbnew.PCB_TRACK(board)
    pcbtrack.SetStart(pcbnew.VECTOR2I(int(track.start.x * 1e6), int(track.start.y * 1e6)))
    pcbtrack.SetEnd(pcbnew.VECTOR2I(int(track.end.x * 1e6), int(track.end.y * 1e6)))
    pcbtrack.SetWidth(int(track.width * 1e6))
    pcbtrack.SetLayer(layer)

    board.Add(pcbtrack)
    # group.AddItem(pcbtrack)


class JsonImportPlugin(pcbnew.ActionPlugin):
    def defaults(self):
        self.name = "Import JSON"
        self.category = "import"
        self.description = "import a cusom plugin"
        # self.show_toolbar_button = False # Optional, defaults to False
        # self.icon_file_name = os.path.join(os.path.dirname(__file__), 'simple_plugin.png') # Optional, defaults to ""

    def Run(self):
        dialog = wx.FileDialog(None, "Choose a json file", "", "", "*.json", wx.FD_OPEN)
        if dialog.ShowModal() == wx.ID_OK:
            # read the file
            with open(dialog.GetPath(), "r") as f:
                # load up the JSON with the coil parameters
                json_dict = json.load(f)

                # Map dictionary to BoardData instance
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
                board = pcbnew.GetBoard()
                pcb_group = pcbnew.PCB_GROUP(board)
                for track in board_data.tracks:
                    create_mm_track(board, pcb_group, track)


JsonImportPlugin().register()
