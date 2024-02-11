from typing import List
import math
from shapely.geometry import Point, Polygon, LineString

class Polygon2D:
    def __init__(self, vertices: List[Point], center: Point):
        self.vertices = vertices
        self.center = center
        self.polygon = Polygon(vertices)

    def get_area(self) -> float:
        return self.polygon.area

    def angle_between_lines(self, line1, line2):
        # Calculate vectors
        vector1 = (line1.coords[1][0] - line1.coords[0][0], line1.coords[1][1] - line1.coords[0][1])
        vector2 = (line2.coords[1][0] - line2.coords[0][0], line2.coords[1][1] - line2.coords[0][1])

        # Calculate dot product
        dot_product = vector1[0] * vector2[0] + vector1[1] * vector2[1]

        # Calculate magnitudes
        magnitude1 = math.sqrt(vector1[0] ** 2 + vector1[1] ** 2)
        magnitude2 = math.sqrt(vector2[0] ** 2 + vector2[1] ** 2)

        # Calculate angle in radians
        angle_rad = math.acos(dot_product / (magnitude1 * magnitude2))

        # Convert radians to degrees
        angle_deg = math.degrees(angle_rad)

        return angle_deg
    def get_perimeter(self) -> float:
        return self.polygon.length

    def shoot_ray(self, angle: float) -> (float, float):
        # Convert angle to radians
        angle_rad = angle

        # Calculate the end point of the ray
        end_point_x = self.center.x + math.cos(angle_rad)*1000.0
        end_point_y = self.center.y + math.sin(angle_rad)*1000.0
        ray_line = LineString([self.center,Point(end_point_x, end_point_y)])
        for i in range(len(self.polygon.exterior.coords) - 1):
            segment_start = self.polygon.exterior.coords[i]
            segment_end = self.polygon.exterior.coords[i + 1]
            segment = LineString([segment_start,segment_end])
            intersection = segment.intersection(ray_line)

            if not intersection.is_empty:
                angle = self.angle_between_lines(ray_line, segment)
                return (self.center.distance(intersection), math.radians(angle))


        return (math.nan, math.nan)
        # Check for intersection with the polygon

        # If the ray intersects the polygon, return the distance
# Example usage