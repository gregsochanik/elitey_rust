extends Control
## A 2D chart of the systems. Click one to select it; arrow keys pan, plus and minus or the wheel zoom.

signal system_selected(name: String)
signal jump_asked

const Theme_ := preload("res://theme/elite_theme.gd")

var galaxy: Dictionary = {}
var selected := ""
var hovered := ""
var route: Array = []
## Where the view is centred, in galaxy coordinates, and how far it is zoomed in (1 fits the jump range)
var view_centre := Vector2.ZERO
var zoom := 1.0
## Set when the screen opens, so the fresh chart data that follows is what the view centres on
var centre_on_next_data := false

const PAN_SPEED := 340.0
const ZOOM_STEP := 1.25
const WHEEL_STEP := 1.12
const MAX_ZOOM := 8.0
const LABEL_ZOOM := 0.6
const LABEL_ALL_ZOOM := 2.2
const FUEL_LABEL_ZOOM := 0.9


## Highlight a path through the stars, as a list of system names
func set_route(names: Array) -> void:
	route = names
	queue_redraw()


func show_galaxy(data: Dictionary) -> void:
	var first := galaxy.is_empty()
	galaxy = data
	if first or centre_on_next_data:
		reset_view()
	queue_redraw()


func set_selected(name: String) -> void:
	selected = name
	queue_redraw()


## Centre on the commander's star at the default zoom
func reset_view() -> void:
	zoom = 1.0
	for system in galaxy.get("systems", []):
		if system["is_current"]:
			view_centre = Vector2(system["x"], system["y"])
	queue_redraw()


## Move the view; `direction` is a unit-ish vector in screen directions
func pan(direction: Vector2, seconds: float) -> void:
	view_centre += direction * PAN_SPEED * seconds / _scale()
	view_centre = view_centre.clamp(Vector2.ZERO, Vector2(256, 256))
	queue_redraw()


## Zoom by a factor around a point on the map (the middle when none is given)
func zoom_by(factor: float, around := Vector2(-1, -1)) -> void:
	var anchor := size / 2.0 if around.x < 0 else around
	var before := _to_galaxy(anchor)
	zoom = clampf(zoom * factor, _min_zoom(), MAX_ZOOM)
	view_centre += before - _to_galaxy(anchor)
	queue_redraw()


## Select the nearest star in a direction from the selected one (or from the commander's star)
## and bring it to the middle of the view.
func select_toward(direction: Vector2) -> void:
	var from := Vector2.ZERO
	var from_name := selected
	for system in galaxy.get("systems", []):
		if system["name"] == selected or (selected == "" and system["is_current"]):
			from = Vector2(system["x"], system["y"])
			from_name = system["name"]
	var best := ""
	var best_score := INF
	for system in galaxy.get("systems", []):
		if system["name"] == from_name:
			continue
		var offset := Vector2(system["x"], system["y"]) - from
		var along := offset.normalized().dot(direction.normalized())
		if along < 0.5:
			continue
		var score := offset.length() / along
		if score < best_score:
			best_score = score
			best = system["name"]
	if best == "":
		return
	selected = best
	for system in galaxy["systems"]:
		if system["name"] == best:
			view_centre = Vector2(system["x"], system["y"])
	system_selected.emit(best)
	queue_redraw()


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_STOP
	focus_mode = Control.FOCUS_ALL
	clip_contents = true
	resized.connect(queue_redraw)
	visibility_changed.connect(_on_visibility_changed)


## Opening the map starts it centred on the commander, ready for the keys
func _on_visibility_changed() -> void:
	if is_visible_in_tree():
		reset_view()
		grab_focus()
		centre_on_next_data = true
		await get_tree().process_frame
		centre_on_next_data = false


func _process(delta: float) -> void:
	if not has_focus():
		return
	var direction := Vector2(
		float(Input.is_key_pressed(KEY_RIGHT)) - float(Input.is_key_pressed(KEY_LEFT)),
		float(Input.is_key_pressed(KEY_DOWN)) - float(Input.is_key_pressed(KEY_UP)))
	if direction != Vector2.ZERO and not Input.is_key_pressed(KEY_SHIFT):
		pan(direction, delta)


## Pixels per grid unit at the default zoom: the jump range just fits the shorter side
func _base_scale() -> float:
	var reach := maxf(float(galaxy.get("fuel", 7)), 4.0) * 4.0 * 1.35
	return minf(size.x, size.y) / (reach * 2.0)


func _scale() -> float:
	return _base_scale() * zoom


## The zoom at which the whole galaxy fits
func _min_zoom() -> float:
	return minf(0.95 * minf(size.x, size.y) / (256.0 * _base_scale()), 1.0)


func _to_galaxy(at: Vector2) -> Vector2:
	return view_centre + (at - size / 2.0) / _scale()


## Screen position of every system
func _layout() -> Dictionary:
	var positions := {}
	var scale_factor := _scale()
	for system in galaxy.get("systems", []):
		var p := Vector2(system["x"], system["y"])
		positions[system["name"]] = (p - view_centre) * scale_factor + size / 2.0
	return positions


