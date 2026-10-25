extends VBoxContainer
## Refuel and repair at the station.

signal changed

const Theme_ := preload("res://theme/elite_theme.gd")

var game: EliteGame
var heading := Label.new()
var fuel_label := Label.new()
var hull_label := Label.new()
var feedback := Label.new()
var fuel_buttons := HBoxContainer.new()
var hull_buttons := HBoxContainer.new()


func setup(the_game: EliteGame) -> void:
	game = the_game


func _ready() -> void:
	add_theme_constant_override("separation", 14)
	heading.text = "🔧  Station Services"
	heading.add_theme_font_size_override("font_size", Theme_.FONT_TITLE)
	heading.add_theme_color_override("font_color", Theme_.ACCENT)
	add_child(heading)

	add_child(_section("⛽  Fuel", fuel_label, fuel_buttons))
	_add_button(fuel_buttons, "Fill the tank", func(): _act(game.refuel(0)))
	_add_button(fuel_buttons, "+1", func(): _act(game.refuel(1)))
	_add_button(fuel_buttons, "+3", func(): _act(game.refuel(3)))

	add_child(_section("🛡️  Hull", hull_label, hull_buttons))
	_add_button(hull_buttons, "Repair all", func(): _act(game.repair(0)))
	_add_button(hull_buttons, "+5", func(): _act(game.repair(5)))
	_add_button(hull_buttons, "+20", func(): _act(game.repair(20)))

	add_child(feedback)
	refresh()


func _section(title: String, label: Label, buttons: HBoxContainer) -> Control:
	var panel := PanelContainer.new()
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 10)
	panel.add_child(box)
	var name := Label.new()
	name.text = title
	name.add_theme_font_size_override("font_size", Theme_.FONT_STRONG)
	box.add_child(name)
	box.add_child(label)
	buttons.add_theme_constant_override("separation", 10)
	box.add_child(buttons)
	return panel


func _add_button(row: HBoxContainer, text: String, action: Callable) -> void:
	var button := Button.new()
	button.text = text
	button.pressed.connect(action)
	row.add_child(button)


func refresh() -> void:
	var services: Dictionary = game.services()
	fuel_label.text = _missing_text("light years", services["fuel_missing"], services["fuel_price"])
	hull_label.text = _missing_text("hull points", services["hull_missing"], services["repair_price"])
	for button in fuel_buttons.get_children():
		button.disabled = services["fuel_missing"] == 0
	for button in hull_buttons.get_children():
		button.disabled = services["hull_missing"] == 0


func _missing_text(unit: String, missing: int, price: int) -> String:
	if missing == 0:
		return "Full — nothing to buy."
	return "%d %s missing. %d cr each, %d cr to fill." % [missing, unit, price, missing * price]


func _act(result: Dictionary) -> void:
	feedback.text = ("✅  " if result["ok"] else "❌  ") + result["message"]
	feedback.add_theme_color_override("font_color", Theme_.GOOD if result["ok"] else Theme_.BAD)
	refresh()
	changed.emit()
