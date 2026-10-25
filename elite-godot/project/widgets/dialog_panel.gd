extends Control
## A modal dialog over a dimmed screen: subtitle, title, text, and buttons that also answer to the number keys.

const Theme_ := preload("res://theme/elite_theme.gd")

var panel := PanelContainer.new()
var box := VBoxContainer.new()
var choices: Array[Button] = []


func _ready() -> void:
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_STOP
	visible = false

	var dim := ColorRect.new()
	dim.color = Color(0, 0, 0, 0.6)
	dim.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	dim.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(dim)

	var centre := CenterContainer.new()
	centre.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	centre.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(centre)

	var style := StyleBoxFlat.new()
	style.bg_color = Theme_.PANEL
	style.set_corner_radius_all(12)
	style.set_border_width_all(2)
	style.border_color = Theme_.ACCENT
	style.shadow_size = 18
	style.shadow_color = Color(0, 0, 0, 0.6)
	for side in ["left", "right"]:
		style.set("content_margin_" + side, 28)
	for side in ["top", "bottom"]:
		style.set("content_margin_" + side, 22)
	panel.add_theme_stylebox_override("panel", style)
	panel.custom_minimum_size = Vector2(580, 0)
	box.add_theme_constant_override("separation", 12)
	panel.add_child(box)
	centre.add_child(panel)


func clear() -> void:
	visible = false
	choices.clear()
	for child in box.get_children():
		box.remove_child(child)
		child.queue_free()


func subtitle(text: String) -> void:
	var label := Label.new()
	label.text = text.to_upper()
	label.add_theme_font_size_override("font_size", Theme_.FONT_SMALL)
	label.add_theme_color_override("font_color", Theme_.MUTED)
	box.add_child(label)


func title(text: String) -> void:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", Theme_.FONT_TITLE)
	label.add_theme_color_override("font_color", Theme_.ACCENT)
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(label)
	box.add_child(HSeparator.new())


func text(body: String, colour := Theme_.TEXT) -> void:
	var label := Label.new()
	label.text = body
	label.add_theme_color_override("font_color", colour)
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(label)


## A choice button; the first nine also answer to the keys 1 to 9.
func choice(label: String, action: Callable) -> Button:
	var button := Button.new()
	choices.append(button)
	button.text = "%d   %s" % [choices.size(), label] if choices.size() <= 9 else label
	button.alignment = HORIZONTAL_ALIGNMENT_LEFT
	button.custom_minimum_size = Vector2(0, 42)
	button.pressed.connect(action)
	box.add_child(button)
	return button


func present() -> void:
	visible = true


func _unhandled_key_input(event: InputEvent) -> void:
	if not visible or not event.pressed or event.echo:
		return
	var number: int = event.keycode - KEY_1
	if number >= 0 and number < choices.size() and not choices[number].disabled:
		choices[number].pressed.emit()
		get_viewport().set_input_as_handled()