func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, size), Theme_.PANEL)
	if has_focus():
		draw_rect(Rect2(Vector2.ONE, size - Vector2(2, 2)), Theme_.ACCENT.darkened(0.6), false, 2.0)
	if galaxy.is_empty():
		return
	var positions := _layout()
	var font := get_theme_default_font()
	var current_at := Vector2.ZERO
	for system in galaxy["systems"]:
		if system["is_current"] and positions.has(system["name"]):
			current_at = positions[system["name"]]

	if current_at != Vector2.ZERO:
		var reach_px := _reach_pixels(positions)
		draw_arc(current_at, reach_px, 0.0, TAU, 96, Theme_.ACCENT.darkened(0.55), 2.0)

	for i in range(route.size() - 1):
		if positions.has(route[i]) and positions.has(route[i + 1]):
			draw_line(positions[route[i]], positions[route[i + 1]], Theme_.WARNING, 2.5)

	var labels := []
	for system in galaxy["systems"]:
		if not positions.has(system["name"]):
			continue
		var at: Vector2 = positions[system["name"]]
		if at.x < -20 or at.y < -20 or at.x > size.x + 20 or at.y > size.y + 20:
			continue
		var colour := Theme_.MUTED.darkened(0.3)
		if system["visited"]:
			colour = Theme_.MUTED.lightened(0.15)
		var radius := 3.0
		var labelled := false
		if system["reachable"]:
			colour = Theme_.ACCENT
			radius = 6.0
			labelled = zoom >= LABEL_ZOOM
		if system["is_current"]:
			colour = Color.WHITE
			radius = 8.0
			labelled = true
		if system["name"] == selected:
			draw_arc(at, radius + 7.0, 0.0, TAU, 32, Theme_.WARNING, 3.0)
			labelled = true
		if system["name"] == hovered or zoom >= LABEL_ALL_ZOOM:
			labelled = true
		draw_circle(at, radius, colour)
		if system["frontier"]:
			draw_arc(at, radius + 6.0, 0.0, TAU, 24, Theme_.GOOD if system["discovered"] else Color("6fb1ff"), 1.5)
		if not system["visited"] and not system["is_current"]:
			draw_arc(at, radius + 3.0, 0.0, TAU, 24, Theme_.WARNING.darkened(0.2), 1.5)
		if system["is_current"]:
			draw_arc(current_at, radius + 4.0, 0.0, TAU, 32, Theme_.ACCENT, 2.0)
		if labelled:
			var priority := 0
			if system["is_current"] or system["name"] == selected or system["name"] == hovered:
				priority = 2
			elif system["reachable"]:
				priority = 1
			var fuel := ""
			if system["reachable"] and system["name"] != selected and zoom >= FUEL_LABEL_ZOOM:
				fuel = "%d ⛽" % system["fuel_cost"]
			labels.append({"name": system["name"], "at": at + Vector2(radius + 6.0, 6.0), "fuel": fuel, "priority": priority})
	_draw_labels(font, labels)


## Draw star names most important first, skipping any that would sit on top of one already drawn
func _draw_labels(font: Font, labels: Array) -> void:
	labels.sort_custom(func(a, b): return a["priority"] > b["priority"])
	var taken: Array[Rect2] = []
	for label in labels:
		var width := font.get_string_size(label["name"], HORIZONTAL_ALIGNMENT_LEFT, -1, Theme_.FONT_BODY).x
		var rect := Rect2(label["at"] + Vector2(0, -16), Vector2(width, 36))
		var overlaps := false
		for other in taken:
			if other.intersects(rect):
				overlaps = true
				break
		if overlaps and label["priority"] < 2:
			continue
		taken.append(rect)
		draw_string(font, label["at"], label["name"], HORIZONTAL_ALIGNMENT_LEFT, -1, Theme_.FONT_BODY, Theme_.TEXT)
		if label["fuel"] != "":
			draw_string(font, label["at"] + Vector2(0, 18), label["fuel"], HORIZONTAL_ALIGNMENT_LEFT, -1, Theme_.FONT_SMALL, Theme_.MUTED)


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
	if event is InputEventKey and event.pressed:
		match event.keycode:
			KEY_LEFT, KEY_RIGHT, KEY_UP, KEY_DOWN:
				if event.shift_pressed:
					select_toward(Vector2(
						float(event.keycode == KEY_RIGHT) - float(event.keycode == KEY_LEFT),
						float(event.keycode == KEY_DOWN) - float(event.keycode == KEY_UP)))
				accept_event()
			KEY_ENTER, KEY_KP_ENTER:
				jump_asked.emit()
				accept_event()
			KEY_EQUAL, KEY_PLUS, KEY_KP_ADD:
				zoom_by(ZOOM_STEP)
				accept_event()
			KEY_HOME:
				reset_view()
				accept_event()
			KEY_MINUS, KEY_KP_SUBTRACT:
				zoom_by(1.0 / ZOOM_STEP)
				accept_event()
	elif event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_WHEEL_UP:
		zoom_by(WHEEL_STEP, event.position)
	elif event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
		zoom_by(1.0 / WHEEL_STEP, event.position)
	elif event is InputEventMouseMotion:
		var near := _nearest(event.position)
		if near != hovered:
			hovered = near
			queue_redraw()
	elif event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		grab_focus()
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
