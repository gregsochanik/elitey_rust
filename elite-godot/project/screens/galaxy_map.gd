extends Control
## A 2D chart of the systems. Click one to select it.

signal system_selected(name: String)

const Theme_ := preload("res://theme/elite_theme.gd")

var galaxy: Dictionary = {}
var selected := ""
var local_view := true
var hovered := ""
var route: Array = []


## Highlight a path through the stars, as a list of system names
func set_route(names: Array) -> void:
	route = names
	queue_redraw()


func show_galaxy(data: Dictionary) -> void:
	galaxy = data
	queue_redraw()


func set_selected(name: String) -> void:
	selected = name
	queue_redraw()


func set_local(local: bool) -> void:
	local_view = local
	queue_redraw()


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_STOP
	clip_contents = true
	resized.connect(queue_redraw)


## Screen position of every system: the nearby region around the ship, or the whole galaxy.
func _layout() -> Dictionary:
	var positions := {}
	if galaxy.is_empty():
		return positions
	var systems: Array = galaxy["systems"]
	var low := Vector2(INF, INF)
	var high := Vector2(-INF, -INF)
	var centre := Vector2.ZERO
	for system in systems:
		if system["is_current"]:
			centre = Vector2(system["x"], system["y"])
	if local_view:
		var reach := maxf(float(galaxy["fuel"]), 4.0) * 4.0 * 1.35
		low = centre - Vector2(reach, reach)
		high = centre + Vector2(reach, reach)
	else:
		for system in systems:
			var p := Vector2(system["x"], system["y"])
			low = Vector2(minf(low.x, p.x), minf(low.y, p.y))
			high = Vector2(maxf(high.x, p.x), maxf(high.y, p.y))
	var span := high - low
	var margin := 40.0
	var scale_factor := minf((size.x - margin * 2.0) / maxf(span.x, 1.0), (size.y - margin * 2.0) / maxf(span.y, 1.0))
	var offset := (size - span * scale_factor) / 2.0
	for system in systems:
		var p := Vector2(system["x"], system["y"])
		var at := (p - low) * scale_factor + offset
		if local_view and (at.x < 0 or at.y < 0 or at.x > size.x or at.y > size.y):
			continue
		positions[system["name"]] = at
	return positions


func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, size), Theme_.PANEL)
	if galaxy.is_empty():
		return
	var positions := _layout()
	var font := get_theme_default_font()
	var current_at := Vector2.ZERO
	for system in galaxy["systems"]:
		if system["is_current"] and positions.has(system["name"]):
			current_at = positions[system["name"]]

	if local_view and current_at != Vector2.ZERO:
		var reach_px := _reach_pixels(positions)
		draw_arc(current_at, reach_px, 0.0, TAU, 96, Theme_.ACCENT.darkened(0.55), 2.0)

	for i in range(route.size() - 1):
		if positions.has(route[i]) and positions.has(route[i + 1]):
			draw_line(positions[route[i]], positions[route[i + 1]], Theme_.WARNING, 2.5)

	for system in galaxy["systems"]:
		if not positions.has(system["name"]):
			continue
		var at: Vector2 = positions[system["name"]]
		var colour := Theme_.MUTED.darkened(0.3)
		if system["visited"]:
			colour = Theme_.MUTED.lightened(0.15)
		var radius := 3.0
		var labelled := false
		if system["reachable"]:
			colour = Theme_.ACCENT
			radius = 6.0
			labelled = true
		if system["is_current"]:
			colour = Color.WHITE
			radius = 8.0
			labelled = true
		if system["name"] == selected:
			draw_arc(at, radius + 7.0, 0.0, TAU, 32, Theme_.WARNING, 3.0)
			labelled = true
		if system["name"] == hovered:
			labelled = true
		draw_circle(at, radius, colour)
		if system["frontier"]:
			draw_arc(at, radius + 6.0, 0.0, TAU, 24, Theme_.GOOD if system["discovered"] else Color("6fb1ff"), 1.5)
		if not system["visited"] and not system["is_current"]:
			draw_arc(at, radius + 3.0, 0.0, TAU, 24, Theme_.WARNING.darkened(0.2), 1.5)
		if system["is_current"] and local_view:
			draw_arc(current_at, radius + 4.0, 0.0, TAU, 32, Theme_.ACCENT, 2.0)
		if labelled:
			draw_string(font, at + Vector2(radius + 6.0, 6.0), system["name"], HORIZONTAL_ALIGNMENT_LEFT, -1, Theme_.FONT_BODY, Theme_.TEXT)
		if system["reachable"] and system["name"] != selected and local_view:
			draw_string(font, at + Vector2(radius + 6.0, 24.0), "%d ⛽" % system["fuel_cost"], HORIZONTAL_ALIGNMENT_LEFT, -1, Theme_.FONT_SMALL, Theme_.MUTED)


## Pixels per light year of fuel, from the nearest reachable system
func _reach_pixels(positions: Dictionary) -> float:
	var current_at := Vector2.ZERO
	for system in galaxy["systems"]:
		if system["is_current"]:
			current_at = positions[system["name"]]
	for system in galaxy["systems"]:
		if system["distance"] > 0.5 and positions.has(system["name"]):
			var pixels := current_at.distance_to(positions[system["name"]]) / float(system["distance"])
			return pixels * float(galaxy["fuel"])
	return 0.0


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		var near := _nearest(event.position)
		if near != hovered:
			hovered = near
			queue_redraw()
	elif event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		var near := _nearest(event.position)
		if near != "":
			selected = near
			system_selected.emit(near)
			queue_redraw()


func _nearest(at: Vector2) -> String:
	var best := ""
	var best_distance := 18.0
	var positions := _layout()
	for name in positions:
		var distance: float = (positions[name] as Vector2).distance_to(at)
		if distance < best_distance:
			best_distance = distance
			best = name
	return best
