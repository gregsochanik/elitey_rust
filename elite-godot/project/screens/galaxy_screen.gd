extends HBoxContainer
## Chart of the neighbourhood with details and a jump plan for the selected system.

const Theme_ := preload("res://theme/elite_theme.gd")
const GalaxyMap := preload("res://screens/galaxy_map.gd")

var game: EliteGame
var map := GalaxyMap.new()
var title := Label.new()
var details := RichTextLabel.new()
var plan := Label.new()
var jump := Button.new()
var toggle := Button.new()
var selected := ""


func setup(the_game: EliteGame) -> void:
	game = the_game


func _ready() -> void:
	add_theme_constant_override("separation", 12)

	var left := VBoxContainer.new()
	left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left.add_theme_constant_override("separation", 10)
	add_child(left)

	var heading := Label.new()
	heading.text = "🗺️  Galaxy Chart"
	heading.add_theme_font_size_override("font_size", Theme_.FONT_TITLE)
	heading.add_theme_color_override("font_color", Theme_.ACCENT)
	left.add_child(heading)

	map.size_flags_vertical = Control.SIZE_EXPAND_FILL
	map.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	map.custom_minimum_size = Vector2(400, 300)
	map.system_selected.connect(_select)
	left.add_child(map)

	toggle.toggle_mode = true
	toggle.text = "Show whole galaxy"
	toggle.toggled.connect(func(whole: bool):
		map.set_local(not whole)
		toggle.text = "Show nearby" if whole else "Show whole galaxy")
	left.add_child(toggle)

	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(430, 0)
	add_child(panel)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 12)
	panel.add_child(box)

	title.add_theme_font_size_override("font_size", Theme_.FONT_TITLE)
	box.add_child(title)
	details.bbcode_enabled = true
	details.fit_content = false
	details.scroll_active = true
	details.custom_minimum_size = Vector2(0, 160)
	details.size_flags_vertical = Control.SIZE_EXPAND_FILL
	box.add_child(details)
	plan.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	box.add_child(plan)
	jump.text = "🚀  Jump"
	jump.disabled = true
	jump.tooltip_text = "Jumping arrives with the journey screen"
	box.add_child(jump)
	refresh()


func refresh() -> void:
	var data: Dictionary = game.galaxy()
	map.show_galaxy(data)
	if selected == "" or selected == data["current"]:
		selected = ""
		title.text = "Select a system"
		details.text = "Click a system on the chart.\n\nBright systems are within reach of your fuel."
		plan.text = ""
		return
	_select(selected)


func _select(name: String) -> void:
	selected = name
	map.set_selected(name)
	var info: Dictionary = game.system_detail(name)
	if info.is_empty():
		return
	title.text = info["name"]
	var text := ""
	text += "[color=#7d8da3]Economy[/color]  %s\n" % info["economy"]
	text += "[color=#7d8da3]Government[/color]  %s\n" % info["government"]
	text += "[color=#7d8da3]Tech level[/color]  %d\n" % info["tech_level"]
	text += "[color=#7d8da3]Population[/color]  %s\n\n" % info["population"]
	text += info["description"]
	details.text = text
	var jump_plan: Dictionary = game.plan_jump(name)
	if jump_plan["ok"]:
		plan.text = "%s → %s\n%.1f light years · %d fuel" % [jump_plan["origin"], jump_plan["destination"], jump_plan["distance"], jump_plan["fuel_cost"]]
		plan.add_theme_color_override("font_color", Theme_.GOOD)
	else:
		plan.text = "❌  %s" % jump_plan["message"]
		plan.add_theme_color_override("font_color", Theme_.BAD)
